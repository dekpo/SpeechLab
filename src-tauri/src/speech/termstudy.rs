//! Key-term study: how many dataset key terms (drug names, technical terms...) each engine gets
//! right, and what a technique (biasing, post-correction) fixes or breaks (T4, D-036).
//!
//! Everything here works on STORED run records (`runs.jsonl`): no engine is run. The word-level
//! comparison between a baseline run and a variant run answers two questions:
//!   - FIXES: reference words that were wrong in the baseline and are right in the variant;
//!   - FALSE CORRECTIONS (regressions): reference words that were right in the baseline and are
//!     wrong in the variant. A technique that fixes three drug names and breaks five other
//!     words is a regression, so both numbers are always reported together.
//!
//! Word status comes from the same alignment as the WER (`metrics::compare_texts`), so the
//! numbers are consistent with the summaries. A reference word can flip because the alignment
//! of a nearby error changed; the examples are listed so a human can judge.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::benchmark::RunRecord;
use super::critical::{self, KeyTerm};
use super::metrics::{self, DiffKind};

/// Key terms of every sample, by sample id.
pub type KeyTerms = BTreeMap<String, Vec<KeyTerm>>;

/// (occurrences, missed) per key-term kind.
pub type KindCounts = BTreeMap<String, (usize, usize)>;

/// One engine configuration inside a run folder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ConfigKey {
    pub model_id: String,
    pub decoding: String,
}

impl ConfigKey {
    pub fn of(r: &RunRecord) -> Self {
        Self { model_id: r.model_id.clone(), decoding: r.decoding.clone() }
    }
    pub fn label(&self) -> String {
        format!("{} ({})", self.model_id, self.decoding)
    }
}

/// For each key term of the sample: (kind, text, found in the transcript).
pub fn term_outcomes(reference: &str, text: &str, terms: &[KeyTerm]) -> Vec<(String, String, bool)> {
    let flags = critical::analyze(reference, text, terms).flags;
    terms
        .iter()
        .map(|t| {
            let missed = flags
                .iter()
                .any(|f| f.kind == critical::FlagKind::KeyTerm && f.expected == t.text);
            (t.kind.clone(), t.text.clone(), !missed)
        })
        .collect()
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ConfigSummary {
    pub samples: usize,
    pub reference_words: usize,
    pub word_errors: usize,
    pub kinds: KindCounts,
    /// Samples with at least one critical flag (numbers, units, negations, dates, drug names).
    pub critical_samples: usize,
}

impl ConfigSummary {
    pub fn wer(&self) -> f64 {
        if self.reference_words == 0 {
            0.0
        } else {
            self.word_errors as f64 / self.reference_words as f64
        }
    }
    pub fn kind(&self, kind: &str) -> (usize, usize) {
        self.kinds.get(kind).copied().unwrap_or((0, 0))
    }
}

/// Summarises the records of one configuration. Errored records are skipped.
pub fn summarize_config(records: &[&RunRecord], terms: &KeyTerms) -> ConfigSummary {
    let mut s = ConfigSummary::default();
    for r in records.iter().filter(|r| r.error.is_none()) {
        s.samples += 1;
        s.reference_words += r.reference_words;
        s.word_errors += r.substitutions + r.deletions + r.insertions;
        s.critical_samples += usize::from(r.has_critical);
        let sample_terms = terms.get(&r.sample_id).map(Vec::as_slice).unwrap_or(&[]);
        for (kind, _, hit) in term_outcomes(&r.reference, &r.text, sample_terms) {
            let e = s.kinds.entry(kind).or_default();
            e.0 += 1;
            e.1 += usize::from(!hit);
        }
    }
    s
}

pub fn group_by_config(records: &[RunRecord]) -> BTreeMap<ConfigKey, Vec<&RunRecord>> {
    let mut m: BTreeMap<ConfigKey, Vec<&RunRecord>> = BTreeMap::new();
    for r in records {
        m.entry(ConfigKey::of(r)).or_default().push(r);
    }
    m
}

// ------------------------------------------------------------------ word-level comparison

/// One reference word whose status changed between the baseline and the variant.
#[derive(Debug, Clone, Serialize)]
pub struct WordChange {
    pub sample_id: String,
    pub reference_word: String,
    /// What the baseline produced for this word ("(missing)" if deleted).
    pub baseline: String,
    pub variant: String,
    /// The word belongs to a key term of the sample.
    pub in_key_term: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Delta {
    pub fixes: Vec<WordChange>,
    pub regressions: Vec<WordChange>,
    /// Words added by the variant that are not aligned with any reference word, minus the same
    /// for the baseline (negative = fewer extra words).
    pub insertions_delta: i64,
    pub pairs: usize,
}

struct WordStatus {
    word: String,
    correct: bool,
    hyp: String,
}

fn word_statuses(reference: &str, text: &str) -> Vec<WordStatus> {
    metrics::compare_texts(reference, text)
        .diff
        .into_iter()
        .filter_map(|op| {
            let word = op.reference?;
            Some(WordStatus {
                correct: op.kind == DiffKind::Equal,
                hyp: op.hypothesis.unwrap_or_else(|| "(missing)".into()),
                word,
            })
        })
        .collect()
}

fn key_words(terms: &[KeyTerm]) -> BTreeSet<String> {
    terms
        .iter()
        .flat_map(|t| metrics::normalize_words(&t.text))
        .flat_map(|w| w.split('\'').map(str::to_string).collect::<Vec<_>>())
        .collect()
}

/// Compares two records of the same sample (same engine configuration).
pub fn compare_pair(base: &RunRecord, var: &RunRecord, terms: &[KeyTerm], delta: &mut Delta) {
    let b = word_statuses(&base.reference, &base.text);
    let v = word_statuses(&var.reference, &var.text);
    delta.pairs += 1;
    let keys = key_words(terms);
    if b.len() == v.len() {
        for (x, y) in b.iter().zip(&v) {
            if x.correct == y.correct {
                continue;
            }
            let change = WordChange {
                sample_id: base.sample_id.clone(),
                reference_word: x.word.clone(),
                baseline: x.hyp.clone(),
                variant: y.hyp.clone(),
                in_key_term: x.word.split('\'').any(|part| keys.contains(part)),
            };
            if y.correct {
                delta.fixes.push(change);
            } else {
                delta.regressions.push(change);
            }
        }
    }
    let ins = |r: &RunRecord| r.insertions as i64;
    delta.insertions_delta += ins(var) - ins(base);
}

/// Pairs the records of two runs by (configuration, sample, repetition) and compares them.
pub fn compare_runs(
    base: &[RunRecord],
    var: &[RunRecord],
    terms: &KeyTerms,
) -> BTreeMap<ConfigKey, Delta> {
    let index: BTreeMap<(ConfigKey, String, u32), &RunRecord> = var
        .iter()
        .filter(|r| r.error.is_none())
        .map(|r| ((ConfigKey::of(r), r.sample_id.clone(), r.repetition), r))
        .collect();
    let mut out: BTreeMap<ConfigKey, Delta> = BTreeMap::new();
    for b in base.iter().filter(|r| r.error.is_none()) {
        let key = (ConfigKey::of(b), b.sample_id.clone(), b.repetition);
        let Some(v) = index.get(&key) else { continue };
        let sample_terms = terms.get(&b.sample_id).map(Vec::as_slice).unwrap_or(&[]);
        compare_pair(b, v, sample_terms, out.entry(key.0).or_default());
    }
    out
}

// ------------------------------------------------------------------ reports

fn pct(x: f64) -> String {
    format!("{:.1} %", x * 100.0)
}

fn frac((total, missed): (usize, usize)) -> String {
    format!("{}/{}", total - missed, total)
}

/// Markdown table of the key-term outcome of one run folder (one row per configuration).
pub fn baseline_markdown(records: &[RunRecord], terms: &KeyTerms) -> String {
    let groups = group_by_config(records);
    let kinds: BTreeSet<String> = groups
        .values()
        .flat_map(|g| summarize_config(g, terms).kinds.into_keys().collect::<Vec<_>>())
        .collect();
    let mut out = String::from("| configuration | samples | WER |");
    for k in &kinds {
        out.push_str(&format!(" {k} found |"));
    }
    out.push_str(" samples with a critical flag |\n|---|---|---|");
    for _ in &kinds {
        out.push_str("---|");
    }
    out.push_str("---|\n");
    for (key, g) in &groups {
        let s = summarize_config(g, terms);
        out.push_str(&format!("| {} | {} | {} |", key.label(), s.samples, pct(s.wer())));
        for k in &kinds {
            out.push_str(&format!(" {} |", frac(s.kind(k))));
        }
        out.push_str(&format!(" {} |\n", s.critical_samples));
    }
    out
}

/// Markdown comparison of a variant against a baseline: per configuration the WER, the key terms
/// found, and the fixes versus false corrections.
pub fn comparison_markdown(base: &[RunRecord], var: &[RunRecord], terms: &KeyTerms) -> String {
    let (gb, gv) = (group_by_config(base), group_by_config(var));
    let deltas = compare_runs(base, var, terms);
    let mut out = String::from(
        "| configuration | WER base -> variant | drug found | term found | tech found | critical samples | fixed words | broken words (all) | broken words outside key terms | extra words |\n|---|---|---|---|---|---|---|---|---|---|\n",
    );
    let mut examples = String::new();
    for (key, d) in &deltas {
        let (Some(b), Some(v)) = (gb.get(key), gv.get(key)) else { continue };
        // Restrict both sides to the samples present in both runs.
        let ids: BTreeSet<&str> = v.iter().map(|r| r.sample_id.as_str()).collect();
        let b: Vec<&RunRecord> = b.iter().copied().filter(|r| ids.contains(r.sample_id.as_str())).collect();
        let (sb, sv) = (summarize_config(&b, terms), summarize_config(v, terms));
        let cell = |k: &str| format!("{} -> {}", frac(sb.kind(k)), frac(sv.kind(k)));
        let outside = d.regressions.iter().filter(|c| !c.in_key_term).count();
        out.push_str(&format!(
            "| {} | {} -> {} | {} | {} | {} | {} -> {} | {} | {} | {} | {:+} |\n",
            key.label(),
            pct(sb.wer()),
            pct(sv.wer()),
            cell("drug"),
            cell("term"),
            cell("tech"),
            sb.critical_samples,
            sv.critical_samples,
            d.fixes.len(),
            d.regressions.len(),
            outside,
            d.insertions_delta
        ));
        if !d.regressions.is_empty() || !d.fixes.is_empty() {
            examples.push_str(&format!("\n#### {}\n", key.label()));
            for (title, list) in [("Fixed", &d.fixes), ("Broken", &d.regressions)] {
                for c in list.iter().take(25) {
                    examples.push_str(&format!(
                        "- {title}: `{}` expected `{}`, baseline `{}`, variant `{}`{}\n",
                        c.sample_id,
                        c.reference_word,
                        c.baseline,
                        c.variant,
                        if c.in_key_term { " (key term)" } else { "" }
                    ));
                }
            }
        }
    }
    out.push_str(&examples);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speech::benchmark::RunRecord;

    fn record(sample: &str, reference: &str, text: &str, terms: &[KeyTerm]) -> RunRecord {
        let cmp = metrics::compare_texts(reference, text);
        let report = critical::analyze(reference, text, terms);
        let json = serde_json::json!({
            "runId": "t", "timestampMs": 0, "modelId": "m", "provider": "p", "decoding": "greedy search",
            "sampleId": sample, "speakerId": "s", "category": "c", "language": "fr", "domain": "d",
            "utteranceType": "u", "private": false, "repetition": 1, "coldStart": false, "loadMs": 0,
            "inferenceMs": 0, "audioMs": 0, "rtf": null, "threads": 1, "peakMemoryMb": null, "cpuMs": null,
            "avgCores": null, "reference": reference, "text": text, "wer": cmp.wer, "cer": cmp.cer,
            "substitutions": cmp.substitutions, "deletions": cmp.deletions, "insertions": cmp.insertions,
            "referenceWords": cmp.reference_words, "critical": report.flags, "hasCritical": report.has_critical,
            "error": null
        });
        serde_json::from_value(json).expect("record")
    }

    fn drug(text: &str) -> KeyTerm {
        KeyTerm { text: text.into(), kind: "drug".into() }
    }

    #[test]
    fn outcomes_flag_a_misspelled_drug() {
        let t = [drug("amoxicilline")];
        let ok = term_outcomes("Il prend de l'amoxicilline.", "il prend de l'amoxicilline", &t);
        assert_eq!(ok, vec![("drug".to_string(), "amoxicilline".to_string(), true)]);
        let bad = term_outcomes("Il prend de l'amoxicilline.", "il prend de l'amoxycilline", &t);
        assert!(!bad[0].2);
    }

    #[test]
    fn summary_counts_terms_and_errors() {
        let t = vec![drug("amoxicilline")];
        let terms: KeyTerms = [("a".to_string(), t.clone())].into_iter().collect();
        let r = record("a", "il prend de l'amoxicilline", "il prend de l'amoxycilline", &t);
        let s = summarize_config(&[&r], &terms);
        assert_eq!(s.kind("drug"), (1, 1));
        assert_eq!(s.word_errors, 1);
        assert_eq!(s.samples, 1);
    }

    #[test]
    fn fixes_and_false_corrections_are_told_apart() {
        let t = vec![drug("amoxicilline")];
        let reference = "il prend de l'amoxicilline avec la mer";
        let base = record("a", reference, "il prend de l'amoxycilline avec la mer", &t);
        // The variant fixes the drug and breaks an unrelated word.
        let var = record("a", reference, "il prend de l'amoxicilline avec la mère", &t);
        let mut d = Delta::default();
        compare_pair(&base, &var, &t, &mut d);
        assert_eq!(d.fixes.len(), 1);
        assert_eq!(d.fixes[0].reference_word, "l'amoxicilline");
        assert!(d.fixes[0].in_key_term);
        assert_eq!(d.regressions.len(), 1);
        assert_eq!(d.regressions[0].reference_word, "mer");
        assert_eq!(d.regressions[0].variant, "mère");
        assert!(!d.regressions[0].in_key_term);
    }

    #[test]
    fn identical_records_change_nothing() {
        let r = record("a", "bonjour tout le monde", "bonjour tout le monde", &[]);
        let mut d = Delta::default();
        compare_pair(&r, &r, &[], &mut d);
        assert!(d.fixes.is_empty() && d.regressions.is_empty());
        assert_eq!(d.pairs, 1);
    }

    #[test]
    fn markdown_reports_run_pairs() {
        let t = vec![drug("amoxicilline")];
        let terms: KeyTerms = [("a".to_string(), t.clone())].into_iter().collect();
        let base = vec![record("a", "de l'amoxicilline", "de l'amoxycilline", &t)];
        let var = vec![record("a", "de l'amoxicilline", "de l'amoxicilline", &t)];
        let md = comparison_markdown(&base, &var, &terms);
        assert!(md.contains("0/1 -> 1/1"), "{md}");
        assert!(baseline_markdown(&base, &terms).contains("drug found"));
    }
}
