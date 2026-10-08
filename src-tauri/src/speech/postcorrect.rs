//! Dictionary post-correction (technique B of the drug-name study, D-036).
//!
//! After the engine has produced a text, words that are a near miss of a known term (a drug name,
//! a technical term) are replaced by that term. It is a plain string step: no engine knowledge, no
//! audio, so it works behind any provider. It is OFF by default and never runs unless asked.
//!
//! RISK, stated up front: a dictionary cannot tell a misspelling from a real, unusual word. With
//! "convention" in the vocabulary, a correct "conversion" is two edits away and can be silently
//! replaced. In a medical text a silent replacement is worse than a visible misspelling, because
//! nobody will look at it twice. That is why the default thresholds are strict (long words only,
//! one or two edits at most, same first letter) and why every change is returned in `changes`, so
//! a product can show or log it.
//!
//! What it never touches: numbers (any word with a digit), units, negations, weekday and month
//! names, words below the minimum length, and words that are already a term of the vocabulary.
//! Matching ignores case and accents; the replacement uses the spelling of the vocabulary entry
//! (with a leading capital kept when the original word had one).

use std::fs;
use std::path::Path;

use super::critical::{DATE_WORDS, STRONG_NEGATIONS};
use super::numbers::canonical_unit;

/// How eager the correction is.
#[derive(Debug, Clone, PartialEq)]
pub struct PostCorrectConfig {
    /// Shortest word (letters, after joining a multi-word window) that may be changed, and shortest
    /// term that may be a target.
    pub min_len: usize,
    /// Allowed edit distance = floor(length of the term x ratio), at most `max_distance`.
    pub ratio: f64,
    pub max_distance: usize,
    /// The first letter (accent-folded) of the word and of the term must agree.
    pub same_first_letter: bool,
}

impl PostCorrectConfig {
    /// Default: long words, one edit up to 11 letters, two from 12.
    pub fn strict() -> Self {
        Self { min_len: 6, ratio: 0.17, max_distance: 2, same_first_letter: true }
    }
    /// Study setting: shorter words (5+), one edit up to 9 letters, two from 10.
    pub fn medium() -> Self {
        Self { min_len: 5, ratio: 0.22, max_distance: 2, same_first_letter: true }
    }
    /// Study setting, deliberately risky: 4+ letters, up to three edits, any first letter.
    pub fn loose() -> Self {
        Self { min_len: 4, ratio: 0.3, max_distance: 3, same_first_letter: false }
    }
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "strict" => Some(Self::strict()),
            "medium" => Some(Self::medium()),
            "loose" => Some(Self::loose()),
            _ => None,
        }
    }
    fn allowed(&self, term_len: usize) -> usize {
        ((term_len as f64 * self.ratio).floor() as usize).min(self.max_distance)
    }
}

impl Default for PostCorrectConfig {
    fn default() -> Self {
        Self::strict()
    }
}

/// One replacement that was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub from: String,
    pub to: String,
    pub distance: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corrected {
    pub text: String,
    pub changes: Vec<Change>,
}

/// Reads a vocabulary file: one term per line, blank lines and lines starting with `#` ignored.
pub fn load_vocabulary(path: &Path) -> std::io::Result<Vec<String>> {
    Ok(parse_vocabulary(&fs::read_to_string(path)?))
}

pub fn parse_vocabulary(text: &str) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    text.lines()
        .map(|l| l.trim().trim_start_matches('\u{feff}').trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter(|l| seen.insert(l.to_lowercase()))
        .map(str::to_string)
        .collect()
}

fn fold_char(c: char) -> Vec<char> {
    match c {
        'à' | 'â' | 'ä' | 'á' | 'ã' => vec!['a'],
        'é' | 'è' | 'ê' | 'ë' => vec!['e'],
        'î' | 'ï' | 'í' => vec!['i'],
        'ô' | 'ö' | 'ó' | 'õ' => vec!['o'],
        'ù' | 'û' | 'ü' | 'ú' => vec!['u'],
        'ç' => vec!['c'],
        'ÿ' => vec!['y'],
        'œ' => vec!['o', 'e'],
        'æ' => vec!['a', 'e'],
        other => vec![other],
    }
}

/// Lower-case, accent-folded letters and digits.
fn fold(s: &str) -> Vec<char> {
    s.chars().flat_map(|c| c.to_lowercase()).flat_map(fold_char).collect()
}

fn levenshtein(a: &[char], b: &[char]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push((prev[j] + usize::from(ca != cb)).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// A word of the text with its byte range.
#[derive(Debug, Clone)]
struct Word {
    start: usize,
    end: usize,
}

fn words_of(text: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in text.char_indices() {
        if c.is_alphanumeric() {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            out.push(Word { start: s, end: i });
        }
    }
    if let Some(s) = start {
        out.push(Word { start: s, end: text.len() });
    }
    out
}

/// Words that must never be rewritten.
fn is_protected(word: &str) -> bool {
    if word.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }
    let lower = word.to_lowercase();
    STRONG_NEGATIONS.contains(&lower.as_str())
        || DATE_WORDS.contains(&lower.as_str())
        || canonical_unit(&lower).is_some()
}

struct Term {
    surface: String,
    folded: Vec<char>,
    words: usize,
}

fn prepare_terms(vocabulary: &[String], cfg: &PostCorrectConfig) -> Vec<Term> {
    vocabulary
        .iter()
        .filter_map(|t| {
            let ws = words_of(t);
            let folded: Vec<char> = ws.iter().flat_map(|w| fold(&t[w.start..w.end])).collect();
            (folded.len() >= cfg.min_len && !ws.is_empty() && !ws.iter().any(|w| is_protected(&t[w.start..w.end])))
                .then(|| Term { surface: t.clone(), folded, words: ws.len() })
        })
        .collect()
}

/// The text between two words may only be blanks, a hyphen or an apostrophe for the words to count
/// as one term ("hypertension artérielle", "contre-indiqué").
fn gap_joins(gap: &str) -> bool {
    !gap.is_empty() && gap.chars().all(|c| matches!(c, ' ' | '-' | '\'' | '’' | '\u{a0}'))
}

/// "backups" against "backup": a trailing s or x is a plural mark, and a changed number is a
/// changed meaning, so such pairs are not treated as misspellings.
fn differs_only_by_plural_mark(a: &[char], b: &[char]) -> bool {
    let longer_by_mark = |long: &[char], short: &[char]| {
        long.len() == short.len() + 1 && long[..short.len()] == *short && matches!(long[short.len()], 's' | 'x')
    };
    longer_by_mark(a, b) || longer_by_mark(b, a)
}

fn with_original_case(term: &str, original: &str) -> String {
    let upper = original.chars().next().is_some_and(char::is_uppercase);
    let lower_term = term.chars().next().is_some_and(char::is_lowercase);
    if upper && lower_term {
        let mut it = term.chars();
        it.next().map(|f| f.to_uppercase().collect::<String>() + it.as_str()).unwrap_or_default()
    } else {
        term.to_string()
    }
}

/// Replaces near-miss words by the vocabulary term they most likely are.
pub fn correct(text: &str, vocabulary: &[String], cfg: &PostCorrectConfig) -> Corrected {
    let terms = prepare_terms(vocabulary, cfg);
    let words = words_of(text);
    let mut out = String::with_capacity(text.len());
    let mut changes = Vec::new();
    let mut cursor = 0;
    let mut i = 0;
    while i < words.len() {
        let mut best: Option<(usize, usize, &Term)> = None; // (distance, words consumed, term)
        let mut tie = false;
        'terms: for term in &terms {
            // The engine may split or glue words ("contreindiqué" for "contre-indiqué"), so windows
            // of one word fewer and one word more than the term are tried as well; the smallest
            // distance wins, which keeps a neighbouring word from being swallowed.
            for size in term.words.saturating_sub(1).max(1)..=term.words + 1 {
                if i + size > words.len() {
                    continue;
                }
                // Every word of the window is inside one blank-joined run and none is protected.
                let window = &words[i..i + size];
                if window.iter().any(|w| is_protected(&text[w.start..w.end])) {
                    continue;
                }
                if (1..size).any(|j| !gap_joins(&text[window[j - 1].end..window[j].start])) {
                    continue;
                }
                let original = &text[window[0].start..window[size - 1].end];
                let folded: Vec<char> = window.iter().flat_map(|w| fold(&text[w.start..w.end])).collect();
                if folded.len() < cfg.min_len {
                    continue;
                }
                // Already exactly the term (case apart): leave it as the engine wrote it.
                if original.to_lowercase() == term.surface.to_lowercase() {
                    best = None;
                    tie = false;
                    break 'terms;
                }
                // Already another term of the vocabulary: not a misspelling.
                if terms.iter().any(|t| t.folded == folded && t.surface != term.surface) {
                    continue;
                }
                if cfg.same_first_letter && folded.first() != term.folded.first() {
                    continue;
                }
                // Same word in the plural: the engine's number is kept, never "corrected".
                if differs_only_by_plural_mark(&folded, &term.folded) {
                    continue;
                }
                let d = levenshtein(&folded, &term.folded);
                if d > cfg.allowed(term.folded.len()) {
                    continue;
                }
                match &best {
                    Some((bd, ..)) if d > *bd => {}
                    Some((bd, bs, bt)) if d == *bd => {
                        if size < *bs && bt.surface == term.surface {
                            best = Some((d, size, term));
                        } else if bt.surface != term.surface {
                            tie = true;
                        }
                    }
                    _ => {
                        best = Some((d, size, term));
                        tie = false;
                    }
                }
            }
        }
        match best {
            Some((d, k, term)) if !tie => {
                let first = &words[i];
                let last = &words[i + k - 1];
                let original = &text[first.start..last.end];
                out.push_str(&text[cursor..first.start]);
                out.push_str(&with_original_case(&term.surface, original));
                changes.push(Change { from: original.to_string(), to: term.surface.clone(), distance: d });
                cursor = last.end;
                i += k;
            }
            _ => i += 1,
        }
    }
    out.push_str(&text[cursor..]);
    Corrected { text: out, changes }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocab(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn strict(text: &str, v: &[&str]) -> Corrected {
        correct(text, &vocab(v), &PostCorrectConfig::strict())
    }

    #[test]
    fn near_miss_drug_name_is_snapped_to_the_vocabulary() {
        let r = strict("Il prend de l'amoxiciline le matin.", &["amoxicilline"]);
        assert_eq!(r.text, "Il prend de l'amoxicilline le matin.");
        assert_eq!(r.changes.len(), 1);
        assert_eq!(r.changes[0].from, "amoxiciline");
        assert_eq!(r.changes[0].distance, 1);
    }

    #[test]
    fn accent_only_difference_is_fixed_and_exact_word_is_left_alone() {
        let r = strict("du paracetamol et de l'ibuprofène", &["paracétamol", "ibuprofène"]);
        assert_eq!(r.text, "du paracétamol et de l'ibuprofène");
        assert_eq!(r.changes.len(), 1);
        assert_eq!(r.changes[0].to, "paracétamol");
        assert_eq!(r.changes[0].distance, 0);
    }

    #[test]
    fn exact_match_in_other_case_is_not_rewritten() {
        let r = strict("Utiliser postgresql ici", &["PostgreSQL"]);
        assert!(r.changes.is_empty());
        assert_eq!(r.text, "Utiliser postgresql ici");
    }

    #[test]
    fn capital_letter_of_the_original_is_kept() {
        let r = strict("Amoxiciline, deux fois par jour", &["amoxicilline"]);
        assert_eq!(r.text, "Amoxicilline, deux fois par jour");
    }

    #[test]
    fn numbers_units_negations_and_dates_are_never_touched() {
        // Vocabulary entries that look like protected words must not be targets either.
        let v = ["septembre", "milligramme", "jamais", "500mg"];
        let r = strict("en septembre 500 mg jamais 12mg Septembres", &v);
        assert_eq!(r.text, "en septembre 500 mg jamais 12mg Septembres");
        assert!(r.changes.is_empty());
        // A word with a digit is protected even when its letters match a term.
        let r = strict("amoxicilline2 amoxiciline", &["amoxicilline"]);
        assert_eq!(r.text, "amoxicilline2 amoxicilline");
    }

    #[test]
    fn short_words_are_not_touched() {
        // "main" is a 4-letter term: with the strict minimum length it is not even a target.
        let r = strict("mais la mer est calme", &["main"]);
        assert!(r.changes.is_empty());
        let loose = correct("mais la mer est calme", &vocab(&["main"]), &PostCorrectConfig::loose());
        assert_eq!(loose.text, "main la mer est calme", "the loose setting is the dangerous one");
    }

    #[test]
    fn a_real_but_unusual_word_two_edits_away_is_replaced_by_the_loose_setting_only() {
        // "conversion" is a correct word; "convention" is 2 edits away (documented risk).
        let v = ["convention"];
        assert!(strict("la conversion est finie", &v).changes.is_empty());
        let r = correct("la conversion est finie", &vocab(&v), &PostCorrectConfig::loose());
        assert_eq!(r.text, "la convention est finie");
    }

    #[test]
    fn multi_word_terms_match_across_blanks_but_not_across_punctuation() {
        let v = ["hypertension artérielle"];
        let r = strict("une hypertention arterielle connue", &v);
        assert_eq!(r.text, "une hypertension artérielle connue");
        let r = strict("une hypertention. arterielle connue", &v);
        assert!(r.changes.is_empty(), "a full stop separates two sentences");
    }

    #[test]
    fn hyphenated_term_is_matched_as_two_words() {
        let r = strict("contre indiquée chez", &["contre-indiqué"]);
        assert_eq!(r.text, "contre-indiqué chez");
    }

    #[test]
    fn ambiguous_matches_are_left_alone() {
        // "metformine" is one edit from both terms: no guess.
        let r = strict("de la metformine", &["metforminx", "metforminy"]);
        assert!(r.changes.is_empty(), "{:?}", r.changes);
    }

    #[test]
    fn first_letter_guard_blocks_unrelated_words() {
        let r = strict("une barotte ici", &["carotte"]);
        assert!(r.changes.is_empty());
        let no_guard = PostCorrectConfig { same_first_letter: false, ..PostCorrectConfig::strict() };
        assert_eq!(correct("une barotte ici", &vocab(&["carotte"]), &no_guard).text, "une carotte ici");
    }

    #[test]
    fn punctuation_and_spacing_survive() {
        let r = strict("(ibuprofen), puis « amoxiciline » !", &["ibuprofène", "amoxicilline"]);
        assert_eq!(r.text, "(ibuprofène), puis « amoxicilline » !");
        assert_eq!(r.changes.len(), 2);
    }

    #[test]
    fn a_neighbouring_word_is_not_swallowed_when_the_engine_glued_a_compound() {
        let r = strict("elle est contreindiqué en cas de doute", &["contre-indiqué"]);
        assert_eq!(r.text, "elle est contre-indiqué en cas de doute");
    }

    #[test]
    fn plurals_are_left_alone() {
        let v = ["convention", "backup", "justificatif"];
        let r = strict("des conventions, deux backups et les justificatifs", &v);
        assert!(r.changes.is_empty(), "{:?}", r.changes);
    }

    #[test]
    fn known_hazard_a_correct_look_alike_drug_name_is_replaced() {
        // "prednisolone" is a different, correctly spelled drug two edits away from "prednisone".
        // The strict preset (one edit for a 10-letter term) leaves it alone; the medium preset (two
        // edits for a 10-letter term) replaces it silently. This is the risk of a
        // dictionary step in a medical text, written down as a test so nobody forgets it (D-036).
        // Listing both drugs in the vocabulary is the only protection, because a word that is
        // itself a term is never changed.
        let v = vocab(&["prednisone"]);
        assert!(correct("traitement par prednisolone", &v, &PostCorrectConfig::strict()).changes.is_empty());
        let medium = correct("traitement par prednisolone", &v, &PostCorrectConfig::medium());
        assert_eq!(medium.text, "traitement par prednisone");
        let both = vocab(&["prednisone", "prednisolone"]);
        assert!(correct("traitement par prednisolone", &both, &PostCorrectConfig::medium()).changes.is_empty());
    }

    #[test]
    fn empty_inputs_are_fine() {
        assert_eq!(strict("", &["amoxicilline"]).text, "");
        assert_eq!(strict("bonjour", &[]).text, "bonjour");
    }

    #[test]
    fn vocabulary_parser_skips_comments_and_duplicates() {
        let v = parse_vocabulary("\u{feff}# comment\n\n amoxicilline \nAmoxicilline\nKubernetes\n");
        assert_eq!(v, vec!["amoxicilline".to_string(), "Kubernetes".to_string()]);
    }

    #[test]
    fn presets_are_ordered_by_risk() {
        let (s, m, l) = (PostCorrectConfig::strict(), PostCorrectConfig::medium(), PostCorrectConfig::loose());
        assert!(s.min_len > m.min_len && m.min_len > l.min_len);
        assert_eq!(s.allowed(11), 1);
        assert_eq!(s.allowed(12), 2);
        assert_eq!(PostCorrectConfig::by_name("medium"), Some(m));
        assert_eq!(PostCorrectConfig::by_name("nope"), None);
    }
}
