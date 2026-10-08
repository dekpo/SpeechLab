//! Text-to-speech command line tool (M6).
//!
//!   cargo run --release --example tts -- voices
//!   cargo run --release --example tts -- say <voice-id> <fr|en> "<text>" | @file.txt [--speed 1.0] [--repeat N] [--out-dir DIR]
//!   cargo run --release --example tts -- measure --sentences <file.txt> --lang <fr|en> --voices a,b [--reps 3] [--speed 1.0] [--out-dir DIR] [--label x]
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

fn request(voice: &str, lang: &str, text: &str, speed: f32) -> SynthesizeRequest {
    SynthesizeRequest {
        provider_id: "sherpa-onnx-tts".into(),
        voice_id: voice.into(),
        language: lang.into(),
        text: text.into(),
        speed,
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
            let p = provider(out_dir(&args));
            for i in 0..args.num("repeat", 1usize) {
                let r = p.synthesize(&request(voice, lang, text, speed), &CancelToken::new()).unwrap_or_else(|e| panic!("{e}"));
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
    let mut md = String::from("| Voice | Sentence | Chars | Audio ms | Gen ms (median of runs) | RTF (median) | Cold load ms |\n|---|---|---|---|---|---|---|\n");
    for voice in &voices {
        for (i, s) in sentences.iter().enumerate() {
            let (mut gens, mut rtfs, mut audio, mut load) = (Vec::new(), Vec::new(), 0, 0);
            for rep in 0..reps {
                let r = p.synthesize(&request(voice, &lang, s, speed), &CancelToken::new()).unwrap_or_else(|e| panic!("{voice}: {e}"));
                jsonl.push_str(&format!(
                    "{}\n",
                    serde_json::json!({"voice": voice, "lang": lang, "sentence": i + 1, "chars": s.chars().count(), "rep": rep + 1, "speed": speed,
                        "generationMs": r.generation_ms, "audioMs": r.audio_ms, "rtf": r.rtf, "loadMs": r.load_ms, "cold": r.cold_start,
                        "sampleRate": r.sample_rate, "peakMemoryMb": r.peak_memory_mb})
                ));
                if !r.cold_start || rep > 0 {
                    gens.push(r.generation_ms as f64);
                    rtfs.push(r.rtf.unwrap_or(0.0));
                }
                audio = r.audio_ms;
                load = load.max(r.load_ms);
                let _ = std::fs::remove_file(&r.wav_path);
            }
            if gens.is_empty() {
                gens.push(0.0);
                rtfs.push(0.0);
            }
            md.push_str(&format!("| {voice} | {} | {} | {audio} | {:.0} | {:.3} | {load} |\n", i + 1, s.chars().count(), median(&mut gens), median(&mut rtfs)));
        }
    }
    std::fs::write(dir.join("tts-measure.jsonl"), jsonl).expect("write jsonl");
    std::fs::write(dir.join("tts-measure.md"), &md).expect("write md");
    println!("{md}\nwritten to {}", dir.display());
}
