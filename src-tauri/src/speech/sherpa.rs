//! sherpa-onnx speech-to-text adapter. The only place that knows sherpa-onnx.
//!
//! Inference is fully local: this module never touches the network (see `download.rs`).
//! The sherpa-onnx offline decode call is blocking and cannot be interrupted, so
//! cancellation is only honoured before a run starts (`supports_cancellation = false`).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use sherpa_onnx::{
    OfflineCanaryModelConfig, OfflineRecognizer, OfflineRecognizerConfig,
    OfflineTransducerModelConfig, OfflineWhisperModelConfig,
};

use super::error::SpeechError;
use super::models::{ModelDef, ModelFamily, ModelManager};
use super::provider::{CancelToken, SpeechToTextProvider};
use super::types::{Capabilities, ProviderInfo, ProviderKind, TranscribeRequest, TranscribeResult};
use super::wav;

pub const SHERPA_ID: &str = "sherpa-onnx";
const DEFAULT_THREADS: u32 = 4;

pub struct SherpaOnnxProvider {
    models: Arc<ModelManager>,
    /// Loaded recognizers keyed by "model:language" (language is baked into the config).
    cache: Mutex<HashMap<String, Arc<OfflineRecognizer>>>,
    threads: u32,
}

impl SherpaOnnxProvider {
    pub fn new(models: Arc<ModelManager>) -> Self {
        Self {
            models,
            cache: Mutex::new(HashMap::new()),
            threads: DEFAULT_THREADS,
        }
    }

    fn build_config(
        &self,
        def: &ModelDef,
        language: &str,
    ) -> Result<OfflineRecognizerConfig, SpeechError> {
        let dir = self.models.model_dir(def);
        let f = &def.files;
        let path = |field: &Option<String>, what: &str| -> Result<Option<String>, SpeechError> {
            Ok(Some(file_str(&dir.join(f.get(field, what)?))))
        };
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.tokens = path(&f.tokens, "tokens")?;
        config.model_config.num_threads = self.threads as i32;
        config.model_config.provider = Some("cpu".into());
        config.decoding_method = Some("greedy_search".into());

        match def.family {
            ModelFamily::Whisper => {
                config.model_config.whisper = OfflineWhisperModelConfig {
                    encoder: path(&f.encoder, "encoder")?,
                    decoder: path(&f.decoder, "decoder")?,
                    language: Some(language.to_string()),
                    task: Some("transcribe".into()),
                    tail_paddings: -1,
                    ..Default::default()
                };
            }
            ModelFamily::Canary => {
                config.model_config.canary = OfflineCanaryModelConfig {
                    encoder: path(&f.encoder, "encoder")?,
                    decoder: path(&f.decoder, "decoder")?,
                    src_lang: Some(language.to_string()),
                    tgt_lang: Some(language.to_string()),
                    use_pnc: true,
                };
            }
            ModelFamily::NemoTransducer => {
                config.model_config.transducer = OfflineTransducerModelConfig {
                    encoder: path(&f.encoder, "encoder")?,
                    decoder: path(&f.decoder, "decoder")?,
                    joiner: path(&f.joiner, "joiner")?,
                };
                config.model_config.model_type = Some("nemo_transducer".into());
            }
            ModelFamily::GgmlWhisper => {
                return Err(SpeechError::InvalidRequest(format!(
                    "model {} is a ggml model; it belongs to the whisper.cpp provider",
                    def.id
                )))
            }
        }
        Ok(config)
    }

    /// Returns the recognizer and the load time in ms (0 when already loaded).
    fn recognizer(
        &self,
        def: &ModelDef,
        language: &str,
    ) -> Result<(Arc<OfflineRecognizer>, u64, bool), SpeechError> {
        let key = format!("{}:{}", def.id, language);
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| SpeechError::Engine("recognizer cache poisoned".into()))?;
        if let Some(r) = cache.get(&key) {
            return Ok((Arc::clone(r), 0, false));
        }
        let config = self.build_config(def, language)?;
        let start = Instant::now();
        let recognizer = OfflineRecognizer::create(&config).ok_or_else(|| {
            SpeechError::Engine(format!(
                "sherpa-onnx could not create a recognizer for {} (check the model files)",
                def.id
            ))
        })?;
        let load_ms = start.elapsed().as_millis() as u64;
        let recognizer = Arc::new(recognizer);
        cache.insert(key, Arc::clone(&recognizer));
        Ok((recognizer, load_ms, true))
    }
}

fn file_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

impl SpeechToTextProvider for SherpaOnnxProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: SHERPA_ID.into(),
            display_name: "sherpa-onnx (CPU)".into(),
            kind: ProviderKind::Stt,
            is_mock: false,
            languages: vec!["fr".into(), "en".into()],
            capabilities: Capabilities {
                supports_cancellation: false,
                supports_language_auto_detect: false,
                supports_streaming: false,
                supports_speed: false,
            },
        }
    }

    fn transcribe(
        &self,
        request: &TranscribeRequest,
        cancel: &CancelToken,
    ) -> Result<TranscribeResult, SpeechError> {
        let def = self.models.def(&request.model_id)?;
        if def.provider != SHERPA_ID {
            return Err(SpeechError::InvalidRequest(format!(
                "model {} does not belong to {SHERPA_ID}",
                def.id
            )));
        }
        if !def.languages.iter().any(|l| l == &request.language) {
            return Err(SpeechError::InvalidRequest(format!(
                "model {} does not support language '{}'",
                def.id, request.language
            )));
        }
        if !self.models.is_installed(def) {
            return Err(SpeechError::ModelNotInstalled(def.id.clone()));
        }
        if request.audio_path.trim().is_empty() {
            return Err(SpeechError::InvalidRequest("audio path is empty".into()));
        }

        let audio = wav::read_wav_mono(Path::new(&request.audio_path))?;
        if cancel.is_cancelled() {
            return Err(SpeechError::Cancelled);
        }

        let (recognizer, load_ms, cold_start) = self.recognizer(def, &request.language)?;
        if cancel.is_cancelled() {
            return Err(SpeechError::Cancelled);
        }

        let start = Instant::now();
        let stream = recognizer.create_stream();
        stream.accept_waveform(audio.sample_rate as i32, &audio.samples);
        recognizer.decode(&stream);
        let result = stream
            .get_result()
            .ok_or_else(|| SpeechError::Engine("sherpa-onnx returned no result".into()))?;
        let processing_ms = start.elapsed().as_millis() as u64;

        let audio_ms = audio.duration_ms();
        Ok(TranscribeResult {
            provider_id: SHERPA_ID.into(),
            model_id: def.id.clone(),
            language: request.language.clone(),
            text: result.text.trim().to_string(),
            processing_ms,
            load_ms,
            cold_start,
            audio_ms: Some(audio_ms),
            rtf: (audio_ms > 0).then(|| processing_ms as f64 / audio_ms as f64),
            threads: self.threads,
            decoding: "greedy search".into(),
            is_mock: false,
        })
    }
}
