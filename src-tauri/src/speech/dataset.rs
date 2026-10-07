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

pub fn load_scripts(root: &Path) -> Result<Vec<ScriptItem>, SpeechError> {
    let mut all = Vec::new();
    for (_, items) in read_json_files::<Vec<ScriptItem>>(&root.join("scripts"))? {
        all.extend(items);
    }
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
