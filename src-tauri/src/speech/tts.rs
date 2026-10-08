//! sherpa-onnx text-to-speech adapter (M6). Together with `sherpa.rs` the only place that knows
//! sherpa-onnx; it never touches the network (models come from `models.rs`).
//!
//! Voices come from the manifest (`role: "tts"`): Piper/VITS packages (one voice, or many
//! speakers) and the Kokoro multi-voice package. A voice id is `"<model id>:<speaker id>"`.
//! The library lets a callback stop generation between chunks, so cancellation is real, but the
//! first chunk is not interruptible. The phonemizer inside these packages is espeak-ng (GPL-3.0),
//! see D-012.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig, OfflineTtsModelConfig,
    OfflineTtsVitsModelConfig,
};

use super::error::SpeechError;
use super::models::{ModelDef, ModelFamily, ModelRole, ModelManager};
use super::probe::SelfSampler;
use super::provider::{CancelToken, TextToSpeechProvider};
use super::types::{
    Capabilities, ProviderInfo, ProviderKind, SynthesizeRequest, SynthesizeResult, VoiceInfo,
};

pub const SHERPA_TTS_ID: &str = "sherpa-onnx-tts";
const DEFAULT_THREADS: i32 = 4;
/// Longest text accepted in one request (the library splits it into sentences itself).
pub const MAX_TEXT_CHARS: usize = 5000;
pub const MIN_SPEED: f32 = 0.5;
pub const MAX_SPEED: f32 = 2.0;
/// A package with hundreds of speakers is listed with this many; any speaker number still works.
const MAX_LISTED_SPEAKERS: i32 = 12;

/// Speaker order of the Kokoro multi-language v1.0 package (from the package's own
/// `voices.bin` generator script). The first letter is the language, the second the gender
/// (documented in the model's VOICES.md); only French and English voices are offered.
const KOKORO_V1_0_VOICES: &[(i32, &str)] = &[
    (0, "af_alloy"), (1, "af_aoede"), (2, "af_bella"), (3, "af_heart"), (4, "af_jessica"),
    (5, "af_kore"), (6, "af_nicole"), (7, "af_nova"), (8, "af_river"), (9, "af_sarah"),
    (10, "af_sky"), (11, "am_adam"), (12, "am_echo"), (13, "am_eric"), (14, "am_fenrir"),
    (15, "am_liam"), (16, "am_michael"), (17, "am_onyx"), (18, "am_puck"), (19, "am_santa"),
    (20, "bf_alice"), (21, "bf_emma"), (22, "bf_isabella"), (23, "bf_lily"), (24, "bm_daniel"),
    (25, "bm_fable"), (26, "bm_george"), (27, "bm_lewis"), (30, "ff_siwis"),
];
const KOKORO_SPEAKER_COUNT: i32 = 54;

pub struct SherpaTtsProvider {
    models: Arc<ModelManager>,
    out_dir: PathBuf,
    /// Loaded voices keyed by "model:language".
    cache: Mutex<HashMap<String, Arc<OfflineTts>>>,
    threads: i32,
}

/// Folder of generated audio, next to the models folder.
pub fn output_dir(models: &ModelManager) -> PathBuf {
    models.models_dir().with_file_name("tts-output")
}

/// Reads a generated WAV for playback. Only `.wav` files directly inside `dir` are served, so the
/// command cannot be used to read arbitrary files.
pub fn read_generated(dir: &Path, path: &str) -> Result<Vec<u8>, SpeechError> {
    let file = Path::new(path);
    let ok_name = file.extension().and_then(|e| e.to_str()) == Some("wav");
    let inside = match (file.canonicalize(), dir.canonicalize()) {
        (Ok(f), Ok(d)) => f.parent() == Some(d.as_path()),
        _ => false,
    };
    if !ok_name || !inside {
        return Err(SpeechError::InvalidRequest("not a generated audio file".into()));
    }
    fs::read(file).map_err(|e| SpeechError::Audio(e.to_string()))
}

/// Deletes every generated WAV in `dir`; returns how many were removed.
pub fn clear_generated(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else { return 0 };
    entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("wav"))
        .filter(|p| fs::remove_file(p).is_ok())
        .count()
}

/// "model:sid" -> (model id, speaker id).
pub fn parse_voice_id(voice_id: &str) -> Result<(&str, i32), SpeechError> {
    let (model, sid) = voice_id
        .rsplit_once(':')
        .ok_or_else(|| SpeechError::InvalidRequest(format!("voice id '{voice_id}' is not <model>:<speaker>")))?;
    let sid = sid
        .parse::<i32>()
        .ok()
        .filter(|s| *s >= 0)
        .ok_or_else(|| SpeechError::InvalidRequest(format!("voice id '{voice_id}' has a bad speaker number")))?;
    Ok((model, sid))
}

fn kokoro_voice_language(name: &str) -> Option<&'static str> {
    match name.chars().next()? {
        'a' | 'b' => Some("en"),
        'f' => Some("fr"),
        _ => None,
    }
}

fn kokoro_voice_gender(name: &str) -> &'static str {
    match name.chars().nth(1) {
        Some('f') => "female",
        Some('m') => "male",
        _ => "unknown",
    }
}

/// Speaker count and sample rate of a Piper package, from the `<model>.onnx.json` next to it.
fn piper_metadata(dir: &Path, model_file: &str) -> (i32, Option<u32>) {
    let json = fs::read_to_string(dir.join(format!("{model_file}.json"))).ok();
    let value: Option<serde_json::Value> = json.and_then(|j| serde_json::from_str(&j).ok());
    let speakers = value.as_ref().and_then(|v| v["num_speakers"].as_i64()).unwrap_or(1).max(1) as i32;
    let rate = value.as_ref().and_then(|v| v["audio"]["sample_rate"].as_u64()).map(|r| r as u32);
    (speakers, rate)
}

fn path_str(dir: &Path, name: &str) -> String {
    dir.join(name).to_string_lossy().into_owned()
}

impl SherpaTtsProvider {
    pub fn new(models: Arc<ModelManager>, out_dir: PathBuf) -> Self {
        Self { models, out_dir, cache: Mutex::new(HashMap::new()), threads: DEFAULT_THREADS }
    }

    fn tts_def(&self, model_id: &str) -> Result<&ModelDef, SpeechError> {
        let def = self.models.def(model_id)?;
        if def.role != ModelRole::Tts {
            return Err(SpeechError::InvalidRequest(format!("model {model_id} is not a text-to-speech voice")));
        }
        Ok(def)
    }

    fn build_config(&self, def: &ModelDef, language: &str) -> Result<OfflineTtsConfig, SpeechError> {
        let dir = self.models.model_dir(def);
        let f = &def.files;
        let model = f.get(&f.model, "model")?;
        let tokens = f.get(&f.tokens, "tokens")?;
        let mut model_config = OfflineTtsModelConfig { num_threads: self.threads, debug: false, provider: Some("cpu".into()), ..Default::default() };
        match def.family {
            ModelFamily::PiperVits => {
                model_config.vits = OfflineTtsVitsModelConfig {
                    model: Some(path_str(&dir, model)),
                    tokens: Some(path_str(&dir, tokens)),
                    data_dir: Some(path_str(&dir, f.get(&f.data_dir, "dataDir")?)),
                    ..Default::default()
                };
            }
            ModelFamily::CoquiVits => {
                // Character-based: no phonemizer data folder.
                model_config.vits = OfflineTtsVitsModelConfig {
                    model: Some(path_str(&dir, model)),
                    tokens: Some(path_str(&dir, tokens)),
                    ..Default::default()
                };
            }
            ModelFamily::Kokoro => {
                let lexicon = f.lexicon.as_deref().map(|l| l.split(',').map(|n| path_str(&dir, n)).collect::<Vec<_>>().join(","));
                model_config.kokoro = OfflineTtsKokoroModelConfig {
                    model: Some(path_str(&dir, model)),
                    voices: Some(path_str(&dir, f.get(&f.voices, "voices")?)),
                    tokens: Some(path_str(&dir, tokens)),
                    data_dir: Some(path_str(&dir, f.get(&f.data_dir, "dataDir")?)),
                    dict_dir: Some(path_str(&dir, "dict")),
                    lexicon,
                    // English text is handled by the lexicon; other languages go through espeak-ng.
                    lang: (language != "en").then(|| language.to_string()),
                    ..Default::default()
                };
            }
            _ => {
                return Err(SpeechError::InvalidRequest(format!("model {} is not a text-to-speech voice", def.id)));
            }
        }
        Ok(OfflineTtsConfig { model: model_config, max_num_sentences: 1, silence_scale: 0.2, ..Default::default() })
    }

    /// Returns the loaded voice, the load time in ms (0 when already loaded) and whether it was cold.
    fn engine(&self, def: &ModelDef, language: &str) -> Result<(Arc<OfflineTts>, u64, bool), SpeechError> {
        let key = format!("{}:{language}", def.id);
        let mut cache = self.cache.lock().map_err(|_| SpeechError::Engine("voice cache poisoned".into()))?;
        if let Some(e) = cache.get(&key) {
            return Ok((Arc::clone(e), 0, false));
        }
        let config = self.build_config(def, language)?;
        let start = Instant::now();
        let engine = OfflineTts::create(&config)
            .ok_or_else(|| SpeechError::Engine(format!("sherpa-onnx could not load voice {}", def.id)))?;
        let load_ms = start.elapsed().as_millis() as u64;
        let engine = Arc::new(engine);
        cache.insert(key, Arc::clone(&engine));
        Ok((engine, load_ms, true))
    }

    fn voices_of(&self, def: &ModelDef) -> Vec<VoiceInfo> {
        let dir = self.models.model_dir(def);
        let mk = |sid: i32, name: String, language: &str, gender: &str, count: i32| VoiceInfo {
            id: format!("{}:{sid}", def.id),
            display_name: name,
            language: language.to_string(),
            gender: gender.to_string(),
            license: def.license.clone(),
            license_tier: def.license_tier.clone(),
            model_id: def.id.clone(),
            speaker_id: sid,
            speaker_count: count,
        };
        match def.family {
            ModelFamily::Kokoro => KOKORO_V1_0_VOICES
                .iter()
                .filter_map(|(sid, name)| {
                    let lang = kokoro_voice_language(name)?;
                    def.languages.iter().any(|l| l == lang).then(|| {
                        mk(*sid, format!("{name} (Kokoro)"), lang, kokoro_voice_gender(name), KOKORO_SPEAKER_COUNT)
                    })
                })
                .collect(),
            ModelFamily::PiperVits => {
                let (count, _) = piper_metadata(&dir, def.files.model.as_deref().unwrap_or_default());
                let lang = def.languages.first().cloned().unwrap_or_default();
                // The package cards do not document the speakers' gender, so it is never guessed.
                (0..count.min(MAX_LISTED_SPEAKERS))
                    .map(|sid| {
                        let name = if count > 1 {
                            format!("{} - speaker {sid} of {count}", def.display_name)
                        } else {
                            def.display_name.clone()
                        };
                        mk(sid, name, &lang, "unknown", count)
                    })
                    .collect()
            }
            ModelFamily::CoquiVits => {
                let lang = def.languages.first().cloned().unwrap_or_default();
                vec![mk(0, def.display_name.clone(), &lang, "unknown", 1)]
            }
            _ => Vec::new(),
        }
    }
}

impl TextToSpeechProvider for SherpaTtsProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: SHERPA_TTS_ID.into(),
            display_name: "sherpa-onnx text-to-speech (CPU)".into(),
            kind: ProviderKind::Tts,
            is_mock: false,
            languages: vec!["fr".into(), "en".into()],
            capabilities: Capabilities {
                supports_cancellation: true,
                supports_language_auto_detect: false,
                supports_streaming: false,
                // The parameter exists in the library; its audible effect per voice is measured, not assumed.
                supports_speed: true,
            },
        }
    }

    fn voices(&self) -> Result<Vec<VoiceInfo>, SpeechError> {
        let mut out = Vec::new();
        for info in self.models.tts_models() {
            let def = self.models.def(&info.id)?;
            if self.models.is_installed(def) {
                out.extend(self.voices_of(def));
            }
        }
        Ok(out)
    }

    fn synthesize(&self, request: &SynthesizeRequest, cancel: &CancelToken) -> Result<SynthesizeResult, SpeechError> {
        let (model_id, sid) = parse_voice_id(&request.voice_id)?;
        let def = self.tts_def(model_id)?;
        if !self.models.is_installed(def) {
            return Err(SpeechError::ModelNotInstalled(def.id.clone()));
        }
        if !def.languages.iter().any(|l| l == &request.language) {
            return Err(SpeechError::InvalidRequest(format!("voice {} does not support language '{}'", def.id, request.language)));
        }
        let listed = self.voices_of(def);
        let voice = match listed.iter().find(|v| v.speaker_id == sid) {
            Some(v) => v.clone(),
            // Any speaker number of a multi-speaker Piper package is valid even when it is not listed.
            None if def.family == ModelFamily::PiperVits => listed
                .first()
                .filter(|v| sid < v.speaker_count)
                .cloned()
                .ok_or_else(|| SpeechError::InvalidRequest(format!("voice {} has no speaker {sid}", def.id)))?,
            None => return Err(SpeechError::InvalidRequest(format!("voice {} has no speaker {sid}", def.id))),
        };
        if voice.language != request.language && def.family == ModelFamily::Kokoro {
            return Err(SpeechError::InvalidRequest(format!(
                "voice {} speaks '{}', not '{}'",
                voice.display_name, voice.language, request.language
            )));
        }
        let text = request.text.trim();
        if text.is_empty() {
            return Err(SpeechError::InvalidRequest("the text is empty".into()));
        }
        if text.chars().count() > MAX_TEXT_CHARS {
            return Err(SpeechError::InvalidRequest(format!("the text is longer than {MAX_TEXT_CHARS} characters")));
        }
        if !(MIN_SPEED..=MAX_SPEED).contains(&request.speed) {
            return Err(SpeechError::InvalidRequest(format!("speed must be between {MIN_SPEED} and {MAX_SPEED}")));
        }
        if cancel.is_cancelled() {
            return Err(SpeechError::Cancelled);
        }

        let sampler = SelfSampler::start();
        let (engine, load_ms, cold_start) = self.engine(def, &request.language)?;
        if cancel.is_cancelled() {
            let _ = sampler.finish();
            return Err(SpeechError::Cancelled);
        }
        let config = GenerationConfig { speed: request.speed, sid, ..Default::default() };
        let stop = cancel.clone();
        let start = Instant::now();
        let audio = engine.generate_with_config(text, &config, Some(move |_samples: &[f32], _progress: f32| !stop.is_cancelled()));
        let generation_ms = start.elapsed().as_millis() as u64;
        let (peak_memory_mb, _) = sampler.finish();
        if cancel.is_cancelled() {
            return Err(SpeechError::Cancelled);
        }
        let audio = audio.ok_or_else(|| SpeechError::Engine("sherpa-onnx produced no audio".into()))?;
        let sample_rate = audio.sample_rate().max(0) as u32;
        let frames = audio.samples().len() as u64;
        if frames == 0 || sample_rate == 0 {
            return Err(SpeechError::Engine("sherpa-onnx produced empty audio".into()));
        }
        let audio_ms = frames * 1000 / sample_rate as u64;

        fs::create_dir_all(&self.out_dir).map_err(|e| SpeechError::Audio(e.to_string()))?;
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let file = self.out_dir.join(format!("{}-{sid}-{stamp}.wav", def.id));
        if !audio.save(&file.to_string_lossy()) {
            return Err(SpeechError::Audio(format!("cannot write {}", file.display())));
        }
        Ok(SynthesizeResult {
            provider_id: SHERPA_TTS_ID.into(),
            voice_id: request.voice_id.clone(),
            wav_path: file.to_string_lossy().into_owned(),
            generation_ms,
            audio_ms,
            load_ms,
            cold_start,
            sample_rate,
            rtf: (audio_ms > 0).then(|| generation_ms as f64 / audio_ms as f64),
            speed: request.speed,
            peak_memory_mb,
            is_mock: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> SherpaTtsProvider {
        let models = Arc::new(ModelManager::new(std::env::temp_dir().join("speechlab_tts_test_models")).unwrap());
        SherpaTtsProvider::new(models, std::env::temp_dir().join("speechlab_tts_test_out"))
    }

    #[test]
    fn generated_audio_is_served_only_from_the_output_folder() {
        let dir = std::env::temp_dir().join("speechlab_tts_guard");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let inside = dir.join("a.wav");
        fs::write(&inside, b"RIFFdata").unwrap();
        let outside = std::env::temp_dir().join("speechlab_tts_guard_outside.wav");
        fs::write(&outside, b"RIFFdata").unwrap();
        let not_wav = dir.join("notes.txt");
        fs::write(&not_wav, b"x").unwrap();
        assert_eq!(read_generated(&dir, &inside.to_string_lossy()).unwrap(), b"RIFFdata");
        assert!(read_generated(&dir, &outside.to_string_lossy()).is_err());
        assert!(read_generated(&dir, &not_wav.to_string_lossy()).is_err());
        assert!(read_generated(&dir, &dir.join("..").join("speechlab_tts_guard").join("a.wav").to_string_lossy()).is_ok(), "same file by another spelling");
        assert!(read_generated(&dir, &dir.join("missing.wav").to_string_lossy()).is_err());
        assert_eq!(clear_generated(&dir), 1);
        assert!(not_wav.is_file() && !inside.exists());
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(outside);
    }

    #[test]
    fn voice_ids_round_trip_and_bad_ones_are_rejected() {
        assert_eq!(parse_voice_id("tts-kokoro-int8-multi-lang-v1_0:30").unwrap(), ("tts-kokoro-int8-multi-lang-v1_0", 30));
        assert!(parse_voice_id("no-colon").is_err());
        assert!(parse_voice_id("m:-1").is_err());
        assert!(parse_voice_id("m:x").is_err());
    }

    #[test]
    fn kokoro_voices_offer_only_french_and_english_with_documented_gender() {
        let langs: Vec<_> = KOKORO_V1_0_VOICES.iter().filter_map(|(_, n)| kokoro_voice_language(n)).collect();
        assert_eq!(langs.iter().filter(|l| **l == "fr").count(), 1, "one French voice in this package");
        assert_eq!(kokoro_voice_gender("ff_siwis"), "female");
        assert_eq!(kokoro_voice_gender("bm_george"), "male");
        assert_eq!(kokoro_voice_language("zf_xiaobei"), None);
    }

    #[test]
    fn provider_reports_real_capabilities_and_no_voices_when_nothing_is_installed() {
        let p = provider();
        let info = p.info();
        assert!(!info.is_mock && info.kind == ProviderKind::Tts);
        assert!(info.capabilities.supports_cancellation);
        assert!(p.voices().unwrap().is_empty());
    }

    /// Real-engine tests run only when the voice is installed in the normal models folder.
    fn installed_provider(model_id: &str) -> Option<SherpaTtsProvider> {
        let dir = std::env::var_os("SPEECHLAB_MODELS_DIR")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("ai.assistantcabinet.speechlab").join("models")))?;
        let models = Arc::new(ModelManager::new(dir).ok()?);
        let installed = models.def(model_id).map(|d| models.is_installed(d)).unwrap_or(false);
        installed.then(|| SherpaTtsProvider::new(models, std::env::temp_dir().join("speechlab_tts_real_out")))
    }

    fn req(voice: &str, text: &str, speed: f32) -> SynthesizeRequest {
        SynthesizeRequest { provider_id: SHERPA_TTS_ID.into(), voice_id: voice.into(), language: "fr".into(), text: text.into(), speed }
    }

    #[test]
    fn real_voice_makes_audio_follows_speed_and_stops_on_cancel() {
        const VOICE: &str = "tts-vits-piper-fr_FR-siwis-medium";
        let Some(p) = installed_provider(VOICE) else {
            eprintln!("skipped: {VOICE} is not installed");
            return;
        };
        let id = format!("{VOICE}:0");
        let cancel = CancelToken::new();
        let normal = p.synthesize(&req(&id, "Le patient prend cinq cents milligrammes.", 1.0), &cancel).unwrap();
        assert!(normal.audio_ms > 1000 && normal.sample_rate == 22050 && normal.cold_start == (normal.load_ms > 0));
        assert!(Path::new(&normal.wav_path).is_file());
        let slow = p.synthesize(&req(&id, "Le patient prend cinq cents milligrammes.", 0.5), &cancel).unwrap();
        let fast = p.synthesize(&req(&id, "Le patient prend cinq cents milligrammes.", 2.0), &cancel).unwrap();
        assert!(slow.audio_ms > normal.audio_ms && normal.audio_ms > fast.audio_ms, "{} {} {}", slow.audio_ms, normal.audio_ms, fast.audio_ms);
        assert_eq!(slow.load_ms, 0, "the voice stays loaded");

        // Cancelled before the start: nothing is generated.
        let pre = CancelToken::new();
        pre.cancel();
        assert_eq!(p.synthesize(&req(&id, "Bonjour.", 1.0), &pre).unwrap_err(), SpeechError::Cancelled);

        // Cancelled during a long text: it stops long before the whole text would be done.
        let long = "Le dossier du patient a été transmis au secrétariat ce matin. ".repeat(40);
        let whole = Instant::now();
        let full = p.synthesize(&req(&id, &long, 1.0), &CancelToken::new()).unwrap();
        let full_ms = whole.elapsed().as_millis() as u64;
        let c = CancelToken::new();
        let c2 = c.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(full_ms / 5));
            c2.cancel();
        });
        let started = Instant::now();
        assert_eq!(p.synthesize(&req(&id, &long, 1.0), &c).unwrap_err(), SpeechError::Cancelled);
        assert!(started.elapsed().as_millis() as u64 * 2 < full_ms, "cancel took {} ms of {} ms ({} ms of audio)", started.elapsed().as_millis(), full_ms, full.audio_ms);
        let _ = fs::remove_dir_all(std::env::temp_dir().join("speechlab_tts_real_out"));
    }

    #[test]
    fn requests_are_validated_before_any_model_is_touched() {
        let p = provider();
        let cancel = CancelToken::new();
        let base = SynthesizeRequest {
            provider_id: SHERPA_TTS_ID.into(),
            voice_id: "tts-vits-piper-fr_FR-siwis-medium:0".into(),
            language: "fr".into(),
            text: "Bonjour.".into(),
            speed: 1.0,
        };
        assert!(matches!(p.synthesize(&base, &cancel), Err(SpeechError::ModelNotInstalled(_))));
        let mut r = base.clone();
        r.voice_id = "stt-model:0".into();
        assert!(p.synthesize(&r, &cancel).is_err());
        r = base.clone();
        r.voice_id = "garbage".into();
        assert!(matches!(p.synthesize(&r, &cancel), Err(SpeechError::InvalidRequest(_))));
    }
}
