//! Reproducible benchmark CLI.
//!
//!   cargo run --release --example bench -- check
//!   cargo run --release --example bench -- preflight     (is the machine quiet enough? exit 0 or 2)
//!   cargo run --release --example bench -- run [options]
//!   cargo run --release --example bench -- summarize --dir <results folder>
//!   cargo run --release --example bench -- rescore --dir <results folder>
//!   cargo run --release --example bench -- termstudy --dir <baseline> [--against <variant>] [--category a,b] [--out file.md]
//!   cargo run --release --example bench -- postcorrect --dir <run> --vocab-dir <dir> [--preset strict|medium|loose] --label <name>
//!
//! run options:
//!   --models a,b,c      model ids (default: every installed model)
//!   --beams 5,1         whisper.cpp beam sizes to test (default 5,1; sherpa-onnx is always greedy)
//!   --reps N            repetitions per sample (default 1; use 3 or more for timing studies)
//!   --category c1,c2    only these categories (default: all)
//!   --limit N           only the first N samples (smoke tests)
//!   --label text        suffix of the results folder name
//!   --include-private   include samples-private/ (results then go to results/private/)
//!   --chunking vad      cut clips longer than 25 s at silences (Silero VAD) before the engine; the
//!                       default is off (whole clip). Needs the support model: transcribe install silero-vad
//!   --chunk-max-s N     longest segment when chunking, in seconds (default 25; experiments only)
//!   --vocab-dir DIR     bias the engines with the vocabulary files DIR/fr.txt and DIR/en.txt (one term per
//!                       line): whisper.cpp gets an initial prompt, sherpa-onnx Parakeet gets hotwords when
//!                       --hotwords-score is also given; other engines ignore it (D-036)
//!   --hotwords-score S  sherpa-onnx NeMo transducer: modified beam search with this hotwords score; without
//!                       --vocab-dir it is the beam-search control run
//!   --no-isolate        run every model inside this process (memory figures then mix models)
//!   --force             run even if the quiet-machine check fails (config.json records it)
//!
//! By default each model runs in its own child process, so cold-start time and memory belong to
//! that model alone. Results: benchmark/results/<UTC stamp>-<label>/ (runs.jsonl, summary.csv,
//! summary.md, system.json, config.json). Use --release for meaningful speed figures.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

use speechlab_lib::speech::benchmark::{
    analyze_audio, now_ms, run_spec, select_samples, summary_markdown, system_info, utc_stamp, write_results,
    preflight, rescore, summarize, summary_csv, RunOptions, RunRecord, RunSpec, SystemInfo,
};
use speechlab_lib::speech::dataset::{default_root, Dataset};
use speechlab_lib::speech::models::ModelManager;
use speechlab_lib::speech::postcorrect::{self, PostCorrectConfig};
use speechlab_lib::speech::provider::CancelToken;
use speechlab_lib::speech::termstudy::{self, KeyTerms};
use speechlab_lib::speech::types::InstallStatus;

// Background CPU load tolerated before a run (D-033, relaxed from 15 % to 30 % by D-034 so that the
// figures are measured under a realistic client-machine load).
const MAX_BACKGROUND_CPU_PERCENT: f64 = 30.0;

const RECORD_PREFIX: &str = "RECORD\t";

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
    values: HashMap<String, String>,
    flags: Vec<String>,
}

impl Args {
    fn parse(raw: &[String]) -> Args {
        let mut values = HashMap::new();
        let mut flags = Vec::new();
        let mut i = 0;
        while i < raw.len() {
            if let Some(k) = raw[i].strip_prefix("--") {
                if raw.get(i + 1).is_some_and(|n| !n.starts_with("--")) {
                    values.insert(k.to_string(), raw[i + 1].clone());
                    i += 1;
                } else {
                    flags.push(k.to_string());
                }
            }
            i += 1;
        }
        Args { values, flags }
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.values.get(k).map(String::as_str)
    }
    fn list(&self, k: &str) -> Vec<String> {
        self.get(k).map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default()
    }
    fn flag(&self, k: &str) -> bool {
        self.flags.iter().any(|f| f == k)
    }
    fn options(&self) -> RunOptions {
        RunOptions {
            repetitions: self.get("reps").and_then(|v| v.parse().ok()).unwrap_or(1),
            categories: self.list("category"),
            limit: self.get("limit").and_then(|v| v.parse().ok()),
            include_private: self.flag("include-private"),
            chunking: match self.get("chunking") {
                None | Some("off") => false,
                Some("vad") => true,
                Some(other) => panic!("--chunking expects 'vad' or 'off', got '{other}'"),
            },
            chunk_max_s: self.get("chunk-max-s").and_then(|v| v.parse().ok()),
            vocabulary: self.get("vocab-dir").map(|d| load_vocab_dir(&PathBuf::from(d))).unwrap_or_default(),
            hotwords_score: self.get("hotwords-score").and_then(|v| v.parse().ok()),
        }
    }
}

/// Vocabulary files by language: DIR/fr.txt and DIR/en.txt.
fn load_vocab_dir(dir: &std::path::Path) -> std::collections::BTreeMap<String, Vec<String>> {
    ["fr", "en"]
        .iter()
        .map(|lang| {
            let file = dir.join(format!("{lang}.txt"));
            let terms = postcorrect::load_vocabulary(&file).unwrap_or_else(|e| panic!("cannot read {}: {e}", file.display()));
            (lang.to_string(), terms)
        })
        .collect()
}

fn check() {
    let root = default_root();
    let ds = Dataset::load(&root).expect("dataset");
    println!("dataset: {}", root.display());
    println!("samples: {} ({} private), validation issues: {}", ds.samples.len(), ds.samples.iter().filter(|s| s.private).count(), ds.issues.len());
    for i in &ds.issues {
        println!("  ISSUE {}: {}", i.sample_id, i.message);
    }
    let mut by_cat: std::collections::BTreeMap<String, (usize, u64)> = Default::default();
    let (mut hot, mut clipped, mut flagged) = (0, 0, 0);
    let mut peaks = Vec::new();
    for s in ds.runnable() {
        let e = by_cat.entry(s.category.clone()).or_default();
        e.0 += 1;
        e.1 += s.duration_ms.unwrap_or(0);
        match analyze_audio(&ds.audio_path(s), &s.reference) {
            Ok(q) => {
                peaks.push(q.peak);
                hot += usize::from(q.hot_ratio > 0.02);
                clipped += usize::from(q.clipped_ratio > 0.001);
                if !q.warnings.is_empty() {
                    flagged += 1;
                    println!("  WARN {}: {}", s.id, q.warnings.join("; "));
                }
            }
            Err(e) => println!("  ERROR {}: {e}", s.id),
        }
    }
    println!("\ncategory                 samples  minutes");
    for (c, (n, ms)) in &by_cat {
        println!("{c:24} {n:7}  {:7.1}", *ms as f64 / 60000.0);
    }
    peaks.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if !peaks.is_empty() {
        println!(
            "\naudio: {} files with warnings, {} with clipping (>0.1 % of samples at full scale), {} hot (>2 % above 90 %). Peak min/median/max: {:.3} / {:.3} / {:.3}",
            flagged, clipped, hot, peaks[0], peaks[peaks.len() / 2], peaks[peaks.len() - 1]
        );
    }
}

fn installed_models(models: &ModelManager) -> Vec<String> {
    models.list().into_iter().filter(|m| m.install_status == InstallStatus::Installed).map(|m| m.id).collect()
}

fn specs_for(models: &ModelManager, ids: &[String], beams: &[u32]) -> Vec<RunSpec> {
    let mut specs = Vec::new();
    for id in ids {
        let provider = models.def(id).map(|d| d.provider.clone()).unwrap_or_default();
        if provider == "whisper-cpp" {
            for b in beams {
                specs.push(RunSpec { model_id: id.clone(), beam_size: Some(*b) });
            }
        } else {
            specs.push(RunSpec { model_id: id.clone(), beam_size: None });
        }
    }
    specs
}

/// Child mode: run one spec and print every record on stdout.
fn run_one(args: &Args) {
    let models = Arc::new(ModelManager::new(models_dir()).expect("manifest"));
    let ds = Dataset::load(&default_root()).expect("dataset");
    let spec = RunSpec {
        model_id: args.get("model").expect("--model").to_string(),
        beam_size: args.get("beam").and_then(|b| b.parse().ok()),
    };
    let opts = args.options();
    let run_id = args.get("run-id").unwrap_or("run").to_string();
    let total = select_samples(&ds, &opts).len() * opts.repetitions.max(1) as usize;
    let mut done = 0;
    run_spec(&models, &ds, &spec, &opts, &run_id, &CancelToken::new(), &mut |r| {
        done += 1;
        println!("{RECORD_PREFIX}{}", serde_json::to_string(r).expect("json"));
        if done % 10 == 0 || done == total {
            eprintln!("    {}/{} runs ({})", done, total, r.sample_id);
        }
    })
    .expect("run failed");
}

fn run(args: &Args) {
    let models = Arc::new(ModelManager::new(models_dir()).expect("manifest"));
    let mut ids = args.list("models");
    if ids.is_empty() {
        ids = installed_models(&models);
    }
    for id in &ids {
        let def = models.def(id).unwrap_or_else(|_| panic!("unknown model {id}"));
        assert!(models.is_installed(def), "model {id} is not installed");
    }
    let beams: Vec<u32> = {
        let b: Vec<u32> = args.list("beams").iter().filter_map(|v| v.parse().ok()).collect();
        if b.is_empty() { vec![5, 1] } else { b }
    };
    let specs = specs_for(&models, &ids, &beams);
    let opts = args.options();
    let ds = Dataset::load(&default_root()).expect("dataset");
    let n = select_samples(&ds, &opts).len();
    assert!(n > 0, "no runnable samples (run `bench check`)");
    if opts.chunking {
        models.vad_model_path().expect("--chunking vad needs the support model: transcribe install silero-vad");
    }

    // Quiet-machine check: timing and memory figures are only meaningful on an idle machine.
    let pre = preflight(8000, MAX_BACKGROUND_CPU_PERCENT);
    println!(
        "preflight: CPU {:.0} % average over {} s, power {}, busiest: {}",
        pre.idle_cpu_percent,
        pre.window_ms / 1000,
        match pre.on_ac_power { Some(true) => "mains", Some(false) => "BATTERY", None => "unknown" },
        pre.top_processes.iter().map(|(n, c)| format!("{n} {c:.0}%")).collect::<Vec<_>>().join(", ")
    );
    if !pre.problems.is_empty() && !args.flag("force") {
        for p in &pre.problems {
            eprintln!("PREFLIGHT FAILED: {p}");
        }
        eprintln!("Close other programs (or plug in) and run again, or pass --force to run anyway (results will be marked as disturbed).");
        std::process::exit(2);
    }

    let started = now_ms();
    let stamp = utc_stamp(started);
    let label = args.get("label").unwrap_or("run");
    let run_id = format!("{stamp}-{label}");
    let base = if opts.include_private { default_root().join("results").join("private") } else { default_root().join("results") };
    let out_dir = base.join(&run_id);
    println!("run {run_id}: {} configurations x {n} samples x {} repetition(s)", specs.len(), opts.repetitions.max(1));

    let exe = std::env::current_exe().expect("exe");
    let mut records: Vec<RunRecord> = Vec::new();
    for (i, spec) in specs.iter().enumerate() {
        println!("[{}/{}] {} {}", i + 1, specs.len(), spec.model_id, spec.beam_size.map(|b| format!("beam {b}")).unwrap_or_default());
        if args.flag("no-isolate") {
            let r = run_spec(&models, &ds, spec, &opts, &run_id, &CancelToken::new(), &mut |_| {}).expect("run failed");
            records.extend(r);
            continue;
        }
        let mut cmd = Command::new(&exe);
        cmd.arg("run-one").args(["--model", &spec.model_id, "--run-id", &run_id, "--reps", &opts.repetitions.max(1).to_string()]);
        if let Some(b) = spec.beam_size {
            cmd.args(["--beam", &b.to_string()]);
        }
        if !opts.categories.is_empty() {
            cmd.args(["--category", &opts.categories.join(",")]);
        }
        if let Some(l) = opts.limit {
            cmd.args(["--limit", &l.to_string()]);
        }
        if opts.include_private {
            cmd.arg("--include-private");
        }
        if let Some(d) = args.get("vocab-dir") {
            cmd.args(["--vocab-dir", d]);
        }
        if let Some(s) = args.get("hotwords-score") {
            cmd.args(["--hotwords-score", s]);
        }
        if opts.chunking {
            cmd.args(["--chunking", "vad"]);
            if let Some(m) = opts.chunk_max_s {
                cmd.args(["--chunk-max-s", &m.to_string()]);
            }
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());
        let mut child = cmd.spawn().expect("spawn child");
        let reader = BufReader::new(child.stdout.take().expect("stdout"));
        let before = records.len();
        for line in reader.lines().map_while(Result::ok) {
            if let Some(json) = line.strip_prefix(RECORD_PREFIX) {
                records.push(serde_json::from_str(json).expect("record json"));
            }
        }
        let status = child.wait().expect("wait");
        println!("    {} records, child exit: {status}", records.len() - before);
        assert!(status.success(), "child failed for {}", spec.model_id);
    }

    let system = system_info();
    let config = serde_json::json!({
        "runId": run_id,
        "startedAtMs": started,
        "models": ids,
        "beams": beams,
        "repetitions": opts.repetitions.max(1),
        "categories": opts.categories,
        "limit": opts.limit,
        "includePrivate": opts.include_private,
        "chunking": if opts.chunking { "vad" } else { "off" },
        "chunkMaxS": opts.chunk_max_s,
        "vocabularyDir": args.get("vocab-dir"),
        "vocabularyTerms": opts.vocabulary.iter().map(|(l, v)| (l.clone(), v.len())).collect::<std::collections::BTreeMap<_, _>>(),
        "hotwordsScore": opts.hotwords_score,
        "isolatedProcesses": !args.flag("no-isolate"),
        "samples": n,
        "threadsPerEngine": 4,
        "preflight": pre,
        "forcedDespitePreflight": !pre.problems.is_empty(),
    });
    let rows = write_results(&out_dir, &system, &config, &records).expect("write results");
    println!("\nresults written to {}\n", out_dir.display());
    println!("{}", summary_markdown(&system, &rows, &records).lines().take(14).collect::<Vec<_>>().join("\n"));
}

/// Rebuilds summary.csv / summary.md from an existing runs.jsonl (e.g. after the scoring improved
/// or to refresh the report), without running any engine.
fn resummarize(args: &Args) {
    let dir = PathBuf::from(args.get("dir").expect("--dir <results folder>"));
    let text = std::fs::read_to_string(dir.join("runs.jsonl")).expect("runs.jsonl");
    let records: Vec<RunRecord> = text.lines().filter(|l| !l.trim().is_empty()).map(|l| serde_json::from_str(l).expect("record")).collect();
    let system: SystemInfo = serde_json::from_str(&std::fs::read_to_string(dir.join("system.json")).expect("system.json")).expect("system");
    let rows = summarize(&records);
    std::fs::write(dir.join("summary.csv"), summary_csv(&rows)).expect("csv");
    std::fs::write(dir.join("summary.md"), summary_markdown(&system, &rows, &records)).expect("md");
    println!("rewrote summary.csv and summary.md in {} ({} runs)", dir.display(), records.len());
}

/// Re-scores stored transcripts with the current scoring rules (no engine is run), keeping the
/// previous file as runs.scoring-v<N>.jsonl, then rebuilds the summaries.
fn rescore_dir(args: &Args) {
    let dir = PathBuf::from(args.get("dir").expect("--dir <results folder>"));
    let text = std::fs::read_to_string(dir.join("runs.jsonl")).expect("runs.jsonl");
    let mut records: Vec<RunRecord> =
        text.lines().filter(|l| !l.trim().is_empty()).map(|l| serde_json::from_str(l).expect("record")).collect();
    let old_version = records.first().map(|r| r.scoring_version).unwrap_or(1);
    let backup = dir.join(format!("runs.scoring-v{old_version}.jsonl"));
    if !backup.exists() {
        std::fs::write(&backup, &text).expect("backup");
    }
    let ds = Dataset::load(&default_root()).expect("dataset");
    let terms = ds.samples.iter().map(|s| (s.id.clone(), s.key_terms.clone())).collect();
    rescore(&mut records, &terms);
    let mut out = String::new();
    for r in &records {
        out.push_str(&serde_json::to_string(r).expect("json"));
        out.push('\n');
    }
    std::fs::write(dir.join("runs.jsonl"), out).expect("write runs");
    println!("re-scored {} runs (scoring v{old_version} -> current); previous scores kept in {}", records.len(), backup.display());
    resummarize(args);
}

fn read_records(dir: &std::path::Path) -> Vec<RunRecord> {
    let text = std::fs::read_to_string(dir.join("runs.jsonl")).unwrap_or_else(|e| panic!("{}: {e}", dir.join("runs.jsonl").display()));
    text.lines().filter(|l| !l.trim().is_empty()).map(|l| serde_json::from_str(l).expect("record")).collect()
}

fn dataset_key_terms() -> KeyTerms {
    let ds = Dataset::load(&default_root()).expect("dataset");
    ds.samples.iter().filter(|s| !s.private).map(|s| (s.id.clone(), s.key_terms.clone())).collect()
}

fn keep_categories(records: Vec<RunRecord>, categories: &[String]) -> Vec<RunRecord> {
    records.into_iter().filter(|r| categories.is_empty() || categories.contains(&r.category)).collect()
}

/// Key-term outcome of one run, or the comparison of two runs (fixes versus false corrections).
fn termstudy_cmd(args: &Args) {
    let base_dir = PathBuf::from(args.get("dir").expect("--dir <baseline results folder>"));
    let cats = args.list("category");
    let terms = dataset_key_terms();
    let base = keep_categories(read_records(&base_dir), &cats);
    let report = match args.get("against") {
        None => termstudy::baseline_markdown(&base, &terms),
        Some(other) => {
            let var = keep_categories(read_records(&PathBuf::from(other)), &cats);
            termstudy::comparison_markdown(&base, &var, &terms)
        }
    };
    println!("{report}");
    if let Some(out) = args.get("out") {
        std::fs::write(out, &report).expect("write report");
    }
}

/// Applies the dictionary post-correction to the stored transcripts of a run (no engine is run) and
/// writes a new results folder with the corrected texts, re-scored; `rawText` keeps the engine output.
fn postcorrect_cmd(args: &Args) {
    let src = PathBuf::from(args.get("dir").expect("--dir <results folder>"));
    let vocab = load_vocab_dir(&PathBuf::from(args.get("vocab-dir").expect("--vocab-dir <dir>")));
    let preset = args.get("preset").unwrap_or("strict");
    let cfg = PostCorrectConfig::by_name(preset).unwrap_or_else(|| panic!("unknown --preset {preset} (strict, medium, loose)"));
    let mut records = read_records(&src);
    let mut log = String::new();
    let mut changed = 0;
    for r in records.iter_mut().filter(|r| r.error.is_none()) {
        let terms = vocab.get(&r.language).map(Vec::as_slice).unwrap_or(&[]);
        let out = postcorrect::correct(&r.text, terms, &cfg);
        r.post_correction = preset.to_string();
        if out.changes.is_empty() {
            continue;
        }
        changed += 1;
        for c in &out.changes {
            log.push_str(&format!(
                "{}\n",
                serde_json::json!({"model": r.model_id, "decoding": r.decoding, "sample": r.sample_id, "from": c.from, "to": c.to, "distance": c.distance})
            ));
        }
        r.raw_text = Some(std::mem::replace(&mut r.text, out.text));
    }
    let ds = Dataset::load(&default_root()).expect("dataset");
    let key_terms = ds.samples.iter().map(|s| (s.id.clone(), s.key_terms.clone())).collect();
    rescore(&mut records, &key_terms);

    let started = now_ms();
    let label = args.get("label").unwrap_or("postcorrect");
    let run_id = format!("{}-{label}", utc_stamp(started));
    let out_dir = default_root().join("results").join(&run_id);
    let system: SystemInfo = serde_json::from_str(&std::fs::read_to_string(src.join("system.json")).expect("system.json")).expect("system");
    let mut config: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(src.join("config.json")).expect("config.json")).expect("config");
    config["runId"] = serde_json::json!(run_id);
    config["postCorrection"] = serde_json::json!({
        "preset": preset, "sourceRun": src.file_name().map(|n| n.to_string_lossy().into_owned()),
        "vocabularyTerms": vocab.iter().map(|(l, v)| (l.clone(), v.len())).collect::<std::collections::BTreeMap<_, _>>(),
        "note": "text in runs.jsonl is the corrected text, rawText the engine output; timings are those of the source run"
    });
    for r in records.iter_mut() {
        r.run_id = run_id.clone();
    }
    write_results(&out_dir, &system, &config, &records).expect("write results");
    std::fs::write(out_dir.join("post-corrections.jsonl"), log).expect("write log");
    println!("post-correction ({preset}): {changed} of {} transcripts changed; results in {}", records.len(), out_dir.display());
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(raw.get(1..).unwrap_or(&[]));
    match raw.first().map(String::as_str) {
        Some("check") => check(),
        Some("preflight") => {
            // Quiet-machine check only: exit code 0 = quiet, 2 = not quiet. Safe to run at any time.
            let pre = preflight(8000, MAX_BACKGROUND_CPU_PERCENT);
            println!("{}", serde_json::to_string_pretty(&pre).expect("json"));
            if pre.problems.is_empty() {
                println!("QUIET: the machine is suitable for a benchmark.");
            } else {
                for p in &pre.problems {
                    println!("NOT QUIET: {p}");
                }
                std::process::exit(2);
            }
        }
        Some("run") => run(&args),
        Some("run-one") => run_one(&args),
        Some("summarize") => resummarize(&args),
        Some("rescore") => rescore_dir(&args),
        Some("termstudy") => termstudy_cmd(&args),
        Some("postcorrect") => postcorrect_cmd(&args),
        _ => eprintln!("usage: bench check | bench run [--models a,b] [--beams 5,1] [--reps N] [--category c] [--limit N] [--label x] [--chunking vad] [--include-private] [--no-isolate]"),
    }
}
