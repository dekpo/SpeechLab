//! Reproducible benchmark runner (library part; the CLI is `examples/bench.rs`).
//!
//! One run = one model (+ decoding setting) over the selected samples, repeated N times.
//! Every run is recorded with its timings, memory, CPU time, text, WER/CER and critical flags.
//! Nothing here fabricates data: if a run fails, the record carries the error and no metrics.
//!
//! Reading the numbers:
//! - Accuracy (WER/CER/critical flags) uses repetition 1; later repetitions are only used to
//!   check that the output is stable and to measure timing.
//! - `load_ms` is the model loading time of that run (cold start); `inference_ms` excludes it.
//! - Memory is sampled (a lower bound); for in-process engines the benchmark CLI runs one
//!   process per model so the figure belongs to that model alone.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sysinfo::System;

use super::chunking::{self, ChunkConfig};
use super::critical::{self, CriticalFlag};
use super::dataset::{Dataset, Sample};
use super::error::SpeechError;
use super::metrics;
use super::models::ModelManager;
use super::provider::{CancelToken, SpeechToTextProvider};
use super::sherpa::SherpaOnnxProvider;
use super::types::TranscribeRequest;
use super::vad;
use super::wav;
use super::whisper_cpp::WhisperCppProvider;

// ---------------------------------------------------------------- system information

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub cpu: String,
    pub physical_cores: Option<usize>,
    pub logical_cores: usize,
    pub total_ram_gb: f64,
    pub os: String,
    pub arch: String,
    /// Debug builds are slower: the engines are native, but record it anyway.
    pub rust_build: String,
    pub accelerator: String,
    /// Active Windows power plan as reported by `powercfg` (raw text, may be localised).
    #[serde(default)]
    pub power_plan: Option<String>,
    /// Some(true) on mains power, Some(false) on battery, None when unknown or no battery.
    #[serde(default)]
    pub on_ac_power: Option<bool>,
}

pub fn system_info() -> SystemInfo {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();
    SystemInfo {
        cpu: sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default(),
        physical_cores: System::physical_core_count(),
        logical_cores: sys.cpus().len(),
        total_ram_gb: (sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0,
        os: System::long_os_version().unwrap_or_else(|| "unknown".into()),
        arch: System::cpu_arch(),
        rust_build: if cfg!(debug_assertions) { "debug".into() } else { "release".into() },
        accelerator: "none (CPU only)".into(),
        power_plan: power_plan(),
        on_ac_power: on_ac_power(),
    }
}

#[cfg(windows)]
fn run_text(program: &str, args: &[&str]) -> Option<String> {
    use std::os::windows::process::CommandExt;
    let out = std::process::Command::new(program)
        .args(args)
        .creation_flags(0x0800_0000)
        .output()
        .ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(windows)]
fn power_plan() -> Option<String> {
    run_text("powercfg", &["/getactivescheme"]).filter(|s| !s.is_empty())
}

#[cfg(windows)]
fn on_ac_power() -> Option<bool> {
    // Win32_Battery.BatteryStatus: 2 = on AC power; 1, 4, 5... = discharging or other.
    let out = run_text(
        "powershell",
        &["-NoProfile", "-Command", "(Get-CimInstance Win32_Battery | Select-Object -First 1).BatteryStatus"],
    )?;
    match out.trim() {
        "" => None,
        "2" => Some(true),
        _ => Some(false),
    }
}

#[cfg(not(windows))]
fn power_plan() -> Option<String> {
    None
}

#[cfg(not(windows))]
fn on_ac_power() -> Option<bool> {
    None
}

/// Result of the quiet-machine check done before a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preflight {
    /// Average CPU use of the whole machine over the observation window, in percent.
    pub idle_cpu_percent: f64,
    pub window_ms: u64,
    /// The busiest processes during the window: (name, CPU percent of one core).
    pub top_processes: Vec<(String, f32)>,
    pub on_ac_power: Option<bool>,
    pub problems: Vec<String>,
}

/// Observes the machine for `window_ms` and reports whether it is quiet enough for timing figures:
/// global CPU use under `max_cpu_percent` and, when known, mains power.
pub fn preflight(window_ms: u64, max_cpu_percent: f64) -> Preflight {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());
    let steps = (window_ms / 500).max(2);
    let mut samples = Vec::new();
    for _ in 0..steps {
        std::thread::sleep(std::time::Duration::from_millis(500));
        sys.refresh_cpu_all();
        samples.push(f64::from(sys.global_cpu_usage()));
    }
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());
    let mut procs: Vec<(String, f32)> = sys
        .processes()
        .values()
        .map(|p| (p.name().to_string_lossy().into_owned(), p.cpu_usage()))
        .filter(|(_, c)| *c > 1.0)
        .collect();
    procs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(5);
    let idle = samples.iter().sum::<f64>() / samples.len().max(1) as f64;
    let ac = on_ac_power();
    let mut problems = Vec::new();
    if idle > max_cpu_percent {
        problems.push(format!("the machine is busy: {idle:.0} % CPU on average (limit {max_cpu_percent:.0} %)"));
    }
    if ac == Some(false) {
        problems.push("running on battery: plug in the charger".into());
    }
    Preflight { idle_cpu_percent: idle, window_ms, top_processes: procs, on_ac_power: ac, problems }
}

// ---------------------------------------------------------------- audio quality

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioQuality {
    pub duration_ms: u64,
    pub peak: f64,
    pub rms: f64,
    /// Share of samples at or above 99 % of full scale.
    pub clipped_ratio: f64,
    /// Share of samples at or above 90 % of full scale (a "hot" signal).
    pub hot_ratio: f64,
    pub leading_silence_ms: u64,
    pub trailing_silence_ms: u64,
    pub words_per_second: Option<f64>,
    pub warnings: Vec<String>,
}

pub fn analyze_audio(path: &Path, reference: &str) -> Result<AudioQuality, SpeechError> {
    let audio = wav::read_wav_mono(path)?;
    let n = audio.samples.len();
    let rate = audio.sample_rate as usize;
    let (mut peak, mut sum_sq, mut clipped, mut hot) = (0f64, 0f64, 0usize, 0usize);
    for &s in &audio.samples {
        let a = f64::from(s).abs();
        peak = peak.max(a);
        sum_sq += a * a;
        clipped += usize::from(a >= 0.99);
        hot += usize::from(a >= 0.9);
    }
    let frame = (rate / 50).max(1); // 20 ms
    let silent = |start: usize| -> bool {
        let end = (start + frame).min(n);
        let seg = &audio.samples[start..end];
        let e = seg.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / seg.len().max(1) as f64;
        e.sqrt() < 0.01
    };
    let mut lead = 0;
    while lead * frame < n && silent(lead * frame) {
        lead += 1;
    }
    let mut trail = 0;
    while (trail + 1) * frame <= n && silent(n - (trail + 1) * frame) {
        trail += 1;
    }
    let duration_ms = audio.duration_ms();
    let words = metrics::normalize_words(reference).len();
    let wps = (duration_ms > 0 && words > 0).then(|| words as f64 / (duration_ms as f64 / 1000.0));
    let ms = |frames: usize| (frames * frame * 1000 / rate.max(1)) as u64;

    let q = AudioQuality {
        duration_ms,
        peak,
        rms: (sum_sq / n.max(1) as f64).sqrt(),
        clipped_ratio: clipped as f64 / n.max(1) as f64,
        hot_ratio: hot as f64 / n.max(1) as f64,
        leading_silence_ms: ms(lead),
        trailing_silence_ms: ms(trail),
        words_per_second: wps,
        warnings: Vec::new(),
    };
    let mut warnings = Vec::new();
    if q.clipped_ratio > 0.001 {
        warnings.push(format!("clipping: {:.2} % of samples at full scale", q.clipped_ratio * 100.0));
    }
    if q.hot_ratio > 0.02 {
        warnings.push(format!("hot signal: {:.1} % of samples above 90 % of full scale", q.hot_ratio * 100.0));
    }
    if q.peak < 0.1 {
        warnings.push(format!("very quiet: peak {:.0} %", q.peak * 100.0));
    }
    if q.leading_silence_ms > 1500 || q.trailing_silence_ms > 2000 {
        warnings.push("long silence at the start or the end".into());
    }
    if let Some(w) = q.words_per_second {
        if !(1.2..=5.0).contains(&w) {
            warnings.push(format!("unusual reading speed: {w:.1} words/s (wrong sentence read?)"));
        }
    }
    Ok(AudioQuality { warnings, ..q })
}

// ---------------------------------------------------------------- runs

/// Version of the scoring rules (normalisation + critical detector). Bump it whenever scoring
/// changes; `rescore` upgrades old result files without re-running any engine.
/// 1 = first rules, 2 = number grammar, digit/letter splitting, spoken decimals, 17-19, "h" unit.
pub const SCORING_VERSION: u32 = 2;

fn scoring_v1() -> u32 {
    1
}

fn chunking_off() -> String {
    "off".into()
}

fn one_segment() -> u32 {
    1
}

fn biasing_none() -> String {
    "none".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSpec {
    pub model_id: String,
    /// whisper.cpp only: 1 = greedy, 5 = beam search with 5 beams. None for other engines.
    pub beam_size: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub repetitions: u32,
    /// Empty = every category.
    pub categories: Vec<String>,
    pub limit: Option<usize>,
    pub include_private: bool,
    /// Cut clips longer than one segment at silences (Silero VAD) before the engine (D-035).
    pub chunking: bool,
    /// Longest segment in seconds when chunking (None = the default of `ChunkConfig`, 25 s).
    pub chunk_max_s: Option<f32>,
    /// Vocabulary to bias the engine with, by language code (T4, D-036). Empty = no biasing.
    pub vocabulary: std::collections::BTreeMap<String, Vec<String>>,
    /// sherpa-onnx NeMo transducer: use modified beam search with this hotwords score (the
    /// vocabulary above is then boosted). Some without a vocabulary is the beam-search control.
    pub hotwords_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    pub run_id: String,
    /// Which scoring rules produced wer/cer/critical (see `SCORING_VERSION`).
    #[serde(default = "scoring_v1")]
    pub scoring_version: u32,
    pub timestamp_ms: u64,
    pub model_id: String,
    pub provider: String,
    pub decoding: String,
    pub sample_id: String,
    pub speaker_id: String,
    pub category: String,
    pub language: String,
    pub domain: String,
    pub utterance_type: String,
    pub private: bool,
    pub repetition: u32,
    pub cold_start: bool,
    pub load_ms: u64,
    /// Sum over the segments when chunking was used; chunking itself is in `chunking_ms`.
    pub inference_ms: u64,
    pub audio_ms: u64,
    pub rtf: Option<f64>,
    /// "off" (whole clip) or "vad" (cut at silences when longer than one segment).
    #[serde(default = "chunking_off")]
    pub chunking: String,
    /// Number of pieces sent to the engine; 1 for the whole clip.
    #[serde(default = "one_segment")]
    pub segments: u32,
    /// Speech detection, planning and writing the pieces; not part of `inference_ms`.
    #[serde(default)]
    pub chunking_ms: u64,
    /// How the engine was biased toward a vocabulary: "none", or the engine's own description
    /// (for example "initial prompt"), or "ignored by engine" (D-036).
    #[serde(default = "biasing_none")]
    pub biasing: String,
    /// Post-correction applied after the engine ("none" or the preset name, D-036). When it is
    /// not "none", `text` is the corrected text and `raw_text` the engine's own output.
    #[serde(default = "biasing_none")]
    pub post_correction: String,
    #[serde(default)]
    pub raw_text: Option<String>,
    pub threads: u32,
    pub peak_memory_mb: Option<f64>,
    pub cpu_ms: Option<u64>,
    /// CPU time / (load + inference): average number of cores kept busy.
    pub avg_cores: Option<f64>,
    pub reference: String,
    pub text: String,
    pub wer: Option<f64>,
    pub cer: Option<f64>,
    pub substitutions: usize,
    pub deletions: usize,
    pub insertions: usize,
    pub reference_words: usize,
    pub critical: Vec<CriticalFlag>,
    pub has_critical: bool,
    pub error: Option<String>,
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// "20261007-153042" in UTC, without extra dependencies.
pub fn utc_stamp(unix_ms: u64) -> String {
    let secs = (unix_ms / 1000) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Civil-from-days (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, (rem % 3600) / 60, rem % 60)
}

/// Recomputes WER/CER and critical flags of stored runs with the CURRENT scoring rules, from the
/// stored transcripts. Key terms come from the dataset (matched by sample id). No engine is run.
pub fn rescore(records: &mut [RunRecord], key_terms: &BTreeMap<String, Vec<critical::KeyTerm>>) {
    for r in records.iter_mut().filter(|r| r.error.is_none()) {
        let cmp = metrics::compare_texts(&r.reference, &r.text);
        let terms = key_terms.get(&r.sample_id).map(Vec::as_slice).unwrap_or(&[]);
        let report = critical::analyze(&r.reference, &r.text, terms);
        r.wer = cmp.wer;
        r.cer = cmp.cer;
        r.substitutions = cmp.substitutions;
        r.deletions = cmp.deletions;
        r.insertions = cmp.insertions;
        r.reference_words = cmp.reference_words;
        r.has_critical = report.has_critical;
        r.critical = report.flags;
        r.scoring_version = SCORING_VERSION;
    }
}

pub fn select_samples<'a>(ds: &'a Dataset, opts: &RunOptions) -> Vec<&'a Sample> {
    let mut v: Vec<&Sample> = ds
        .runnable()
        .into_iter()
        .filter(|s| opts.include_private || !s.private)
        .filter(|s| opts.categories.is_empty() || opts.categories.contains(&s.category))
        .collect();
    if let Some(n) = opts.limit {
        v.truncate(n);
    }
    v
}

fn build_provider(
    models: &Arc<ModelManager>,
    spec: &RunSpec,
    opts: &RunOptions,
) -> Result<(Box<dyn SpeechToTextProvider>, String, String), SpeechError> {
    let def = models.def(&spec.model_id)?;
    match def.provider.as_str() {
        "sherpa-onnx" => {
            let mut provider = SherpaOnnxProvider::new(Arc::clone(models));
            if let Some(score) = opts.hotwords_score {
                provider = provider.with_hotwords(score);
            }
            // The label is the engine's base setting, shared by all variants of the study so the
            // records can be paired; the technique itself is recorded in `biasing`.
            Ok((Box::new(provider), def.provider.clone(), "greedy search".into()))
        }
        "whisper-cpp" => {
            let beams = spec.beam_size.unwrap_or(5).max(1);
            Ok((
                Box::new(WhisperCppProvider::new(Arc::clone(models)).with_beam_size(beams)),
                def.provider.clone(),
                if beams == 1 { "greedy (beam 1)".into() } else { format!("beam search ({beams} beams)") },
            ))
        }
        other => Err(SpeechError::UnknownProvider(other.to_string())),
    }
}

/// Runs one spec over the selected samples. `on_record` is called after every run so callers can
/// stream progress or results.
pub fn run_spec(
    models: &Arc<ModelManager>,
    ds: &Dataset,
    spec: &RunSpec,
    opts: &RunOptions,
    run_id: &str,
    cancel: &CancelToken,
    on_record: &mut dyn FnMut(&RunRecord),
) -> Result<Vec<RunRecord>, SpeechError> {
    let (provider, provider_id, decoding) = build_provider(models, spec, opts)?;
    let vad_model = if opts.chunking { Some(models.vad_model_path()?) } else { None };
    let chunk_cfg = ChunkConfig {
        max_segment_s: opts.chunk_max_s.unwrap_or(ChunkConfig::default().max_segment_s),
        ..ChunkConfig::default()
    };
    let samples = select_samples(ds, opts);
    let reps = opts.repetitions.max(1);
    let mut out = Vec::new();

    for sample in samples {
        for repetition in 1..=reps {
            if cancel.is_cancelled() {
                return Err(SpeechError::Cancelled);
            }
            let request = TranscribeRequest {
                provider_id: provider_id.clone(),
                model_id: spec.model_id.clone(),
                language: sample.language.clone(),
                audio_path: ds.audio_path(sample).display().to_string(),
                vocabulary: opts.vocabulary.get(&sample.language).cloned().unwrap_or_default(),
            };
            let mut rec = RunRecord {
                run_id: run_id.to_string(),
                scoring_version: SCORING_VERSION,
                timestamp_ms: now_ms(),
                model_id: spec.model_id.clone(),
                provider: provider_id.clone(),
                decoding: decoding.clone(),
                sample_id: sample.id.clone(),
                speaker_id: sample.speaker.id.clone(),
                category: sample.category.clone(),
                language: sample.language.clone(),
                domain: sample.domain.clone(),
                utterance_type: sample.utterance_type.clone(),
                private: sample.private,
                repetition,
                cold_start: false,
                load_ms: 0,
                inference_ms: 0,
                audio_ms: sample.duration_ms.unwrap_or(0),
                rtf: None,
                chunking: if opts.chunking { "vad".into() } else { chunking_off() },
                segments: 1,
                chunking_ms: 0,
                biasing: biasing_none(),
                post_correction: biasing_none(),
                raw_text: None,
                threads: 0,
                peak_memory_mb: None,
                cpu_ms: None,
                avg_cores: None,
                reference: sample.reference.clone(),
                text: String::new(),
                wer: None,
                cer: None,
                substitutions: 0,
                deletions: 0,
                insertions: 0,
                reference_words: 0,
                critical: Vec::new(),
                has_critical: false,
                error: None,
            };
            let outcome = match &vad_model {
                Some(model) => {
                    let detect = |samples: &[f32]| vad::silero_speech_spans(model, samples);
                    chunking::transcribe_chunked(provider.as_ref(), &request, cancel, &detect, &chunk_cfg)
                        .map(|(r, report)| (r, report.segments as u32, report.chunking_ms))
                }
                None => provider.transcribe(&request, cancel).map(|r| (r, 1, 0)),
            };
            match outcome {
                Ok((r, segments, chunking_ms)) => {
                    rec.segments = segments;
                    rec.chunking_ms = chunking_ms;
                    if !request.vocabulary.is_empty() {
                        rec.biasing = match r.decoding.split_once(" + ") {
                            Some((_, technique)) => technique.to_string(),
                            None => "ignored by engine".into(),
                        };
                    } else if opts.hotwords_score.is_some() && r.decoding.starts_with("modified beam") {
                        rec.biasing = format!("{} without vocabulary (control)", r.decoding);
                    }
                    let cmp = metrics::compare_texts(&sample.reference, &r.text);
                    let report = critical::analyze(&sample.reference, &r.text, &sample.key_terms);
                    rec.cold_start = r.cold_start;
                    rec.load_ms = r.load_ms;
                    rec.inference_ms = r.processing_ms;
                    rec.audio_ms = r.audio_ms.unwrap_or(rec.audio_ms);
                    rec.rtf = r.rtf;
                    rec.threads = r.threads;
                    rec.peak_memory_mb = r.peak_memory_mb;
                    rec.cpu_ms = r.cpu_ms;
                    let wall = r.load_ms + r.processing_ms;
                    rec.avg_cores = r.cpu_ms.filter(|_| wall > 0).map(|c| c as f64 / wall as f64);
                    rec.text = r.text;
                    rec.wer = cmp.wer;
                    rec.cer = cmp.cer;
                    rec.substitutions = cmp.substitutions;
                    rec.deletions = cmp.deletions;
                    rec.insertions = cmp.insertions;
                    rec.reference_words = cmp.reference_words;
                    rec.has_critical = report.has_critical;
                    rec.critical = report.flags;
                }
                Err(SpeechError::Cancelled) => return Err(SpeechError::Cancelled),
                Err(e) => rec.error = Some(e.to_string()),
            }
            on_record(&rec);
            out.push(rec);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------- summary

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryRow {
    pub model_id: String,
    pub decoding: String,
    /// A category name, or "ALL".
    pub group: String,
    pub samples: usize,
    pub runs: usize,
    pub failures: usize,
    /// (substitutions + deletions + insertions) / reference words, summed over samples.
    pub wer_micro: Option<f64>,
    /// Mean of the per-sample WER.
    pub wer_macro: Option<f64>,
    pub cer_macro: Option<f64>,
    /// Samples whose WER is 100 % or more: garbage, or runaway repetition loops.
    pub runaway_outputs: usize,
    pub samples_with_critical: usize,
    pub critical_flags: usize,
    pub flags_by_kind: BTreeMap<String, usize>,
    /// Samples whose text differs between repetitions.
    pub unstable_samples: usize,
    pub rtf_median: Option<f64>,
    pub inference_ms_median: Option<f64>,
    pub inference_ms_p95: Option<f64>,
    /// Median model load time over cold-start runs.
    pub load_ms_cold_median: Option<f64>,
    pub peak_memory_mb_max: Option<f64>,
    pub avg_cores_median: Option<f64>,
}

/// Samples that are hard for most configurations. When most engines agree with each other and
/// not with the reference, the reference or the reading may be wrong: listen to these first.
pub fn hard_samples(records: &[RunRecord], min_median_wer: f64) -> Vec<(String, f64, &RunRecord)> {
    let mut by_sample: BTreeMap<&str, Vec<&RunRecord>> = BTreeMap::new();
    for r in records.iter().filter(|r| r.repetition == 1 && r.error.is_none()) {
        by_sample.entry(r.sample_id.as_str()).or_default().push(r);
    }
    let mut out = Vec::new();
    for (id, rs) in by_sample {
        let wers = sorted(rs.iter().filter_map(|r| r.wer).collect());
        let Some(median) = percentile(&wers, 50.0) else { continue };
        if median >= min_median_wer {
            let best = rs
                .iter()
                .copied()
                .min_by(|a, b| a.wer.partial_cmp(&b.wer).unwrap_or(std::cmp::Ordering::Equal))
                .expect("non-empty");
            out.push((id.to_string(), median, best));
        }
    }
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn hard_samples_markdown(records: &[RunRecord]) -> String {
    let hard = hard_samples(records, 0.25);
    let mut md = String::from("\n## Hard or suspect samples (median WER over all configurations >= 25 %)\n\n");
    md.push_str("If most engines agree with each other but not with the reference, the speaker may have read or said something different: listen to the audio before blaming the engines.\n\n");
    if hard.is_empty() {
        md.push_str("None.\n");
    }
    for (id, median, best) in hard {
        md.push_str(&format!(
            "- `{id}` (median WER {:.0} %): reference \"{}\"; best output ({}, {}): \"{}\"\n",
            median * 100.0, best.reference, best.model_id, best.decoding, best.text
        ));
    }
    md
}

pub fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    // Nearest rank.
    let rank = ((p / 100.0) * sorted.len() as f64).ceil().max(1.0) as usize;
    Some(sorted[rank.min(sorted.len()) - 1])
}

fn sorted(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v
}

fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

pub fn summarize(records: &[RunRecord]) -> Vec<SummaryRow> {
    let mut configs: Vec<(String, String)> = Vec::new();
    for r in records {
        let key = (r.model_id.clone(), r.decoding.clone());
        if !configs.contains(&key) {
            configs.push(key);
        }
    }
    let mut rows = Vec::new();
    for (model, decoding) in configs {
        let of: Vec<&RunRecord> = records.iter().filter(|r| r.model_id == model && r.decoding == decoding).collect();
        let mut groups: Vec<String> = vec!["ALL".into()];
        for r in &of {
            if !groups.contains(&r.category) {
                groups.push(r.category.clone());
            }
        }
        for group in groups {
            let rs: Vec<&RunRecord> = of.iter().copied().filter(|r| group == "ALL" || r.category == group).collect();
            let ok: Vec<&RunRecord> = rs.iter().copied().filter(|r| r.error.is_none()).collect();
            let first: Vec<&RunRecord> = ok.iter().copied().filter(|r| r.repetition == 1).collect();

            let errors: usize = first.iter().map(|r| r.substitutions + r.deletions + r.insertions).sum();
            let words: usize = first.iter().map(|r| r.reference_words).sum();
            let wers: Vec<f64> = first.iter().filter_map(|r| r.wer).collect();
            let cers: Vec<f64> = first.iter().filter_map(|r| r.cer).collect();

            let mut flags_by_kind: BTreeMap<String, usize> = BTreeMap::new();
            for r in &first {
                for f in &r.critical {
                    let name = serde_json::to_value(f.kind).ok().and_then(|v| v.as_str().map(str::to_string));
                    *flags_by_kind.entry(name.unwrap_or_else(|| "other".into())).or_default() += 1;
                }
            }

            let mut by_sample: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
            for r in &ok {
                by_sample.entry(r.sample_id.as_str()).or_default().push(r.text.as_str());
            }
            let unstable = by_sample.values().filter(|t| t.iter().any(|x| x != &t[0])).count();

            let rtf = sorted(ok.iter().filter_map(|r| r.rtf).collect());
            let inf = sorted(ok.iter().map(|r| r.inference_ms as f64).collect());
            let cold = sorted(ok.iter().filter(|r| r.cold_start).map(|r| r.load_ms as f64).collect());
            let cores = sorted(ok.iter().filter_map(|r| r.avg_cores).collect());

            rows.push(SummaryRow {
                model_id: model.clone(),
                decoding: decoding.clone(),
                group,
                samples: by_sample.len(),
                runs: rs.len(),
                failures: rs.len() - ok.len(),
                wer_micro: (words > 0).then(|| errors as f64 / words as f64),
                wer_macro: mean(&wers),
                cer_macro: mean(&cers),
                runaway_outputs: first.iter().filter(|r| r.wer.is_some_and(|w| w >= 1.0)).count(),
                samples_with_critical: first.iter().filter(|r| r.has_critical).count(),
                critical_flags: flags_by_kind.values().sum(),
                flags_by_kind,
                unstable_samples: unstable,
                rtf_median: percentile(&rtf, 50.0),
                inference_ms_median: percentile(&inf, 50.0),
                inference_ms_p95: percentile(&inf, 95.0),
                load_ms_cold_median: percentile(&cold, 50.0),
                peak_memory_mb_max: ok.iter().filter_map(|r| r.peak_memory_mb).fold(None, |a: Option<f64>, v| Some(a.map_or(v, |x| x.max(v)))),
                avg_cores_median: percentile(&cores, 50.0),
            });
        }
    }
    rows
}

// ---------------------------------------------------------------- output files

fn fmt_opt(v: Option<f64>, digits: usize) -> String {
    v.map(|x| format!("{x:.digits$}")).unwrap_or_default()
}

fn pct(v: Option<f64>) -> String {
    v.map(|x| format!("{:.1} %", x * 100.0)).unwrap_or_else(|| "n/a".into())
}

fn csv_escape(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn summary_csv(rows: &[SummaryRow]) -> String {
    let mut out = String::from(
        "model,decoding,group,samples,runs,failures,wer_micro,wer_macro,cer_macro,runaway_outputs,samples_with_critical,critical_flags,unstable_samples,rtf_median,inference_ms_median,inference_ms_p95,load_ms_cold_median,peak_memory_mb_max,avg_cores_median\n",
    );
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&r.model_id),
            csv_escape(&r.decoding),
            csv_escape(&r.group),
            r.samples,
            r.runs,
            r.failures,
            fmt_opt(r.wer_micro, 4),
            fmt_opt(r.wer_macro, 4),
            fmt_opt(r.cer_macro, 4),
            r.runaway_outputs,
            r.samples_with_critical,
            r.critical_flags,
            r.unstable_samples,
            fmt_opt(r.rtf_median, 3),
            fmt_opt(r.inference_ms_median, 0),
            fmt_opt(r.inference_ms_p95, 0),
            fmt_opt(r.load_ms_cold_median, 0),
            fmt_opt(r.peak_memory_mb_max, 0),
            fmt_opt(r.avg_cores_median, 2),
        ));
    }
    out
}

/// Human-readable summary: overall table, per-category WER, and the critical flags to review.
pub fn summary_markdown(system: &SystemInfo, rows: &[SummaryRow], records: &[RunRecord]) -> String {
    let mut md = String::new();
    md.push_str("# Benchmark summary\n\n");
    md.push_str(&format!(
        "Machine: {} ({} logical cores), {} GB RAM, {} {}, build {}, {}.\n\n",
        system.cpu, system.logical_cores, system.total_ram_gb, system.os, system.arch, system.rust_build, system.accelerator
    ));
    if records.iter().any(|r| r.chunking != "off") {
        md.push_str("Chunking: clips longer than 25 s were cut at silences (Silero VAD) and the texts joined; shorter clips were passed whole. ");
        md.push_str("Inference time is the sum over the segments; speech detection is extra (`chunkingMs` in runs.jsonl).

");
    }
    md.push_str("Accuracy uses repetition 1 only. WER micro = total errors / total reference words; ");
    md.push_str("macro = mean of per-sample WER. Critical = samples with at least one critical flag ");
    md.push_str("(changed number, unit, negation, weekday/month, missing key term). ");
    md.push_str("Memory is a sampled lower bound. One speaker only: these are not general claims.\n\n");

    md.push_str("## Overall\n\n| Model | Decoding | Samples | Failures | WER micro | WER macro | CER | Runaway (WER>=100%) | Critical samples | Flags (incl. warnings) | RTF med | Infer med (ms) | Infer p95 (ms) | Cold load (ms) | Peak mem (MB) | Busy cores |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for r in rows.iter().filter(|r| r.group == "ALL") {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            r.model_id, r.decoding, r.samples, r.failures, pct(r.wer_micro), pct(r.wer_macro), pct(r.cer_macro),
            r.runaway_outputs, r.samples_with_critical, r.critical_flags, fmt_opt(r.rtf_median, 3), fmt_opt(r.inference_ms_median, 0),
            fmt_opt(r.inference_ms_p95, 0), fmt_opt(r.load_ms_cold_median, 0), fmt_opt(r.peak_memory_mb_max, 0),
            fmt_opt(r.avg_cores_median, 1)
        ));
    }

    let groups: Vec<&str> = {
        let mut g: Vec<&str> = Vec::new();
        for r in rows.iter().filter(|r| r.group != "ALL") {
            if !g.contains(&r.group.as_str()) {
                g.push(&r.group);
            }
        }
        g
    };
    if !groups.is_empty() {
        md.push_str("\n## WER micro by category (critical samples in brackets)\n\n| Model | Decoding |");
        for g in &groups {
            md.push_str(&format!(" {g} |"));
        }
        md.push_str("\n|---|---|");
        for _ in &groups {
            md.push_str("---|");
        }
        md.push('\n');
        let mut configs: Vec<(&str, &str)> = Vec::new();
        for r in rows {
            if !configs.contains(&(r.model_id.as_str(), r.decoding.as_str())) {
                configs.push((&r.model_id, &r.decoding));
            }
        }
        for (m, d) in configs {
            md.push_str(&format!("| {m} | {d} |"));
            for g in &groups {
                match rows.iter().find(|r| r.model_id == m && r.decoding == d && r.group == *g) {
                    Some(r) => md.push_str(&format!(" {} ({}) |", pct(r.wer_micro), r.samples_with_critical)),
                    None => md.push_str(" - |"),
                }
            }
            md.push('\n');
        }
    }

    md.push_str(&hard_samples_markdown(records));
    md.push_str("\n## Critical flags to review (repetition 1, first 6 per configuration)\n\n");
    let mut shown: BTreeMap<(String, String), usize> = BTreeMap::new();
    for r in records.iter().filter(|r| r.repetition == 1 && r.has_critical) {
        let n = shown.entry((r.model_id.clone(), r.decoding.clone())).or_default();
        if *n >= 6 {
            continue;
        }
        *n += 1;
        for f in &r.critical {
            md.push_str(&format!(
                "- {} / {} / {}: {:?} expected `{}` found `{}`: {}\n",
                r.model_id, r.decoding, r.sample_id, f.kind, f.expected, f.found, f.message
            ));
        }
    }
    md
}

pub fn write_results(
    dir: &Path,
    system: &SystemInfo,
    config: &serde_json::Value,
    records: &[RunRecord],
) -> Result<Vec<SummaryRow>, SpeechError> {
    std::fs::create_dir_all(dir).map_err(|e| SpeechError::Engine(e.to_string()))?;
    let io = |e: std::io::Error| SpeechError::Engine(e.to_string());
    let json = |e: serde_json::Error| SpeechError::Engine(e.to_string());
    std::fs::write(dir.join("system.json"), serde_json::to_string_pretty(system).map_err(json)? + "\n").map_err(io)?;
    std::fs::write(dir.join("config.json"), serde_json::to_string_pretty(config).map_err(json)? + "\n").map_err(io)?;
    let mut lines = String::new();
    for r in records {
        lines.push_str(&serde_json::to_string(r).map_err(json)?);
        lines.push('\n');
    }
    std::fs::write(dir.join("runs.jsonl"), lines).map_err(io)?;
    let rows = summarize(records);
    std::fs::write(dir.join("summary.csv"), summary_csv(&rows)).map_err(io)?;
    std::fs::write(dir.join("summary.md"), summary_markdown(system, &rows, records)).map_err(io)?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic records are used ONLY to test the aggregation code, never as results.
    fn rec(model: &str, sample: &str, cat: &str, rep: u32, wer: f64, errs: (usize, usize, usize), words: usize, infer: u64, text: &str) -> RunRecord {
        RunRecord {
            run_id: "t".into(), scoring_version: SCORING_VERSION, timestamp_ms: 0, model_id: model.into(), provider: "p".into(), decoding: "greedy search".into(),
            sample_id: sample.into(), speaker_id: "s".into(), category: cat.into(), language: "fr".into(), domain: "general".into(),
            utterance_type: "statement".into(), private: false, repetition: rep, cold_start: rep == 1, load_ms: 100, inference_ms: infer,
            audio_ms: 1000, rtf: Some(infer as f64 / 1000.0), chunking: "off".into(), segments: 1, chunking_ms: 0, biasing: "none".into(), post_correction: "none".into(), raw_text: None, threads: 4, peak_memory_mb: Some(200.0 + infer as f64), cpu_ms: Some(infer * 2),
            avg_cores: Some(2.0), reference: "r".into(), text: text.into(), wer: Some(wer), cer: Some(wer / 2.0), substitutions: errs.0,
            deletions: errs.1, insertions: errs.2, reference_words: words, critical: vec![], has_critical: false, error: None,
        }
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        let v = [10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&v, 50.0), Some(30.0));
        assert_eq!(percentile(&v, 95.0), Some(50.0));
        assert_eq!(percentile(&v, 1.0), Some(10.0));
        assert_eq!(percentile(&[], 50.0), None);
    }

    #[test]
    fn utc_stamp_formats_known_instants() {
        assert_eq!(utc_stamp(0), "19700101-000000");
        // A clip recorded at 09:28:03 local time (UTC+2) carries the stamp 1791358083.
        assert_eq!(utc_stamp(1_791_358_083_000), "20261007-072803");
        assert_eq!(utc_stamp(951_782_400_000), "20000229-000000"); // leap day
    }

    #[test]
    fn summary_micro_vs_macro_wer_and_timing_use_the_right_repetitions() {
        let records = vec![
            rec("m", "a", "c1", 1, 0.5, (1, 0, 0), 2, 100, "x"),
            rec("m", "a", "c1", 2, 0.5, (1, 0, 0), 2, 300, "x"),
            rec("m", "b", "c1", 1, 0.0, (0, 0, 0), 8, 200, "y"),
            rec("m", "b", "c1", 2, 0.0, (0, 0, 0), 8, 200, "y"),
        ];
        let rows = summarize(&records);
        let all = rows.iter().find(|r| r.group == "ALL").unwrap();
        assert_eq!(all.samples, 2);
        assert_eq!(all.runs, 4);
        assert_eq!(all.wer_micro, Some(1.0 / 10.0), "1 error over 10 reference words");
        assert_eq!(all.wer_macro, Some(0.25), "mean of 0.5 and 0.0");
        assert_eq!(all.inference_ms_median, Some(200.0));
        assert_eq!(all.load_ms_cold_median, Some(100.0));
        assert_eq!(all.unstable_samples, 0);
        assert_eq!(rows.iter().filter(|r| r.group == "c1").count(), 1);
    }

    #[test]
    fn failures_are_counted_and_excluded_from_accuracy_and_unstable_samples_are_detected() {
        let mut bad = rec("m", "c", "c1", 1, 0.0, (0, 0, 0), 5, 100, "");
        bad.error = Some("boom".into());
        let records = vec![
            bad,
            rec("m", "a", "c1", 1, 0.0, (0, 0, 0), 4, 100, "same"),
            rec("m", "a", "c1", 2, 0.0, (0, 0, 0), 4, 100, "different"),
        ];
        let all = summarize(&records).into_iter().find(|r| r.group == "ALL").unwrap();
        assert_eq!(all.failures, 1);
        assert_eq!(all.wer_micro, Some(0.0));
        assert_eq!(all.unstable_samples, 1);
    }

    #[test]
    fn hard_samples_are_found_by_median_across_configurations_and_runaways_are_counted() {
        let mut records = Vec::new();
        for (model, w) in [("a", 0.4), ("b", 0.5), ("c", 0.3), ("d", 1.5)] {
            records.push(rec(model, "hard", "c1", 1, w, (1, 0, 0), 5, 100, "x"));
            records.push(rec(model, "easy", "c1", 1, 0.0, (0, 0, 0), 5, 100, "x"));
        }
        let hard = hard_samples(&records, 0.25);
        assert_eq!(hard.len(), 1);
        assert_eq!(hard[0].0, "hard");
        let md = summary_markdown(&system_info(), &summarize(&records), &records);
        assert!(md.contains("Hard or suspect samples") && md.contains("`hard`"));
        let d = summarize(&records).into_iter().find(|r| r.model_id == "d" && r.group == "ALL").unwrap();
        assert_eq!(d.runaway_outputs, 1, "WER 1.5 is a runaway output");
    }

    #[test]
    fn rescore_applies_current_rules_to_stored_transcripts() {
        let mut r = rec("m", "a", "c1", 1, 0.9, (5, 0, 0), 5, 100, "à 14h");
        r.scoring_version = 1;
        r.reference = "à 14 heures".into();
        r.critical = vec![];
        let mut v = vec![r];
        rescore(&mut v, &BTreeMap::new());
        assert_eq!(v[0].wer, Some(0.0), "glued 14h equals 14 heures under the current rules");
        assert!(!v[0].has_critical && v[0].critical.is_empty());
        assert_eq!(v[0].scoring_version, SCORING_VERSION);
    }

    #[test]
    fn csv_escapes_and_markdown_mentions_limits() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        let rows = summarize(&[rec("m", "a", "c1", 1, 0.1, (1, 0, 0), 10, 100, "x")]);
        let md = summary_markdown(&system_info(), &rows, &[]);
        assert!(md.contains("One speaker only"));
        assert!(summary_csv(&rows).lines().count() == 3);
    }

    #[test]
    fn system_info_reports_this_machine() {
        let s = system_info();
        assert!(s.logical_cores >= 1);
        assert!(s.total_ram_gb > 0.5);
        assert!(!s.cpu.is_empty());
    }

    #[test]
    fn audio_analysis_flags_clipping_and_silence() {
        let dir = std::env::temp_dir().join("speechlab_aq");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("loud.wav");
        let spec = hound::WavSpec { channels: 1, sample_rate: 16000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for _ in 0..16000 {
            w.write_sample(0i16).unwrap(); // 1 s of silence first
        }
        for i in 0..16000 {
            w.write_sample(if i % 2 == 0 { 32767i16 } else { -32768 }).unwrap(); // 1 s fully clipped
        }
        w.finalize().unwrap();
        let q = analyze_audio(&path, "un deux trois quatre cinq six sept").unwrap();
        assert_eq!(q.duration_ms, 2000);
        assert!(q.clipped_ratio > 0.4);
        assert!(q.leading_silence_ms >= 900 && q.leading_silence_ms <= 1000);
        assert!(q.warnings.iter().any(|w| w.starts_with("clipping")));
        let _ = std::fs::remove_dir_all(dir);
    }
}
