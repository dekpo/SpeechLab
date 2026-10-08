//! Text-to-speech command line tool (M6).
//!
//!   cargo run --release --example tts -- voices
//!   cargo run --release --example tts -- say <voice-id> <fr|en> "<text>" | @file.txt [--speed 1.0] [--repeat N] [--out-dir DIR]
//!       (extra: --normalise on|off  --whole 1 (one library call for the whole text)  --show-text 1  --show-segments 1; every option takes a value)
//!   cargo run --release --example tts -- measure --sentences <file.txt> --lang <fr|en> --voices a,b [--reps 3] [--speed 1.0] [--out-dir DIR] [--label x] [--compare 1] [--normalise on|off]
//!
//! `say` prints generation time, audio duration and RTF for each repetition (the first one is cold) and
//! the path of the WAV file; it never plays anything. `measure` runs every sentence of the file (one per
//! line, `#` comments) with every voice, repeated, in one process, and writes `tts-measure.jsonl` and
//! `tts-measure.md` into the output folder. Use --release for speed figures and run `bench preflight` first.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use speechlab_lib::speech::models::ModelManager;
use speechlab_lib::speech::provider::{CancelToken, TextToSpeechProvider};
use speechlab_lib::speech::tts::SherpaTtsProvider;
use speechlab_lib::speech::types::SynthesizeRequest;

fn models_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("SPEECHLAB_MODELS_DIR") {
        return PathBuf::from(p);
    }
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .expect("APPDATA or HOME must be set");
    base.join("ai.assistantcabinet.speechlab").join("models")
}

struct Args {
    positional: Vec<String>,
    values: HashMap<String, String>,
}

impl Args {
    fn parse(raw: &[String]) -> Args {
        let (mut positional, mut values) = (Vec::new(), HashMap::new());
        let mut i = 0;
        while i < raw.len() {
            if let Some(k) = raw[i].strip_prefix("--") {
                values.insert(k.to_string(), raw.get(i + 1).cloned().unwrap_or_default());
                i += 1;
            } else {
                positional.push(raw[i].clone());
            }
            i += 1;
        }
        Args { positional, values }
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.values.get(k).map(String::as_str)
    }
    fn num<T: std::str::FromStr>(&self, k: &str, default: T) -> T {
        self.get(k).and_then(|v| v.parse().ok()).unwrap_or(default)
    }
}

fn provider(out_dir: PathBuf) -> SherpaTtsProvider {
    let models = Arc::new(ModelManager::new(models_dir()).expect("manifest"));
    SherpaTtsProvider::new(models, out_dir)
}

fn out_dir(args: &Args) -> PathBuf {
    args.get("out-dir").map(PathBuf::from).unwrap_or_else(|| models_dir().with_file_name("tts-output"))
}

fn request(voice: &str, lang: &str, text: &str, speed: f32, normalise: Option<bool>) -> SynthesizeRequest {
    SynthesizeRequest {
        provider_id: "sherpa-onnx-tts".into(),
        voice_id: voice.into(),
        language: lang.into(),
        text: text.into(),
        speed,
        normalise,
    }
}

/// `--normalise on|off` forces the text rewriting; absent = the voice package's default.
fn normalise_flag(args: &Args) -> Option<bool> {
    match args.get("normalise") {
        Some("on") => Some(true),
        Some("off") => Some(false),
        _ => None,
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(raw.get(1..).unwrap_or(&[]));
    match raw.first().map(String::as_str) {
        Some("voices") => {
            let p = provider(out_dir(&args));
            for v in p.voices().expect("voices") {
                println!("{:48} {:3} {:8} {}", v.id, v.language, v.gender, v.display_name);
            }
        }
        Some("say") => {
            let (voice, lang, text) = (
                args.positional.first().expect("voice id"),
                args.positional.get(1).expect("language"),
                args.positional.get(2).expect("text"),
            );
            // "@file.txt" reads the text from a UTF-8 file (safe for accents on any console).
            let text = &match text.strip_prefix('@') {
                Some(file) => std::fs::read_to_string(file).unwrap_or_else(|e| panic!("cannot read {file}: {e}")).trim().to_string(),
                None => text.to_string(),
            };
            let speed: f32 = args.num("speed", 1.0);
            let normalise = normalise_flag(&args);
            let whole = args.get("whole").is_some();
            let p = provider(out_dir(&args));
            if args.get("show-text").is_some() {
                let on = normalise.unwrap_or(true);
                for s in speechlab_lib::speech::tts::plan_sentences(text, lang, on) {
                    println!("[{}..{}] {} => {}", s.start, s.end, s.text, s.spoken);
                }
            }
            for i in 0..args.num("repeat", 1usize) {
                let req = request(voice, lang, text, speed, normalise);
                let r = if whole { p.synthesize_whole_text(&req, &CancelToken::new()) } else { p.synthesize(&req, &CancelToken::new()) }
                    .unwrap_or_else(|e| panic!("{e}"));
                if args.get("show-segments").is_some() {
                    for (n, g) in r.segments.iter().enumerate() {
                        println!("  segment {}: text {}..{}  audio {}..{} ms", n + 1, g.start, g.end, g.start_ms, g.end_ms);
                    }
                }
                println!(
                    "run {}: generation {} ms, audio {} ms, RTF {:.3}, load {} ms{}, {} Hz, peak memory {} MB -> {}",
                    i + 1,
                    r.generation_ms,
                    r.audio_ms,
                    r.rtf.unwrap_or(0.0),
                    r.load_ms,
                    if r.cold_start { " (cold)" } else { "" },
                    r.sample_rate,
                    r.peak_memory_mb.map(|m| format!("{m:.0}")).unwrap_or_else(|| "?".into()),
                    r.wav_path
                );
            }
        }
        Some("measure") => measure(&args),
        _ => eprintln!("usage: voices | say <voice-id> <fr|en> \"<text>\" [--speed S] [--repeat N] | measure --sentences F --lang L --voices a,b [--reps N]"),
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn measure(args: &Args) {
    let file = PathBuf::from(args.get("sentences").expect("--sentences <file>"));
    let lang = args.get("lang").expect("--lang fr|en").to_string();
    let voices: Vec<String> = args.get("voices").expect("--voices a,b").split(',').map(|s| s.trim().to_string()).collect();
    let reps: usize = args.num("reps", 3);
    let speed: f32 = args.num("speed", 1.0);
    let normalise = normalise_flag(args);
    // --whole 1: only the one-call-per-text mode; --compare 1: both modes, interleaved inside each repetition
    // (the per-sentence overhead measured in one run, I-042); default: per-sentence only.
    let modes: Vec<bool> = if args.get("compare").is_some() { vec![false, true] } else { vec![args.get("whole").is_some()] };
    let sentences: Vec<String> = std::fs::read_to_string(&file)
        .expect("sentences file")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    let dir = out_dir(args).join(format!("measure-{}", args.get("label").unwrap_or("run")));
    let p = provider(dir.join("wav"));
    std::fs::create_dir_all(&dir).expect("out dir");
    let mut jsonl = String::new();
    let mut md = String::from("| Voice | Mode | Sentence | Chars | Sentences | Audio ms | Gen ms (median of warm runs) | RTF (median) | Cold load ms |\n|---|---|---|---|---|---|---|---|---|\n");
    for voice in &voices {
        for (i, s) in sentences.iter().enumerate() {
            // Per mode: generation times, RTFs, last audio length, load, segment count.
            let mut acc: Vec<(Vec<f64>, Vec<f64>, u64, u64, usize)> = modes.iter().map(|_| (Vec::new(), Vec::new(), 0, 0, 0)).collect();
            for rep in 0..reps {
                for (m, &whole) in modes.iter().enumerate() {
                    let req = request(voice, &lang, s, speed, normalise);
                    let r = if whole { p.synthesize_whole_text(&req, &CancelToken::new()) } else { p.synthesize(&req, &CancelToken::new()) }
                        .unwrap_or_else(|e| panic!("{voice}: {e}"));
                    jsonl.push_str(&format!(
                        "{}\n",
                        serde_json::json!({"voice": voice, "lang": lang, "mode": if whole { "whole-text" } else { "per-sentence" }, "sentence": i + 1, "chars": s.chars().count(),
                            "rep": rep + 1, "speed": speed, "normalised": r.normalised, "segments": r.segments.len(),
                            "generationMs": r.generation_ms, "audioMs": r.audio_ms, "rtf": r.rtf, "loadMs": r.load_ms, "cold": r.cold_start,
                            "sampleRate": r.sample_rate, "peakMemoryMb": r.peak_memory_mb})
                    ));
                    let a = &mut acc[m];
                    if !r.cold_start || rep > 0 {
                        a.0.push(r.generation_ms as f64);
                        a.1.push(r.rtf.unwrap_or(0.0));
                    }
                    a.2 = r.audio_ms;
                    a.3 = a.3.max(r.load_ms);
                    a.4 = r.segments.len();
                    let _ = std::fs::remove_file(&r.wav_path);
                }
            }
            for (m, &whole) in modes.iter().enumerate() {
                let a = &mut acc[m];
                if a.0.is_empty() {
                    a.0.push(0.0);
                    a.1.push(0.0);
                }
                md.push_str(&format!(
                    "| {voice} | {} | {} | {} | {} | {} | {:.0} | {:.3} | {} |\n",
                    if whole { "whole-text" } else { "per-sentence" },
                    i + 1,
                    s.chars().count(),
                    if whole { "-".to_string() } else { a.4.to_string() },
                    a.2,
                    median(&mut a.0),
                    median(&mut a.1),
                    a.3
                ));
            }
        }
    }
    std::fs::write(dir.join("tts-measure.jsonl"), jsonl).expect("write jsonl");
    std::fs::write(dir.join("tts-measure.md"), &md).expect("write md");
    println!("{md}\nwritten to {}", dir.display());
}
