//! Headless check of the speech pipeline (no UI).
//!
//!   cargo run --example transcribe -- list
//!   cargo run --example transcribe -- install <model-id>
//!   cargo run --example transcribe -- run <model-id> <fr|en> <file.wav> [repeat] [--chunking vad] [--out result.json]
//!
//! --chunking vad  cut a clip longer than 25 s at silences (needs: install silero-vad)
//! --out file      write the result as JSON to that file and do NOT print the text (private clips)
//!
//! The engine is chosen from the model's provider in the manifest.
//! Models dir: $SPEECHLAB_MODELS_DIR, else %APPDATA%\ai.assistantcabinet.speechlab\models.
//! whisper.cpp: set SPEECHLAB_WHISPER_BEAM_SIZE=1 for greedy decoding (default 5 beams).

use std::path::PathBuf;
use std::sync::Arc;

use speechlab_lib::speech::chunking::{transcribe_chunked, ChunkConfig};
use speechlab_lib::speech::models::ModelManager;
use speechlab_lib::speech::provider::{CancelToken, SpeechToTextProvider};
use speechlab_lib::speech::sherpa::SherpaOnnxProvider;
use speechlab_lib::speech::types::{DownloadPhase, TranscribeRequest};
use speechlab_lib::speech::vad::silero_speech_spans;
use speechlab_lib::speech::whisper_cpp::WhisperCppProvider;

fn default_models_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("SPEECHLAB_MODELS_DIR") {
        return PathBuf::from(p);
    }
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .expect("APPDATA or HOME must be set");
    base.join("ai.assistantcabinet.speechlab").join("models")
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // Options may appear anywhere after the command; take them out of the positional list.
    let mut option = |name: &str| -> Option<String> {
        let i = args.iter().position(|a| a == name)?;
        args.remove(i);
        (i < args.len()).then(|| args.remove(i))
    };
    let chunking = option("--chunking");
    let out_file = option("--out");
    let models = Arc::new(ModelManager::new(default_models_dir()).expect("manifest"));
    println!("models dir: {}", models.models_dir().display());
    match args.first().map(String::as_str) {
        Some("list") => {
            for m in models.list().into_iter().chain(models.support_models()) {
                println!(
                    "{:34} {:12} {:?} {} MB  {}",
                    m.id,
                    m.provider,
                    m.install_status,
                    m.size_bytes / 1_000_000,
                    m.license
                );
            }
        }
        Some("install") => {
            let id = args.get(1).expect("model id");
            let mut last = 0u64;
            let sha = models
                .install(id, &CancelToken::new(), &mut |p| {
                    let step = p.phase != DownloadPhase::Download
                        || p.done_bytes / 20_000_000 != last / 20_000_000;
                    if step {
                        println!("  {:?} {}/{} bytes", p.phase, p.done_bytes, p.total_bytes);
                    }
                    last = p.done_bytes;
                })
                .expect("install failed");
            println!("installed {id}, sha256={sha}");
        }
        Some("run") => {
            let (id, lang, wav) = (&args[1], &args[2], &args[3]);
            let repeat: usize = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(1);
            let vad_model = match chunking.as_deref() {
                None | Some("off") => None,
                Some("vad") => Some(models.vad_model_path().expect("install the support model first: install silero-vad")),
                Some(other) => panic!("--chunking expects 'vad' or 'off', got '{other}'"),
            };
            let provider_id = models.def(id).expect("unknown model").provider.clone();
            let provider: Box<dyn SpeechToTextProvider> = match provider_id.as_str() {
                "sherpa-onnx" => Box::new(SherpaOnnxProvider::new(models)),
                "whisper-cpp" => Box::new(WhisperCppProvider::new(models)),
                other => panic!("no provider for {other}"),
            };
            let cancel = CancelToken::new();
            for i in 1..=repeat {
                let request = TranscribeRequest {
                    provider_id: provider_id.clone(),
                    model_id: id.clone(),
                    language: lang.clone(),
                    audio_path: wav.clone(),
                    vocabulary: Vec::new(),
                };
                let (r, segments, chunking_ms, bounds) = match &vad_model {
                    Some(model) => {
                        let detect = |samples: &[f32]| silero_speech_spans(model, samples);
                        let (r, report) = transcribe_chunked(provider.as_ref(), &request, &cancel, &detect, &ChunkConfig::default())
                            .expect("transcription failed");
                        (r, report.segments, report.chunking_ms, report.bounds_ms)
                    }
                    None => (provider.transcribe(&request, &cancel).expect("transcription failed"), 1, 0, Vec::new()),
                };
                println!(
                    "run {i}: {} cold={} load={}ms infer={}ms audio={}ms rtf={:.3} threads={} decoding={} segments={segments} chunking={chunking_ms}ms bounds_ms={bounds:?}",
                    r.provider_id,
                    r.cold_start,
                    r.load_ms,
                    r.processing_ms,
                    r.audio_ms.unwrap_or(0),
                    r.rtf.unwrap_or(0.0),
                    r.threads,
                    r.decoding,
                );
                match &out_file {
                    Some(path) => {
                        let mut json = serde_json::to_value(&r).expect("json");
                        json["segments"] = segments.into();
                        json["chunkingMs"] = chunking_ms.into();
                        json["chunking"] = (if vad_model.is_some() { "vad" } else { "off" }).into();
                        std::fs::write(path, serde_json::to_string_pretty(&json).expect("json")).expect("write --out");
                        println!("  result written to {path} ({} words)", r.text.split_whitespace().count());
                    }
                    None => println!("  text: {}", r.text),
                }
            }
        }
        _ => eprintln!("usage: list | install <id> | run <id> <fr|en> <wav> [repeat] [--chunking vad] [--out file]"),
    }
}
