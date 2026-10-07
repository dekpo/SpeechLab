//! Critical semantic error detection.
//!
//! A transcript can have a low WER and still be dangerous: a changed dosage, a lost "pas", a
//! wrong weekday, a misspelled drug. This module compares a reference and a hypothesis and
//! raises FLAGS for those cases, independently of WER:
//!   - numbers (values, decimals, folded from spoken form, see `numbers.rs`)
//!   - units (mg, g, ml, %, ... with spoken equivalents)
//!   - negations (a dropped or added "pas", "sans", "not", ...)
//!   - dates (weekday and month names)
//!   - expected key terms listed in the dataset (drug names, legal terms, product names)
//!
//! It is a heuristic safety net for evaluation, not a medical device: it can miss errors
//! (for example a changed noun) and can raise false alarms (for example a compound number it
//! does not understand). Every flag keeps the expected and the found text so a human can judge.

use serde::{Deserialize, Serialize};

use super::metrics::normalize_words;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FlagKind {
    Number,
    Unit,
    Negation,
    Date,
    KeyTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Critical,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticalFlag {
    pub kind: FlagKind,
    pub severity: Severity,
    pub expected: String,
    pub found: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticalReport {
    pub flags: Vec<CriticalFlag>,
    pub has_critical: bool,
}

/// A term the transcript must contain, from the dataset ("amoxicilline", "PostgreSQL").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyTerm {
    pub text: String,
    /// "drug", "name", "legal", "tech", "term"... (drug and name are critical when missed).
    #[serde(default = "default_kind")]
    pub kind: String,
}

fn default_kind() -> String {
    "term".into()
}

fn flag(kind: FlagKind, severity: Severity, expected: &str, found: &str, message: &str) -> CriticalFlag {
    CriticalFlag {
        kind,
        severity,
        expected: expected.to_string(),
        found: found.to_string(),
        message: message.to_string(),
    }
}

// ---------- numbers ----------

fn numeric_value(token: &str) -> Option<String> {
    let mut seen_digit = false;
    let mut seen_sep = false;
    for c in token.chars() {
        match c {
            '0'..='9' => seen_digit = true,
            '.' | ',' if seen_digit && !seen_sep => seen_sep = true,
            _ => return None,
        }
    }
    if !seen_digit || token.ends_with(['.', ',']) {
        return None;
    }
    // Unify the decimal separator and drop leading zeros ("05" == "5").
    let t = token.replace(',', ".");
    let trimmed = t.trim_start_matches('0');
    Some(if trimmed.is_empty() || trimmed.starts_with('.') {
        format!("0{trimmed}")
    } else {
        trimmed.to_string()
    })
}

fn numbers_of(tokens: &[String]) -> Vec<String> {
    tokens.iter().filter_map(|t| numeric_value(t)).collect()
}

/// Removes the multiset intersection; returns (only_in_a, only_in_b) preserving order.
fn multiset_diff(a: &[String], b: &[String]) -> (Vec<String>, Vec<String>) {
    let mut b_left: Vec<Option<&String>> = b.iter().map(Some).collect();
    let mut only_a = Vec::new();
    for x in a {
        if let Some(slot) = b_left.iter_mut().find(|s| s.is_some_and(|y| y == x)) {
            *slot = None;
        } else {
            only_a.push(x.clone());
        }
    }
    (only_a, b_left.into_iter().flatten().cloned().collect())
}

// ---------- units ----------

fn units_of(tokens: &[String]) -> Vec<String> {
    tokens.iter().filter_map(|t| super::numbers::canonical_unit(t).map(str::to_string)).collect()
}

// ---------- negations ----------

const STRONG_NEGATIONS: &[&str] = &[
    "pas", "jamais", "aucun", "aucune", "aucuns", "aucunes", "sans", "non", "rien", "ni", "nul", "nulle",
    "not", "no", "never", "without", "none", "nothing", "nobody", "cannot",
];

fn negation_count(tokens: &[String]) -> usize {
    let strong = tokens
        .iter()
        .filter(|t| STRONG_NEGATIONS.contains(&t.as_str()) || t.ends_with("n't"))
        .count();
    if strong > 0 {
        return strong;
    }
    // "ne ... plus", "n'a ... que": only a weak marker is present.
    usize::from(tokens.iter().any(|t| t == "ne" || t.starts_with("n'")))
}

// ---------- dates ----------

const DATE_WORDS: &[&str] = &[
    "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche", "janvier", "février", "mars",
    "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre", "monday",
    "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday", "january", "february", "march",
    "april", "june", "july", "august", "october", "december",
];

fn dates_of(tokens: &[String]) -> Vec<String> {
    tokens.iter().filter(|t| DATE_WORDS.contains(&t.as_str())).cloned().collect()
}

// ---------- key terms ----------

fn similarity(a: &str, b: &str) -> f64 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let max = a.len().max(b.len());
    if max == 0 {
        return 1.0;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push((prev[j] + usize::from(ca != cb)).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    1.0 - prev[b.len()] as f64 / max as f64
}

/// French elisions glue words together ("l'amoxicilline", "d'allergie"): for term search the
/// tokens are split at apostrophes so the term itself is found.
fn split_elisions(tokens: &[String]) -> Vec<String> {
    tokens
        .iter()
        .flat_map(|t| t.split('\'').filter(|p| !p.is_empty()).map(str::to_string).collect::<Vec<_>>())
        .collect()
}

/// Exact word-sequence match, or the closest window with its similarity (0..1).
fn find_term(term: &[String], hyp: &[String]) -> (bool, Option<(String, f64)>) {
    if term.is_empty() {
        return (true, None);
    }
    if hyp.windows(term.len()).any(|w| w == term) {
        return (true, None);
    }
    let target = term.join("");
    let mut best: Option<(String, f64)> = None;
    for size in term.len().saturating_sub(1).max(1)..=term.len() + 1 {
        for w in hyp.windows(size.min(hyp.len().max(1))) {
            let joined = w.join("");
            let s = similarity(&target, &joined);
            if best.as_ref().map_or(true, |(_, b)| s > *b) {
                best = Some((w.join(" "), s));
            }
        }
    }
    (false, best)
}

pub fn analyze(reference: &str, hypothesis: &str, key_terms: &[KeyTerm]) -> CriticalReport {
    let r = normalize_words(reference);
    let h = normalize_words(hypothesis);
    let mut flags = Vec::new();

    // Numbers
    let (miss, extra) = multiset_diff(&numbers_of(&r), &numbers_of(&h));
    let paired = miss.len().min(extra.len());
    for i in 0..paired {
        flags.push(flag(
            FlagKind::Number,
            Severity::Critical,
            &miss[i],
            &extra[i],
            "a number was changed",
        ));
    }
    for m in miss.iter().skip(paired) {
        flags.push(flag(FlagKind::Number, Severity::Critical, m, "(absent)", "a number was lost"));
    }
    for e in extra.iter().skip(paired) {
        flags.push(flag(FlagKind::Number, Severity::Critical, "(none)", e, "a number was added"));
    }

    // Units
    let (miss, extra) = multiset_diff(&units_of(&r), &units_of(&h));
    let paired = miss.len().min(extra.len());
    for i in 0..paired {
        flags.push(flag(FlagKind::Unit, Severity::Critical, &miss[i], &extra[i], "a unit was changed"));
    }
    for m in miss.iter().skip(paired) {
        flags.push(flag(FlagKind::Unit, Severity::Critical, m, "(absent)", "a unit was lost"));
    }
    for e in extra.iter().skip(paired) {
        flags.push(flag(FlagKind::Unit, Severity::Critical, "(none)", e, "a unit was added"));
    }

    // Negations
    let (nr, nh) = (negation_count(&r), negation_count(&h));
    if nh < nr {
        flags.push(flag(
            FlagKind::Negation,
            Severity::Critical,
            &format!("{nr} negation marker(s)"),
            &format!("{nh}"),
            "a negation was dropped: the meaning may be reversed",
        ));
    } else if nh > nr {
        flags.push(flag(
            FlagKind::Negation,
            Severity::Critical,
            &format!("{nr} negation marker(s)"),
            &format!("{nh}"),
            "a negation was added: the meaning may be reversed",
        ));
    }

    // Dates (weekday and month names)
    let (miss, extra) = multiset_diff(&dates_of(&r), &dates_of(&h));
    let paired = miss.len().min(extra.len());
    for i in 0..paired {
        flags.push(flag(FlagKind::Date, Severity::Critical, &miss[i], &extra[i], "a day or month was changed"));
    }
    for m in miss.iter().skip(paired) {
        flags.push(flag(FlagKind::Date, Severity::Critical, m, "(absent)", "a day or month was lost"));
    }
    for e in extra.iter().skip(paired) {
        flags.push(flag(FlagKind::Date, Severity::Critical, "(none)", e, "a day or month was added"));
    }

    // Expected key terms
    let h_split = split_elisions(&h);
    for t in key_terms {
        let words = split_elisions(&normalize_words(&t.text));
        let (found, near) = find_term(&words, &h_split);
        if found {
            continue;
        }
        let severity = if matches!(t.kind.as_str(), "drug" | "name") {
            Severity::Critical
        } else {
            Severity::Warning
        };
        let (found_text, message) = match near {
            Some((w, s)) if s >= 0.6 => (
                w,
                format!("expected {} term not found; closest text is {:.0} % similar (probable misspelling)", t.kind, s * 100.0),
            ),
            _ => ("(absent)".to_string(), format!("expected {} term not found", t.kind)),
        };
        flags.push(flag(FlagKind::KeyTerm, severity, &t.text, &found_text, &message));
    }

    let has_critical = flags.iter().any(|f| f.severity == Severity::Critical);
    CriticalReport { flags, has_critical }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(r: &CriticalReport) -> Vec<FlagKind> {
        r.flags.iter().map(|f| f.kind).collect()
    }

    #[test]
    fn identical_or_equivalent_text_raises_nothing() {
        let r = analyze("Prendre 500 mg trois fois par jour.", "prendre cinq cents milligrammes 3 fois par jour", &[]);
        assert!(r.flags.is_empty(), "{:?}", r.flags);
        assert!(!r.has_critical);
    }

    #[test]
    fn a_changed_dosage_is_critical_even_when_wer_is_tiny() {
        let reference = "le patient prend 500 mg de ce médicament le matin avant le petit déjeuner et le soir";
        let hypothesis = "le patient prend 50 mg de ce médicament le matin avant le petit déjeuner et le soir";
        let r = analyze(reference, hypothesis, &[]);
        assert!(r.has_critical);
        assert_eq!(kinds(&r), vec![FlagKind::Number]);
        assert_eq!((r.flags[0].expected.as_str(), r.flags[0].found.as_str()), ("500", "50"));
        let wer = super::super::metrics::compare_texts(reference, hypothesis).wer.unwrap();
        assert!(wer < 0.1, "WER stays low ({wer}) while the error is critical");
    }

    #[test]
    fn a_changed_unit_is_critical() {
        let r = analyze("dose de 5 mg", "dose de 5 g", &[]);
        assert_eq!(kinds(&r), vec![FlagKind::Unit]);
        assert!(r.has_critical);
    }

    #[test]
    fn a_decimal_change_is_critical() {
        assert!(analyze("2,5 mg", "2.5 mg", &[]).flags.is_empty(), "same value, other separator");
        assert!(analyze("2,5 mg", "25 mg", &[]).has_critical);
    }

    #[test]
    fn percent_words_match_the_percent_sign() {
        assert!(analyze("réduction de 20 %", "réduction de vingt pour cent", &[]).flags.is_empty());
    }

    #[test]
    fn a_dropped_or_added_negation_is_critical() {
        let dropped = analyze("le patient ne présente pas d'allergie", "le patient présente d'allergie", &[]);
        assert_eq!(kinds(&dropped), vec![FlagKind::Negation]);
        assert!(dropped.has_critical);
        let added = analyze("the dose is correct", "the dose is not correct", &[]);
        assert_eq!(kinds(&added), vec![FlagKind::Negation]);
        assert!(analyze("I don't know", "I do not know", &[]).flags.is_empty());
    }

    #[test]
    fn a_changed_weekday_is_critical() {
        let r = analyze("rendez-vous jeudi à 10 heures", "rendez-vous jeudi à 10 heures", &[]);
        assert!(r.flags.is_empty());
        let r = analyze("rendez-vous jeudi à 10 heures", "rendez-vous mardi à 10 heures", &[]);
        assert_eq!(kinds(&r), vec![FlagKind::Date]);
    }

    #[test]
    fn key_terms_are_checked_and_misspellings_are_explained() {
        let terms = vec![KeyTerm { text: "amoxicilline".into(), kind: "drug".into() }];
        assert!(analyze("prescrire de l'amoxicilline", "prescrire de l'amoxicilline", &terms).flags.is_empty());
        let r = analyze("prescrire de l'amoxicilline", "prescrire de l'amoxicilline", &terms);
        assert!(!r.has_critical);
        let r = analyze("prescrire de l'amoxicilline", "prescrire de l'amoxycilline", &terms);
        assert_eq!(kinds(&r), vec![FlagKind::KeyTerm]);
        assert!(r.has_critical, "a missed drug name is critical");
        assert!(r.flags[0].message.contains("misspelling"), "{}", r.flags[0].message);
        let r = analyze("ouvre PostgreSQL", "ouvre post gré SQL", &[KeyTerm { text: "PostgreSQL".into(), kind: "tech".into() }]);
        assert_eq!(r.flags.len(), 1);
        assert_eq!(r.flags[0].severity, Severity::Warning, "non-drug terms are warnings");
    }

    #[test]
    fn multi_word_terms_and_absent_terms() {
        let t = vec![KeyTerm { text: "responsabilité civile".into(), kind: "legal".into() }];
        assert!(analyze("la responsabilité civile", "la responsabilité civile", &t).flags.is_empty());
        let r = analyze("la responsabilité civile", "la facture mensuelle", &t);
        assert_eq!(r.flags.len(), 1);
        assert_eq!(r.flags[0].found, "(absent)");
    }
}
