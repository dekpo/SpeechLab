//! One-off probe (D-036): does the sherpa-onnx NeMo transducer path (Parakeet TDT) honour hotwords?
//!
//!   cargo run --release --example hotwords_probe -- <wav> <hotwords.txt> [modeling-unit] [score]
//!
//! It builds the Parakeet recognizer with `modified_beam_search`, a hotwords file and a hotwords
//! score, decodes one WAV and prints what happened. The native library may terminate the process
//! when a combination is unsupported, so run it as a separate process and look at its exit code:
//! that outcome is the finding. Also prints the plain greedy result for the same file.
//!
//! Hotwords file: one hotword per line (sherpa-onnx syntax, optional ":score" suffix).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineTransducerModelConfig};
use speechlab_lib::speech::models::ModelManager;
use speechlab_lib::speech::wav;

fn models_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("SPEECHLAB_MODELS_DIR") {
        return PathBuf::from(p);
    }
    let base = std::env::var_os("APPDATA").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).expect("APPDATA or HOME");
    base.join("ai.assistantcabinet.speechlab").join("models")
}

fn config(models: &ModelManager, method: &str, hotwords: Option<(&str, &str, f32)>) -> OfflineRecognizerConfig {
    let def = models.def("sherpa-parakeet-tdt-0.6b-v3-int8").expect("model def");
    let dir = models.model_dir(def);
    let f = &def.files;
    let p = |x: &Option<String>| Some(dir.join(x.as_ref().expect("file")).to_string_lossy().into_owned());
    let mut c = OfflineRecognizerConfig::default();
    c.model_config.tokens = p(&f.tokens);
    c.model_config.num_threads = 4;
    c.model_config.provider = Some("cpu".into());
    c.model_config.transducer = OfflineTransducerModelConfig { encoder: p(&f.encoder), decoder: p(&f.decoder), joiner: p(&f.joiner) };
    c.model_config.model_type = Some("nemo_transducer".into());
    c.decoding_method = Some(method.into());
    if let Some((file, unit, score)) = hotwords {
        c.model_config.bpe_vocab = std::env::var("BPE_VOCAB").ok();
        c.hotwords_file = Some(file.into());
        c.hotwords_score = score;
        c.model_config.modeling_unit = Some(unit.into());
    }
    c
}

fn decode(c: &OfflineRecognizerConfig, samples: &wav::DecodedAudio) -> Option<(String, u128)> {
    decode_with(c, samples, None)
}

fn decode_with(c: &OfflineRecognizerConfig, samples: &wav::DecodedAudio, stream_hotwords: Option<&str>) -> Option<(String, u128)> {
    let r = OfflineRecognizer::create(c)?;
    let start = Instant::now();
    let s = match stream_hotwords {
        Some(h) => r.create_stream_with_hotwords(h),
        None => r.create_stream(),
    };
    s.accept_waveform(samples.sample_rate as i32, &samples.samples);
    r.decode(&s);
    let res = s.get_result()?;
    Some((res.text.trim().to_string(), start.elapsed().as_millis()))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let wav_path = args.first().expect("<wav>");
    let hot_file = args.get(1).expect("<hotwords.txt>");
    let unit = args.get(2).map(String::as_str).unwrap_or("cjkchar");
    let score: f32 = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(3.0);
    let models = Arc::new(ModelManager::new(models_dir()).expect("manifest"));
    let audio = wav::read_wav_mono(std::path::Path::new(wav_path)).expect("wav");

    match decode(&config(&models, "greedy_search", None), &audio) {
        Some((t, ms)) => println!("greedy        : {ms} ms : {t}"),
        None => println!("greedy        : recognizer could not be created"),
    }
    println!("trying modified_beam_search + hotwords (modeling unit {unit}, score {score}) ...");
    match decode(&config(&models, "modified_beam_search", Some((hot_file, unit, score))), &audio) {
        Some((t, ms)) => println!("hotwords      : {ms} ms : {t}"),
        None => println!("hotwords      : recognizer could not be created (unsupported or bad files)"),
    }
    // Per-stream hotwords: recognizer without a hotwords file, list given when the stream is created.
    let mut per_stream = config(&models, "modified_beam_search", None);
    per_stream.model_config.modeling_unit = Some(unit.into());
    per_stream.model_config.bpe_vocab = std::env::var("BPE_VOCAB").ok();
    per_stream.hotwords_score = score;
    let list: String = std::fs::read_to_string(hot_file).unwrap_or_default().lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("/");
    match decode_with(&per_stream, &audio, Some(&list)) {
        Some((t, ms)) => println!("per-stream    : {ms} ms : {t}"),
        None => println!("per-stream    : recognizer could not be created"),
    }
    // No hotwords, beam search only: separates the effect of the beam from the effect of the list.
    match decode(&config(&models, "modified_beam_search", None), &audio) {
        Some((t, ms)) => println!("beam only     : {ms} ms : {t}"),
        None => println!("beam only     : recognizer could not be created"),
    }
}
