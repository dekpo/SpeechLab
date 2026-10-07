//! Headless check of the speech pipeline (no UI).
//!
//!   cargo run --example transcribe -- list
//!   cargo run --example transcribe -- install <model-id>
//!   cargo run --example transcribe -- run <model-id> <fr|en> <file.wav> [repeat]
//!
//! The engine is chosen from the model's provider in the manifest.
//! Models dir: $SPEECHLAB_MODELS_DIR, else %APPDATA%\ai.assistantcabinet.speechlab\models.
//! whisper.cpp: set SPEECHLAB_WHISPER_BEAM_SIZE=1 for greedy decoding (default 5 beams).

use std::path::PathBuf;
use std::sync::Arc;

use speechlab_lib::speech::models::ModelManager;
use speechlab_lib::speech::provider::{CancelToken, SpeechToTextProvider};
use speechlab_lib::speech::sherpa::SherpaOnnxProvider;
use speechlab_lib::speech::types::{DownloadPhase, TranscribeRequest};
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let models = Arc::new(ModelManager::new(default_models_dir()).expect("manifest"));
    println!("models dir: {}", models.models_dir().display());
    match args.first().map(String::as_str) {
        Some("list") => {
            for m in models.list() {
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
            let provider_id = models.def(id).expect("unknown model").provider.clone();
            let provider: Box<dyn SpeechToTextProvider> = match provider_id.as_str() {
                "sherpa-onnx" => Box::new(SherpaOnnxProvider::new(models)),
                "whisper-cpp" => Box::new(WhisperCppProvider::new(models)),
                other => panic!("no provider for {other}"),
            };
            let cancel = CancelToken::new();
            for i in 1..=repeat {
                let r = provider
                    .transcribe(
                        &TranscribeRequest {
                            provider_id: provider_id.clone(),
                            model_id: id.clone(),
                            language: lang.clone(),
                            audio_path: wav.clone(),
                        },
                        &cancel,
                    )
                    .expect("transcription failed");
                println!(
                    "run {i}: {} cold={} load={}ms infer={}ms audio={}ms rtf={:.3} threads={} decoding={}\n  text: {}",
                    r.provider_id,
                    r.cold_start,
                    r.load_ms,
                    r.processing_ms,
                    r.audio_ms.unwrap_or(0),
                    r.rtf.unwrap_or(0.0),
                    r.threads,
                    r.decoding,
                    r.text
                );
            }
        }
        _ => eprintln!("usage: list | install <id> | run <id> <fr|en> <wav> [repeat]"),
    }
}
