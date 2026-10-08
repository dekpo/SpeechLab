//! whisper.cpp speech-to-text adapter. The only place that knows whisper.cpp.
//!
//! whisper.cpp runs as a separate native process (`whisper-cli`), built from the official
//! source by `scripts/build-whisper-cpp.ps1`. Reasons: no libclang/bindgen needed, crash
//! isolation, and real cancellation (the process is killed). Cost: every run is a cold start
//! (the model is loaded again); `loadMs` reports whisper.cpp's own load time.
//!
//! Inference is fully local. This module never touches the network (see `download.rs`).

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::error::SpeechError;
use super::models::{ModelFamily, ModelManager};
use super::probe::ProcessProbe;
use super::provider::{CancelToken, SpeechToTextProvider};
use super::types::{Capabilities, ProviderInfo, ProviderKind, TranscribeRequest, TranscribeResult};
use super::wav;

pub const WHISPER_CPP_ID: &str = "whisper-cpp";
const DEFAULT_THREADS: u32 = 4;
/// whisper-cli's own default is 5 beams; sherpa-onnx Whisper uses greedy search. This
/// difference is a known confound when comparing engines, so it is reported in the result.
const DEFAULT_BEAM_SIZE: u32 = 5;

pub struct WhisperCppProvider {
    models: Arc<ModelManager>,
    cli_override: Option<PathBuf>,
    threads: u32,
    beam_size: u32,
}

impl WhisperCppProvider {
    pub fn new(models: Arc<ModelManager>) -> Self {
        let beam_size = std::env::var("SPEECHLAB_WHISPER_BEAM_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|b| *b >= 1)
            .unwrap_or(DEFAULT_BEAM_SIZE);
        Self {
            models,
            cli_override: None,
            threads: DEFAULT_THREADS,
            beam_size,
        }
    }

    pub fn with_cli(mut self, cli: PathBuf) -> Self {
        self.cli_override = Some(cli);
        self
    }

    pub fn with_beam_size(mut self, beam_size: u32) -> Self {
        self.beam_size = beam_size.max(1);
        self
    }

    fn decoding_label(&self) -> String {
        if self.beam_size == 1 {
            "greedy (beam 1)".into()
        } else {
            format!("beam search ({} beams)", self.beam_size)
        }
    }

    fn cli_path(&self) -> Result<PathBuf, SpeechError> {
        locate_cli(self.cli_override.as_deref()).ok_or_else(|| {
            SpeechError::Engine(
                "whisper-cli not found. Build it with scripts/build-whisper-cpp.ps1 or set SPEECHLAB_WHISPER_CLI"
                    .into(),
            )
        })
    }
}

/// Resolution order: explicit override, SPEECHLAB_WHISPER_CLI, next to the app executable
/// (future sidecar location), then the dev build in vendor/.
pub fn locate_cli(explicit: Option<&Path>) -> Option<PathBuf> {
    let exe = if cfg!(windows) { "whisper-cli.exe" } else { "whisper-cli" };
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = explicit {
        candidates.push(p.to_path_buf());
    }
    if let Some(p) = std::env::var_os("SPEECHLAB_WHISPER_CLI") {
        candidates.push(PathBuf::from(p));
    }
    if let Ok(me) = std::env::current_exe() {
        if let Some(dir) = me.parent() {
            candidates.push(dir.join(exe));
        }
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("vendor")
            .join("whisper.cpp")
            .join("build")
            .join("bin")
            .join(exe),
    );
    candidates.into_iter().find(|p| p.is_file())
}

#[derive(Debug, PartialEq)]
pub struct WhisperTimings {
    pub load_ms: f64,
    pub total_ms: f64,
}

/// Extracts "load time" and "total time" from whisper.cpp's stderr timing report.
pub fn parse_timings(stderr: &str) -> Option<WhisperTimings> {
    let find = |label: &str| -> Option<f64> {
        stderr
            .lines()
            .find(|l| l.contains("whisper_print_timings") && l.contains(label))
            .and_then(|l| l.split('=').nth(1))
            .and_then(|v| v.trim().split_whitespace().next())
            .and_then(|v| v.parse().ok())
    };
    Some(WhisperTimings {
        load_ms: find("load time")?,
        total_ms: find("total time")?,
    })
}

pub struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    /// Peak resident memory of the child, sampled every ~25 ms (a lower bound), in MB.
    pub peak_memory_mb: Option<f64>,
    /// CPU time of the child (all threads), in ms.
    pub cpu_ms: Option<u64>,
}

/// Runs a command, polling the cancel token; the child is killed on cancellation.
pub fn run_cancellable(mut cmd: Command, cancel: &CancelToken) -> Result<ProcessOutput, SpeechError> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| SpeechError::Engine(format!("cannot start process: {e}")))?;

    fn drain(mut pipe: impl Read + Send + 'static) -> thread::JoinHandle<String> {
        thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = pipe.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).into_owned()
        })
    }
    let out = drain(child.stdout.take().expect("piped stdout"));
    let err = drain(child.stderr.take().expect("piped stderr"));

    let mut probe = ProcessProbe::new(child.id());
    let status = loop {
        probe.sample();
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(SpeechError::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(e) => return Err(SpeechError::Engine(format!("wait failed: {e}"))),
        }
    };
    Ok(ProcessOutput {
        stdout: out.join().unwrap_or_default(),
        stderr: err.join().unwrap_or_default(),
        success: status.success(),
        peak_memory_mb: probe.peak_mb(),
        cpu_ms: probe.cpu_ms(),
    })
}

/// The initial prompt built from a vocabulary: the terms as a plain comma-separated list. Whisper
/// reads the prompt as text that came just before the audio, so a list of the spellings it should
/// use nudges the output toward them (D-036). None when the vocabulary is empty.
pub fn vocabulary_prompt(vocabulary: &[String]) -> Option<String> {
    let terms: Vec<&str> = vocabulary.iter().map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
    (!terms.is_empty()).then(|| format!("{}.", terms.join(", ")))
}

/// whisper-cli prints one transcript line per segment on stdout (`-nt`).
pub fn clean_transcript(stdout: &str) -> String {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

impl SpeechToTextProvider for WhisperCppProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: WHISPER_CPP_ID.into(),
            display_name: "whisper.cpp (CPU, external process)".into(),
            kind: ProviderKind::Stt,
            is_mock: false,
            languages: vec!["fr".into(), "en".into()],
            capabilities: Capabilities {
                supports_cancellation: true,
                supports_language_auto_detect: true,
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
        if def.provider != WHISPER_CPP_ID || def.family != ModelFamily::GgmlWhisper {
            return Err(SpeechError::InvalidRequest(format!(
                "model {} does not belong to {WHISPER_CPP_ID}",
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

        // Decode ourselves first: same validation and error messages as the other provider,
        // and the audio duration used for the RTF.
        let audio = wav::read_wav_mono(Path::new(&request.audio_path))?;
        if cancel.is_cancelled() {
            return Err(SpeechError::Cancelled);
        }

        let model_file = self
            .models
            .model_dir(def)
            .join(def.files.get(&def.files.model, "model")?);
        let mut cmd = Command::new(self.cli_path()?);
        cmd.arg("-m")
            .arg(&model_file)
            .arg("-f")
            .arg(&request.audio_path)
            .arg("-l")
            .arg(&request.language)
            .arg("-t")
            .arg(self.threads.to_string())
            .arg("-bs")
            .arg(self.beam_size.to_string())
            .arg("-bo")
            .arg(self.beam_size.to_string())
            .arg("-nt");
        let prompt = vocabulary_prompt(&request.vocabulary);
        if let Some(prompt) = &prompt {
            cmd.arg("--prompt").arg(prompt);
        }

        let wall = Instant::now();
        let output = run_cancellable(cmd, cancel)?;
        let wall_ms = wall.elapsed().as_millis() as u64;
        if !output.success {
            let tail: String = output.stderr.lines().rev().take(5).collect::<Vec<_>>().join(" | ");
            return Err(SpeechError::Engine(format!("whisper-cli failed: {tail}")));
        }

        // Prefer whisper.cpp's own numbers; fall back to wall-clock if the report is missing.
        let (load_ms, processing_ms) = match parse_timings(&output.stderr) {
            Some(t) => (t.load_ms.round() as u64, (t.total_ms - t.load_ms).max(0.0).round() as u64),
            None => (0, wall_ms),
        };
        let audio_ms = audio.duration_ms();
        Ok(TranscribeResult {
            provider_id: WHISPER_CPP_ID.into(),
            model_id: def.id.clone(),
            language: request.language.clone(),
            text: clean_transcript(&output.stdout),
            processing_ms,
            load_ms,
            cold_start: true,
            audio_ms: Some(audio_ms),
            rtf: (audio_ms > 0).then(|| processing_ms as f64 / audio_ms as f64),
            threads: self.threads,
            peak_memory_mb: output.peak_memory_mb,
            cpu_ms: output.cpu_ms,
            decoding: match prompt {
                Some(_) => format!("{} + initial prompt", self.decoding_label()),
                None => self.decoding_label(),
            },
            is_mock: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_STDERR: &str = "\
whisper_print_timings:     load time =    87.92 ms
whisper_print_timings:     fallbacks =   0 p /   0 h
whisper_print_timings:      mel time =     9.44 ms
whisper_print_timings:   encode time =   428.41 ms /     1 runs (   428.41 ms per run)
whisper_print_timings:    total time =   822.42 ms
";

    #[test]
    fn parses_load_and_total_time() {
        assert_eq!(
            parse_timings(SAMPLE_STDERR),
            Some(WhisperTimings { load_ms: 87.92, total_ms: 822.42 })
        );
        assert_eq!(parse_timings("nothing useful"), None);
    }

    #[test]
    fn transcript_lines_are_trimmed_and_joined() {
        assert_eq!(clean_transcript("\n Bonjour.\n  Salut.\n\n"), "Bonjour. Salut.");
        assert_eq!(clean_transcript(""), "");
    }

    #[test]
    fn vocabulary_becomes_a_comma_separated_prompt() {
        assert_eq!(vocabulary_prompt(&[]), None);
        assert_eq!(vocabulary_prompt(&["  ".into()]), None);
        let v = vec!["amoxicilline".to_string(), " ibuprofène ".to_string()];
        assert_eq!(vocabulary_prompt(&v).as_deref(), Some("amoxicilline, ibuprofène."));
    }

    #[test]
    fn decoding_label_distinguishes_greedy_and_beam() {
        let m = Arc::new(ModelManager::new(std::env::temp_dir().join("speechlab_wc")).unwrap());
        assert_eq!(WhisperCppProvider::new(Arc::clone(&m)).with_beam_size(1).decoding_label(), "greedy (beam 1)");
        assert_eq!(WhisperCppProvider::new(m).with_beam_size(5).decoding_label(), "beam search (5 beams)");
    }

    #[test]
    fn missing_cli_gives_actionable_error() {
        let m = Arc::new(ModelManager::new(std::env::temp_dir().join("speechlab_wc")).unwrap());
        let p = WhisperCppProvider::new(m).with_cli(PathBuf::from("Z:/definitely/missing/whisper-cli.exe"));
        // The explicit path is skipped when absent; the result depends on the dev build existing,
        // so only assert that an error, if any, is the actionable one.
        if let Err(SpeechError::Engine(msg)) = p.cli_path() {
            assert!(msg.contains("build-whisper-cpp.ps1"));
        }
    }

    #[cfg(windows)]
    #[test]
    fn cancellation_kills_a_long_running_process() {
        let cancel = CancelToken::new();
        let c2 = cancel.clone();
        let t = thread::spawn(move || {
            thread::sleep(Duration::from_millis(300));
            c2.cancel();
        });
        let mut cmd = Command::new("ping");
        cmd.args(["-n", "30", "127.0.0.1"]);
        let start = Instant::now();
        let r = run_cancellable(cmd, &cancel);
        t.join().unwrap();
        assert_eq!(r.err(), Some(SpeechError::Cancelled));
        assert!(start.elapsed() < Duration::from_secs(5), "process was not killed promptly");
    }
}
