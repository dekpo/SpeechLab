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
use super::probe::SelfSampler;
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
    /// Hotword experiment (D-036): NeMo transducer models use modified beam search with this
    /// hotwords score instead of greedy search. None = off, the default.
    hotword_score: Option<f32>,
}

/// Paths kept by modified beam search (sherpa-onnx's own default).
const BEAM_PATHS: i32 = 4;

impl SherpaOnnxProvider {
    pub fn new(models: Arc<ModelManager>) -> Self {
        Self {
            models,
            cache: Mutex::new(HashMap::new()),
            threads: DEFAULT_THREADS,
            hotword_score: None,
        }
    }

    /// Switches NeMo transducer models (Parakeet) to modified beam search, which is what allows
    /// hotwords; a request that carries a vocabulary then boosts those words with `score`. Without
    /// a vocabulary the same decoding runs with no boost, which is the control for the experiment.
    /// Other model families ignore it. Off by default: earlier results stay comparable.
    pub fn with_hotwords(mut self, score: f32) -> Self {
        self.hotword_score = Some(score);
        self
    }

    fn uses_hotword_decoding(&self, def: &ModelDef) -> bool {
        self.hotword_score.is_some() && def.family == ModelFamily::NemoTransducer
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
                    tail_paddings: whisper_tail_paddings(),
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
                if let Some(score) = self.hotword_score {
                    config.decoding_method = Some("modified_beam_search".into());
                    config.max_active_paths = BEAM_PATHS;
                    config.hotwords_score = score;
                    // Hotwords are cut into pieces with a SentencePiece vocabulary that the model
                    // download does not include; a surrogate is derived from tokens.txt (D-036).
                    config.model_config.modeling_unit = Some("bpe".into());
                    let tokens = dir.join(f.get(&f.tokens, "tokens")?);
                    config.model_config.bpe_vocab = Some(file_str(&surrogate_bpe_vocab(def, &tokens)?));
                }
            }
            ModelFamily::GgmlWhisper => {
                return Err(SpeechError::InvalidRequest(format!(
                    "model {} is a ggml model; it belongs to the whisper.cpp provider",
                    def.id
                )))
            }
            ModelFamily::SileroVad => {
                return Err(SpeechError::InvalidRequest(format!(
                    "model {} is a voice-activity detector, not a speech-to-text model",
                    def.id
                )))
            }
            ModelFamily::PiperVits | ModelFamily::Kokoro | ModelFamily::CoquiVits => {
                return Err(SpeechError::InvalidRequest(format!(
                    "model {} is a text-to-speech voice, not a speech-to-text model",
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
        let key = format!("{}:{}:{:?}", def.id, language, self.hotword_score);
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

/// SentencePiece vocabulary file ("piece<TAB>score" per line) that sherpa-onnx wants for cutting
/// hotwords into pieces. The model download only has `tokens.txt` ("piece id"), so the score is
/// taken as minus the id: SentencePiece numbers its pieces by decreasing score, which makes this
/// an order-preserving stand-in. The real scores are NOT known, so the cut of a hotword into pieces
/// can differ from the one the model learned; that is a limit of the experiment, not of hotwords.
pub fn surrogate_bpe_vocab_text(tokens_txt: &str) -> String {
    let mut out = String::new();
    for line in tokens_txt.lines().filter(|l| !l.trim().is_empty()) {
        if let Some((piece, id)) = line.rsplit_once(' ') {
            if let Ok(id) = id.trim().parse::<i64>() {
                out.push_str(&format!("{piece}\t{}\n", -id));
            }
        }
    }
    out
}

fn surrogate_bpe_vocab(def: &ModelDef, tokens: &Path) -> Result<std::path::PathBuf, SpeechError> {
    let dir = std::env::temp_dir().join("speechlab-hotwords");
    std::fs::create_dir_all(&dir).map_err(|e| SpeechError::Engine(format!("cannot create {}: {e}", dir.display())))?;
    let path = dir.join(format!("{}.surrogate-bpe.vocab", def.id));
    let text = std::fs::read_to_string(tokens)
        .map_err(|e| SpeechError::Engine(format!("cannot read {}: {e}", tokens.display())))?;
    std::fs::write(&path, surrogate_bpe_vocab_text(&text))
        .map_err(|e| SpeechError::Engine(format!("cannot write {}: {e}", path.display())))?;
    Ok(path)
}

/// Whisper "tail paddings" (frames of padding added after the audio). -1 = the library default.
/// `SPEECHLAB_SHERPA_WHISPER_TAIL_PADDINGS` overrides it for experiments (I-018, I-033).
fn whisper_tail_paddings() -> i32 {
    std::env::var("SPEECHLAB_SHERPA_WHISPER_TAIL_PADDINGS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(-1)
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

        let sampler = SelfSampler::start();
        let (recognizer, load_ms, cold_start) = self.recognizer(def, &request.language)?;
        if cancel.is_cancelled() {
            let _ = sampler.finish();
            return Err(SpeechError::Cancelled);
        }

        let start = Instant::now();
        let hotwords_active = self.uses_hotword_decoding(def) && !request.vocabulary.is_empty();
        let stream = if hotwords_active {
            // Hotwords are separated by "/" in sherpa-onnx's per-stream list.
            recognizer.create_stream_with_hotwords(&request.vocabulary.join("/"))
        } else {
            recognizer.create_stream()
        };
        stream.accept_waveform(audio.sample_rate as i32, &audio.samples);
        recognizer.decode(&stream);
        let result = stream
            .get_result()
            .ok_or_else(|| SpeechError::Engine("sherpa-onnx returned no result".into()))?;
        let processing_ms = start.elapsed().as_millis() as u64;
        let (peak_memory_mb, cpu_ms) = sampler.finish();

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
            peak_memory_mb,
            cpu_ms,
            decoding: match (self.hotword_score, self.uses_hotword_decoding(def), hotwords_active) {
                (Some(score), true, true) => format!("modified beam search ({BEAM_PATHS} paths) + hotwords (score {score})"),
                (_, true, false) => format!("modified beam search ({BEAM_PATHS} paths)"),
                _ => "greedy search".into(),
            },
            is_mock: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrogate_vocabulary_scores_follow_the_id_order() {
        let text = surrogate_bpe_vocab_text("<unk> 0\n▁the 5\nab 6\n\n");
        assert_eq!(text, "<unk>\t0\n▁the\t-5\nab\t-6\n");
    }

    #[test]
    fn hotword_decoding_is_off_by_default_and_only_for_nemo_transducers() {
        let m = Arc::new(ModelManager::new(std::env::temp_dir().join("speechlab_sh")).unwrap());
        let off = SherpaOnnxProvider::new(Arc::clone(&m));
        let on = SherpaOnnxProvider::new(Arc::clone(&m)).with_hotwords(1.5);
        let nemo = m.def("sherpa-parakeet-tdt-0.6b-v3-int8").unwrap();
        let whisper = m.def("sherpa-whisper-tiny").unwrap();
        assert!(!off.uses_hotword_decoding(nemo));
        assert!(on.uses_hotword_decoding(nemo));
        assert!(!on.uses_hotword_decoding(whisper));
    }
}
