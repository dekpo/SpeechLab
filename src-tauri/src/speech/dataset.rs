//! Benchmark dataset: scripts to read aloud, and samples (metadata + reference + audio file).
//!
//! Layout (default `<repo>/benchmark`, override with `SPEECHLAB_BENCH_DIR`):
//!   scripts/*.json   arrays of `ScriptItem`: the sentences to read (committed)
//!   samples/*.json   one `Sample` per file: metadata and reference transcript (committed)
//!   samples-private/*.json   same format for material that must never be committed
//!                    (third-party voices, private content); git-ignored
//!   audio/           the WAV files (git-ignored by default: they contain voices)
//!
//! New samples are added by dropping a JSON file and a WAV file; no code changes.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::critical::KeyTerm;
use super::error::SpeechError;
use super::wav;

pub const LANGUAGES: &[&str] = &["fr", "en"];
pub const DOMAINS: &[&str] = &["general", "medical", "administrative", "legal", "it"];
pub const UTTERANCE_TYPES: &[&str] = &["question", "statement", "dictation"];
pub const SOURCE_KINDS: &[&str] = &[
    "own-recording",
    "third-party-private",
    "public-domain",
    "licensed",
    "synthetic",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptItem {
    pub id: String,
    pub language: String,
    pub domain: String,
    pub utterance_type: String,
    /// The exact words to read; becomes the reference transcript.
    pub text: String,
    #[serde(default)]
    pub key_terms: Vec<KeyTerm>,
    /// Free tag for the evaluation category, e.g. "fr-with-english-terms".
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Speaker {
    pub id: String,
    /// Honest short description, e.g. "native French, no marked accent". Never generalised.
    pub profile: String,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    /// One of `SOURCE_KINDS`.
    pub kind: String,
    #[serde(default)]
    pub url: Option<String>,
    /// SPDX id or short statement ("own recording, owner consent"). Required.
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub id: String,
    pub language: String,
    pub domain: String,
    pub utterance_type: String,
    #[serde(default)]
    pub category: String,
    pub speaker: Speaker,
    /// File name inside `audio/`.
    pub audio_file: String,
    /// Optional here; measured from the WAV when the dataset is loaded.
    #[serde(default)]
    pub duration_ms: Option<u64>,
    pub reference: String,
    #[serde(default)]
    pub key_terms: Vec<KeyTerm>,
    pub source: SourceInfo,
    /// May the audio file be committed to git / shared? Defaults to false.
    #[serde(default)]
    pub committable: bool,
    #[serde(default)]
    pub notes: String,
    /// Loaded from `samples-private/` (set by the loader, never read from the file).
    #[serde(skip)]
    pub private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub sample_id: String,
    pub message: String,
}

pub struct Dataset {
    pub root: PathBuf,
    pub samples: Vec<Sample>,
    pub issues: Vec<ValidationIssue>,
}

/// `SPEECHLAB_BENCH_DIR`, else `<repo>/benchmark` next to `src-tauri`.
pub fn default_root() -> PathBuf {
    std::env::var_os("SPEECHLAB_BENCH_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("benchmark"))
}

fn read_json_files<T: for<'de> Deserialize<'de>>(dir: &Path) -> Result<Vec<(PathBuf, T)>, SpeechError> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for p in paths {
        let text = fs::read_to_string(&p).map_err(|e| SpeechError::Engine(format!("{}: {e}", p.display())))?;
        let value: T = serde_json::from_str(&text)
            .map_err(|e| SpeechError::Engine(format!("{}: invalid JSON: {e}", p.display())))?;
        out.push((p, value));
    }
    Ok(out)
}

/// Reading order = the owner's priorities: short general sentences, then professional
/// vocabulary, then IT vocabulary, then English, then long dictations (D-025). Unknown
/// categories come last, in file order.
const CATEGORY_ORDER: &[&str] = &[
    "fr-general",
    "fr-medical",
    "fr-administrative",
    "fr-legal",
    "fr-with-english-terms",
    "en-general",
    "en-technical",
    "fr-dictation",
    "en-dictation",
];

fn category_rank(category: &str) -> usize {
    CATEGORY_ORDER.iter().position(|c| *c == category).unwrap_or(CATEGORY_ORDER.len())
}

pub fn load_scripts(root: &Path) -> Result<Vec<ScriptItem>, SpeechError> {
    let mut all = Vec::new();
    for (_, items) in read_json_files::<Vec<ScriptItem>>(&root.join("scripts"))? {
        all.extend(items);
    }
    // Stable sort: ids inside a category keep their file order.
    all.sort_by_key(|i| category_rank(&i.category));
    Ok(all)
}

fn check_choice(issues: &mut Vec<ValidationIssue>, id: &str, what: &str, value: &str, allowed: &[&str]) {
    if !allowed.contains(&value) {
        issues.push(ValidationIssue {
            sample_id: id.to_string(),
            message: format!("{what} '{value}' is not one of {allowed:?}"),
        });
    }
}

/// Lower-case ASCII id fragment: letters, digits, `-` and `_` only (safe in file names).
pub fn sanitize_id(raw: &str) -> String {
    let cleaned: String = raw
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '-' })
        .collect();
    let collapsed = cleaned.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    collapsed.chars().take(60).collect()
}

/// Writes the audio and the metadata of one recorded sample and returns it. Recording the same
/// sentence again by the same speaker replaces the previous sample. Third-party private material
/// is always stored in `samples-private/` and is never committable.
pub fn create_sample(
    root: &Path,
    script: &ScriptItem,
    speaker: Speaker,
    source: SourceInfo,
    private: bool,
    wav_bytes: &[u8],
) -> Result<Sample, SpeechError> {
    let speaker_id = sanitize_id(&speaker.id);
    if speaker_id.is_empty() {
        return Err(SpeechError::InvalidRequest("speaker id is empty".into()));
    }
    if source.license.trim().is_empty() {
        return Err(SpeechError::InvalidRequest("a licence or consent statement is required".into()));
    }
    if !SOURCE_KINDS.contains(&source.kind.as_str()) {
        return Err(SpeechError::InvalidRequest(format!("unknown source kind '{}'", source.kind)));
    }
    let private = private || source.kind == "third-party-private";
    let id = format!("{}-{}", script.id, speaker_id);
    let audio_file = format!("{id}.wav");
    let audio_dir = root.join("audio");
    fs::create_dir_all(&audio_dir).map_err(|e| SpeechError::Audio(e.to_string()))?;
    let audio_path = audio_dir.join(&audio_file);
    fs::write(&audio_path, wav_bytes).map_err(|e| SpeechError::Audio(e.to_string()))?;
    let measured = match wav::read_wav_mono(&audio_path) {
        Ok(a) => a,
        Err(e) => {
            let _ = fs::remove_file(&audio_path);
            return Err(e);
        }
    };

    let sample = Sample {
        id: id.clone(),
        language: script.language.clone(),
        domain: script.domain.clone(),
        utterance_type: script.utterance_type.clone(),
        category: script.category.clone(),
        speaker: Speaker { id: speaker_id, ..speaker },
        audio_file,
        duration_ms: Some(measured.duration_ms()),
        reference: script.text.clone(),
        key_terms: script.key_terms.clone(),
        source,
        committable: false,
        notes: String::new(),
        private,
    };
    let (dir, other) = if private { ("samples-private", "samples") } else { ("samples", "samples-private") };
    fs::create_dir_all(root.join(dir)).map_err(|e| SpeechError::Audio(e.to_string()))?;
    let json = serde_json::to_string_pretty(&sample).map_err(|e| SpeechError::Engine(e.to_string()))?;
    fs::write(root.join(dir).join(format!("{id}.json")), json + "\n")
        .map_err(|e| SpeechError::Audio(e.to_string()))?;
    let _ = fs::remove_file(root.join(other).join(format!("{id}.json")));
    Ok(sample)
}

/// Everything needed to turn an existing audio file and a typed reference into a dataset sample.
#[derive(Debug, Clone)]
pub struct ImportRequest {
    /// Sample id, already in the `sanitize_id` form (it names the files).
    pub id: String,
    pub wav_path: PathBuf,
    /// The owner's verbatim text. Never printed or logged by the import.
    pub reference: String,
    pub language: String,
    pub domain: String,
    pub utterance_type: String,
    pub category: String,
    pub speaker: Speaker,
    pub source: SourceInfo,
    pub private: bool,
    /// Overwrite a sample with the same id instead of refusing.
    pub replace: bool,
    pub notes: String,
}

/// Licence statement stored with a private third-party clip. A refusal ("no") is an error: such a
/// clip must not enter the dataset; "unknown" is allowed but written down as such.
pub fn private_voice_license(consent: &str) -> Result<String, SpeechError> {
    match consent.trim().to_lowercase().as_str() {
        "yes" | "oui" => Ok("private voice, no redistribution, speaker consent: yes".into()),
        "unknown" | "inconnu" => Ok("private voice, no redistribution, speaker consent: unknown".into()),
        "no" | "non" => Err(SpeechError::InvalidRequest("the speaker did not consent: the clip must not be imported".into())),
        other => Err(SpeechError::InvalidRequest(format!("consent must be yes, no or unknown, got '{other}'"))),
    }
}

/// Reference text as typed in a file: BOM and line breaks removed, runs of spaces collapsed.
pub fn clean_reference(raw: &str) -> String {
    raw.trim_start_matches('\u{feff}').split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Imports an existing audio file (any PCM WAV; stereo is mixed down, other rates are resampled to
/// 16 kHz by linear interpolation) with its reference text. Writes `audio/<id>.wav` (16 kHz mono
/// 16-bit) and the metadata JSON (`samples-private/` for private or third-party material), then
/// validates the result with the normal dataset checks and rolls back if any check fails. Error
/// messages never contain the reference text.
pub fn import_sample(root: &Path, req: &ImportRequest) -> Result<Sample, SpeechError> {
    let bad = |m: String| SpeechError::InvalidRequest(m);
    let id = sanitize_id(&req.id);
    if id.is_empty() || id != req.id {
        return Err(bad(format!("sample id must be lower-case letters, digits, '-' or '_' (got '{}')", req.id)));
    }
    let speaker_id = sanitize_id(&req.speaker.id);
    if speaker_id.is_empty() {
        return Err(bad("speaker id is empty".into()));
    }
    if req.speaker.profile.trim().is_empty() {
        return Err(bad("a speaker description is required (for example 'one speaker, accent as described by the owner')".into()));
    }
    if req.source.license.trim().is_empty() {
        return Err(bad("a licence or consent statement is required".into()));
    }
    let reference = clean_reference(&req.reference);
    if reference.is_empty() {
        return Err(bad("the reference text is empty".into()));
    }
    let private = req.private || req.source.kind == "third-party-private";
    if !private
        && matches!(req.source.kind.as_str(), "public-domain" | "licensed")
        && req.source.url.as_deref().map_or(true, |u| u.trim().is_empty())
    {
        return Err(bad("public or licensed material needs a source URL".into()));
    }
    let mut issues = Vec::new();
    check_choice(&mut issues, &id, "language", &req.language, LANGUAGES);
    check_choice(&mut issues, &id, "domain", &req.domain, DOMAINS);
    check_choice(&mut issues, &id, "utteranceType", &req.utterance_type, UTTERANCE_TYPES);
    check_choice(&mut issues, &id, "source.kind", &req.source.kind, SOURCE_KINDS);
    if let Some(i) = issues.first() {
        return Err(bad(i.message.clone()));
    }

    let audio_path = root.join("audio").join(format!("{id}.wav"));
    let json_paths = [root.join("samples").join(format!("{id}.json")), root.join("samples-private").join(format!("{id}.json"))];
    if !req.replace && (audio_path.exists() || json_paths.iter().any(|p| p.exists())) {
        return Err(bad(format!("sample '{id}' already exists (use replace to overwrite it)")));
    }

    // Decode and convert in memory first, so that a bad file leaves nothing behind.
    let decoded = wav::read_wav_mono(&req.wav_path)?;
    let samples = super::chunking::to_chunk_rate(&decoded.samples, decoded.sample_rate);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: super::chunking::CHUNK_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let audio_err = |e: hound::Error| SpeechError::Audio(format!("cannot write {}: {e}", audio_path.display()));
    fs::create_dir_all(root.join("audio")).map_err(|e| SpeechError::Audio(e.to_string()))?;
    let mut w = hound::WavWriter::create(&audio_path, spec).map_err(audio_err)?;
    for s in &samples {
        w.write_sample((s.clamp(-1.0, 1.0) * 32767.0).round() as i16).map_err(audio_err)?;
    }
    w.finalize().map_err(audio_err)?;

    let sample = Sample {
        id: id.clone(),
        language: req.language.clone(),
        domain: req.domain.clone(),
        utterance_type: req.utterance_type.clone(),
        category: req.category.clone(),
        speaker: Speaker { id: speaker_id, ..req.speaker.clone() },
        audio_file: format!("{id}.wav"),
        duration_ms: Some(samples.len() as u64 * 1000 / super::chunking::CHUNK_RATE as u64),
        reference,
        key_terms: Vec::new(),
        source: req.source.clone(),
        committable: false,
        notes: req.notes.clone(),
        private,
    };
    let (dir, other) = if private { (1, 0) } else { (0, 1) };
    let write_meta = || -> Result<(), SpeechError> {
        let folder = json_paths[dir].parent().expect("parent");
        fs::create_dir_all(folder).map_err(|e| SpeechError::Audio(e.to_string()))?;
        let json = serde_json::to_string_pretty(&sample).map_err(|e| SpeechError::Engine(e.to_string()))?;
        fs::write(&json_paths[dir], json + "\n").map_err(|e| SpeechError::Audio(e.to_string()))?;
        let _ = fs::remove_file(&json_paths[other]);
        Ok(())
    };
    write_meta()?;

    // Normal dataset checks; undo everything if this sample does not pass.
    let problems: Vec<String> = match Dataset::load(root) {
        Ok(ds) => ds.issues.iter().filter(|i| i.sample_id == id).map(|i| i.message.clone()).collect(),
        Err(e) => vec![e.to_string()],
    };
    if !problems.is_empty() {
        let _ = delete_sample(root, &id);
        return Err(bad(format!("imported sample '{id}' failed validation: {}", problems.join("; "))));
    }
    Ok(sample)
}

/// Removes a sample's metadata (both folders) and its audio.
pub fn delete_sample(root: &Path, sample_id: &str) -> Result<(), SpeechError> {
    let id = sanitize_id(sample_id);
    if id.is_empty() || id != sample_id {
        return Err(SpeechError::InvalidRequest("invalid sample id".into()));
    }
    let mut removed = false;
    for dir in ["samples", "samples-private"] {
        removed |= fs::remove_file(root.join(dir).join(format!("{id}.json"))).is_ok();
    }
    removed |= fs::remove_file(root.join("audio").join(format!("{id}.wav"))).is_ok();
    if removed {
        Ok(())
    } else {
        Err(SpeechError::InvalidRequest(format!("sample not found: {id}")))
    }
}

impl Dataset {
    pub fn load(root: &Path) -> Result<Dataset, SpeechError> {
        let mut samples: Vec<Sample> = Vec::new();
        let mut issues = Vec::new();
        for (dir, private) in [("samples", false), ("samples-private", true)] {
            for (_, mut sample) in read_json_files::<Sample>(&root.join(dir))? {
                sample.private = private;
                samples.push(sample);
            }
        }

        let mut seen = HashSet::new();
        for s in &mut samples {
            let id = s.id.clone();
            if !seen.insert(id.clone()) {
                issues.push(ValidationIssue { sample_id: id.clone(), message: "duplicate id".into() });
            }
            check_choice(&mut issues, &id, "language", &s.language, LANGUAGES);
            check_choice(&mut issues, &id, "domain", &s.domain, DOMAINS);
            check_choice(&mut issues, &id, "utteranceType", &s.utterance_type, UTTERANCE_TYPES);
            check_choice(&mut issues, &id, "source.kind", &s.source.kind, SOURCE_KINDS);
            if s.reference.trim().is_empty() {
                issues.push(ValidationIssue { sample_id: id.clone(), message: "empty reference".into() });
            }
            if s.source.license.trim().is_empty() {
                issues.push(ValidationIssue { sample_id: id.clone(), message: "source.license is required".into() });
            }
            if s.source.kind == "third-party-private" && !s.private {
                issues.push(ValidationIssue {
                    sample_id: id.clone(),
                    message: "third-party private material must live in samples-private/".into(),
                });
            }
            if s.committable && s.source.kind == "third-party-private" {
                issues.push(ValidationIssue {
                    sample_id: id.clone(),
                    message: "private third-party audio must not be committable".into(),
                });
            }
            let audio = root.join("audio").join(&s.audio_file);
            if audio.is_file() {
                match wav::read_wav_mono(&audio) {
                    Ok(a) => s.duration_ms = Some(a.duration_ms()),
                    Err(e) => issues.push(ValidationIssue { sample_id: id.clone(), message: e.to_string() }),
                }
            } else {
                issues.push(ValidationIssue {
                    sample_id: id.clone(),
                    message: format!("audio file missing: {}", audio.display()),
                });
            }
        }
        Ok(Dataset { root: root.to_path_buf(), samples, issues })
    }

    pub fn audio_path(&self, sample: &Sample) -> PathBuf {
        self.root.join("audio").join(&sample.audio_file)
    }

    /// Samples whose audio is present and valid (what a benchmark run can actually use).
    pub fn runnable(&self) -> Vec<&Sample> {
        let bad: HashSet<&str> = self
            .issues
            .iter()
            .filter(|i| i.message.contains("audio") || i.message.contains("WAV") || i.message.contains("duplicate"))
            .map(|i| i.sample_id.as_str())
            .collect();
        self.samples.iter().filter(|s| !bad.contains(s.id.as_str())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("speechlab_ds_{name}"));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("samples")).unwrap();
        fs::create_dir_all(d.join("samples-private")).unwrap();
        fs::create_dir_all(d.join("audio")).unwrap();
        fs::create_dir_all(d.join("scripts")).unwrap();
        d
    }

    fn write_wav(path: &Path) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for _ in 0..16000 {
            w.write_sample(0i16).unwrap();
        }
        w.finalize().unwrap();
    }

    fn sample_json(id: &str, kind: &str, committable: bool, reference: &str) -> String {
        format!(
            r#"{{"id":"{id}","language":"fr","domain":"general","utteranceType":"question",
            "speaker":{{"id":"owner","profile":"native French, no marked accent"}},
            "audioFile":"{id}.wav","reference":"{reference}",
            "source":{{"kind":"{kind}","license":"own recording, owner consent"}},
            "committable":{committable}}}"#
        )
    }

    #[test]
    fn loads_a_valid_sample_and_measures_duration() {
        let d = tmp("ok");
        fs::write(d.join("samples/a.json"), sample_json("a", "own-recording", false, "Bonjour ?")).unwrap();
        write_wav(&d.join("audio/a.wav"));
        let ds = Dataset::load(&d).unwrap();
        assert!(ds.issues.is_empty(), "{:?}", ds.issues);
        assert_eq!(ds.samples[0].duration_ms, Some(1000));
        assert_eq!(ds.runnable().len(), 1);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn reports_missing_audio_duplicates_and_policy_violations() {
        let d = tmp("bad");
        fs::write(d.join("samples/a.json"), sample_json("a", "own-recording", false, "x")).unwrap();
        fs::write(d.join("samples/b.json"), sample_json("a", "own-recording", false, "y")).unwrap();
        fs::write(d.join("samples/c.json"), sample_json("c", "third-party-private", true, "")).unwrap();
        write_wav(&d.join("audio/c.wav"));
        let ds = Dataset::load(&d).unwrap();
        let msgs: Vec<String> = ds.issues.iter().map(|i| i.message.clone()).collect();
        assert!(msgs.iter().any(|m| m.contains("duplicate id")));
        assert!(msgs.iter().any(|m| m.contains("audio file missing")));
        assert!(msgs.iter().any(|m| m.contains("empty reference")));
        assert!(msgs.iter().any(|m| m.contains("must not be committable")));
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn private_samples_are_loaded_flagged_and_misplaced_ones_are_reported() {
        let d = tmp("private");
        fs::write(d.join("samples-private/p.json"), sample_json("p", "third-party-private", false, "secret")).unwrap();
        fs::write(d.join("samples/q.json"), sample_json("q", "third-party-private", false, "oops")).unwrap();
        write_wav(&d.join("audio/p.wav"));
        write_wav(&d.join("audio/q.wav"));
        let ds = Dataset::load(&d).unwrap();
        assert!(ds.samples.iter().find(|s| s.id == "p").unwrap().private);
        let msgs: Vec<String> = ds.issues.iter().map(|i| format!("{}: {}", i.sample_id, i.message)).collect();
        assert!(!msgs.iter().any(|m| m.starts_with("p:")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.starts_with("q:") && m.contains("samples-private")), "{msgs:?}");
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn invalid_json_is_an_error_naming_the_file() {
        let d = tmp("json");
        fs::write(d.join("samples/broken.json"), "{not json").unwrap();
        let err = Dataset::load(&d).err().unwrap().to_string();
        assert!(err.contains("broken.json"));
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn an_empty_or_missing_dataset_is_fine() {
        let ds = Dataset::load(&std::env::temp_dir().join("speechlab_ds_does_not_exist")).unwrap();
        assert!(ds.samples.is_empty());
    }

    #[test]
    fn bundled_scripts_are_valid_and_self_consistent() {
        let items = load_scripts(&default_root()).expect("scripts load");
        assert!(items.len() >= 90, "expected the full script set, got {}", items.len());
        let mut ids = HashSet::new();
        for it in &items {
            assert!(ids.insert(it.id.clone()), "duplicate script id {}", it.id);
            assert!(LANGUAGES.contains(&it.language.as_str()), "{}", it.id);
            assert!(DOMAINS.contains(&it.domain.as_str()), "{}", it.id);
            assert!(UTTERANCE_TYPES.contains(&it.utterance_type.as_str()), "{}", it.id);
            assert!(!it.text.trim().is_empty(), "{}", it.id);
            // A transcript equal to the reference must raise no flag; otherwise a key term
            // is not literally present in its own sentence and the benchmark would be wrong.
            let report = crate::speech::critical::analyze(&it.text, &it.text, &it.key_terms);
            assert!(report.flags.is_empty(), "{}: {:?}", it.id, report.flags);
        }
        for lang in LANGUAGES {
            assert!(items.iter().any(|i| i.language == *lang));
        }
        // Reading order follows the owner's priorities: general sentences first, dictations last.
        assert_eq!(items[0].category, "fr-general");
        assert_eq!(items[0].id, "fr-gen-q-01");
        assert!(items.last().unwrap().category.ends_with("dictation"));
        let first_pro = items.iter().position(|i| i.category == "fr-medical").unwrap();
        let first_it = items.iter().position(|i| i.category == "fr-with-english-terms").unwrap();
        assert!(first_pro < first_it);
    }

    fn script(id: &str) -> ScriptItem {
        ScriptItem {
            id: id.into(),
            language: "fr".into(),
            domain: "medical".into(),
            utterance_type: "statement".into(),
            text: "Le patient prend 500 mg d'amoxicilline.".into(),
            key_terms: vec![KeyTerm { text: "amoxicilline".into(), kind: "drug".into() }],
            category: "fr-medical".into(),
        }
    }

    fn wav_bytes_1s() -> Vec<u8> {
        let mut cur = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::new(&mut cur, spec).unwrap();
        for _ in 0..16000 {
            w.write_sample(100i16).unwrap();
        }
        w.finalize().unwrap();
        cur.into_inner()
    }

    fn speaker(id: &str) -> Speaker {
        Speaker { id: id.into(), profile: "native French, no marked accent".into(), gender: None, accent: None }
    }

    fn own_source() -> SourceInfo {
        SourceInfo { kind: "own-recording".into(), url: None, license: "own recording, owner consent".into() }
    }

    #[test]
    fn sanitize_id_is_filesystem_safe() {
        assert_eq!(sanitize_id("Owner 2"), "owner-2");
        assert_eq!(sanitize_id("../../etc/passwd"), "etc-passwd");
        assert_eq!(sanitize_id("  Owner_1 "), "owner_1");
        assert_eq!(sanitize_id("///"), "");
    }

    #[test]
    fn create_sample_writes_audio_and_metadata_and_roundtrips_through_load() {
        let d = tmp("create");
        let s = create_sample(&d, &script("fr-med-01"), speaker("Owner"), own_source(), false, &wav_bytes_1s()).unwrap();
        assert_eq!(s.id, "fr-med-01-owner");
        assert_eq!(s.duration_ms, Some(1000));
        let ds = Dataset::load(&d).unwrap();
        assert!(ds.issues.is_empty(), "{:?}", ds.issues);
        assert_eq!(ds.samples.len(), 1);
        assert_eq!(ds.samples[0].reference, "Le patient prend 500 mg d'amoxicilline.");
        assert_eq!(ds.samples[0].key_terms[0].text, "amoxicilline");
        assert!(!ds.samples[0].committable && !ds.samples[0].private);
        // Recording again replaces the sample instead of duplicating it.
        create_sample(&d, &script("fr-med-01"), speaker("owner"), own_source(), false, &wav_bytes_1s()).unwrap();
        assert_eq!(Dataset::load(&d).unwrap().samples.len(), 1);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn third_party_material_is_forced_into_the_private_folder() {
        let d = tmp("forced");
        let src = SourceInfo { kind: "third-party-private".into(), url: None, license: "private voice note, no redistribution".into() };
        let s = create_sample(&d, &script("fr-med-01"), speaker("guest"), src, false, &wav_bytes_1s()).unwrap();
        assert!(s.private);
        assert!(d.join("samples-private/fr-med-01-guest.json").is_file());
        assert!(!d.join("samples/fr-med-01-guest.json").exists());
        let ds = Dataset::load(&d).unwrap();
        assert!(ds.issues.is_empty(), "{:?}", ds.issues);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn create_sample_rejects_bad_input_and_leaves_no_audio_behind() {
        let d = tmp("reject");
        assert!(create_sample(&d, &script("a"), speaker("///"), own_source(), false, &wav_bytes_1s()).is_err());
        let no_licence = SourceInfo { kind: "own-recording".into(), url: None, license: " ".into() };
        assert!(create_sample(&d, &script("a"), speaker("x"), no_licence, false, &wav_bytes_1s()).is_err());
        assert!(create_sample(&d, &script("a"), speaker("x"), own_source(), false, b"not a wav").is_err());
        assert!(!d.join("audio/a-x.wav").exists(), "invalid audio must be removed");
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn delete_sample_removes_everything_and_rejects_path_tricks() {
        let d = tmp("delete");
        create_sample(&d, &script("fr-med-01"), speaker("owner"), own_source(), false, &wav_bytes_1s()).unwrap();
        assert!(delete_sample(&d, "../samples/fr-med-01-owner").is_err());
        delete_sample(&d, "fr-med-01-owner").unwrap();
        assert!(!d.join("audio/fr-med-01-owner.wav").exists());
        assert!(!d.join("samples/fr-med-01-owner.json").exists());
        assert!(delete_sample(&d, "fr-med-01-owner").is_err(), "second delete reports not found");
        let _ = fs::remove_dir_all(d);
    }

    /// Synthetic stereo tone at an arbitrary rate (no voice involved).
    fn write_tone(path: &Path, channels: u16, rate: u32, seconds: u32) {
        let spec = hound::WavSpec { channels, sample_rate: rate, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for i in 0..rate * seconds {
            let v = ((i as f32 * 0.05).sin() * 8000.0) as i16;
            for _ in 0..channels {
                w.write_sample(v).unwrap();
            }
        }
        w.finalize().unwrap();
    }

    fn import_request(src: &Path, id: &str) -> ImportRequest {
        ImportRequest {
            id: id.into(),
            wav_path: src.to_path_buf(),
            reference: "\u{feff}Premier mot  deuxième mot\r\nsuite du texte.\n".into(),
            language: "fr".into(),
            domain: "general".into(),
            utterance_type: "dictation".into(),
            category: "fr-accent-private".into(),
            speaker: Speaker { id: "Spk A".into(), profile: "one speaker, accent as described by the owner".into(), gender: None, accent: Some("test".into()) },
            source: SourceInfo { kind: "third-party-private".into(), url: None, license: private_voice_license("yes").unwrap() },
            private: true,
            replace: false,
            notes: String::new(),
        }
    }

    #[test]
    fn import_converts_audio_to_16k_mono_and_stores_it_as_private() {
        let d = tmp("import");
        let src = d.join("input-8k-stereo.wav");
        write_tone(&src, 2, 8000, 3);
        let s = import_sample(&d, &import_request(&src, "acc-01")).unwrap();
        assert!(s.private && !s.committable);
        assert_eq!(s.speaker.id, "spk-a");
        assert_eq!(s.reference, "Premier mot deuxième mot suite du texte.", "BOM and line breaks are cleaned");
        assert!(d.join("samples-private/acc-01.json").is_file());
        assert!(!d.join("samples/acc-01.json").exists());
        let out = hound::WavReader::open(d.join("audio/acc-01.wav")).unwrap().spec();
        assert_eq!((out.channels, out.sample_rate, out.bits_per_sample), (1, 16000, 16));
        let ds = Dataset::load(&d).unwrap();
        assert!(ds.issues.is_empty(), "{:?}", ds.issues);
        assert_eq!(ds.samples[0].duration_ms, Some(3000));
        assert!(ds.samples[0].private);
        assert_eq!(ds.runnable().len(), 1);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn import_refuses_an_existing_id_unless_replace_is_set() {
        let d = tmp("import_dup");
        let src = d.join("in.wav");
        write_tone(&src, 1, 16000, 1);
        import_sample(&d, &import_request(&src, "acc-01")).unwrap();
        assert!(import_sample(&d, &import_request(&src, "acc-01")).is_err());
        let mut again = import_request(&src, "acc-01");
        again.replace = true;
        import_sample(&d, &again).unwrap();
        assert_eq!(Dataset::load(&d).unwrap().samples.len(), 1);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn import_rejects_bad_input_leaves_nothing_behind_and_never_echoes_the_reference() {
        let d = tmp("import_bad");
        let good = d.join("in.wav");
        write_tone(&good, 1, 16000, 1);
        let secret = "texte confidentiel unique";

        let mut r = import_request(&good, "Bad Id");
        assert!(import_sample(&d, &r).is_err(), "id must already be sanitised");
        r = import_request(&good, "acc-02");
        r.reference = "  \n ".into();
        assert!(import_sample(&d, &r).is_err());
        r = import_request(&good, "acc-02");
        r.speaker.profile = " ".into();
        assert!(import_sample(&d, &r).is_err());
        r = import_request(&good, "acc-02");
        r.language = "de".into();
        assert!(import_sample(&d, &r).is_err());
        r = import_request(&good, "acc-02");
        r.private = false;
        r.source = SourceInfo { kind: "licensed".into(), url: None, license: "CC-BY-4.0".into() };
        assert!(import_sample(&d, &r).is_err(), "public material needs a URL");

        let fake = d.join("fake.wav");
        fs::write(&fake, b"not a wav at all, definitely").unwrap();
        r = import_request(&fake, "acc-03");
        r.reference = secret.into();
        let err = import_sample(&d, &r).err().unwrap().to_string();
        assert!(!err.contains(secret), "{err}");
        assert!(!d.join("audio/acc-03.wav").exists());
        assert!(!d.join("samples-private/acc-03.json").exists());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn import_of_public_material_goes_to_the_committed_folder_with_its_source() {
        let d = tmp("import_public");
        let src = d.join("in.wav");
        write_tone(&src, 1, 16000, 1);
        let mut r = import_request(&src, "pub-01");
        r.private = false;
        r.source = SourceInfo { kind: "public-domain".into(), url: Some("https://example.org/clip".into()), license: "public domain".into() };
        let s = import_sample(&d, &r).unwrap();
        assert!(!s.private);
        assert!(d.join("samples/pub-01.json").is_file());
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn consent_statement_is_explicit_and_a_refusal_blocks_the_import() {
        assert!(private_voice_license("yes").unwrap().contains("consent: yes"));
        assert!(private_voice_license("Unknown").unwrap().contains("consent: unknown"));
        assert!(private_voice_license("no").is_err());
        assert!(private_voice_license("maybe").is_err());
    }

    #[test]
    fn scripts_load_from_arrays() {
        let d = tmp("scripts");
        fs::write(
            d.join("scripts/s.json"),
            r#"[{"id":"s1","language":"fr","domain":"general","utteranceType":"question","text":"Quelle heure est-il ?"}]"#,
        )
        .unwrap();
        let items = load_scripts(&d).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].key_terms.is_empty());
        let _ = fs::remove_dir_all(d);
    }
}
