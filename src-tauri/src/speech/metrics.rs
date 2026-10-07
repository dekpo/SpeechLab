//! Text comparison metrics: WER, CER and a word-level diff.
//!
//! Original strings are never modified: normalisation is applied to copies for scoring only.
//! Basic normalisation (M4) lower-cases, turns hyphens/dashes and punctuation into spaces and
//! unifies apostrophes. It deliberately KEEPS digits, decimal separators between digits and
//! `%`, so a changed dosage still counts as an error. Critical-error detection (numbers,
//! units, negations, drug names) is a separate step planned for M5; WER alone must never be
//! read as clinical safety.
//!
//! Known limits: no Unicode NFC/NFD normalisation (decomposed accents compare as different),
//! no number-word equivalence ("trois" vs "3" counts as an error, on purpose for now).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffKind {
    Equal,
    /// Word present in both but different.
    Substitute,
    /// Word in the reference missing from the hypothesis.
    Delete,
    /// Extra word in the hypothesis.
    Insert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffOp {
    pub kind: DiffKind,
    pub reference: Option<String>,
    pub hypothesis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextComparison {
    /// Word error rate = (S + D + I) / reference words. None if the reference has no words.
    pub wer: Option<f64>,
    /// Character error rate on normalised text with single spaces. None if reference is empty.
    pub cer: Option<f64>,
    pub reference_words: usize,
    pub hypothesis_words: usize,
    pub substitutions: usize,
    pub deletions: usize,
    pub insertions: usize,
    pub diff: Vec<DiffOp>,
    pub normalized_reference: String,
    pub normalized_hypothesis: String,
}

/// Normalised word list used for scoring (the inputs are left untouched).
pub fn normalize_words(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut cleaned = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let prev_digit = i > 0 && chars[i - 1].is_ascii_digit();
        let next_digit = chars.get(i + 1).is_some_and(|n| n.is_ascii_digit());
        if c.is_alphanumeric() || c == '%' {
            cleaned.extend(c.to_lowercase());
        } else if matches!(c, '\'' | '’' | '‘' | '`') {
            cleaned.push('\'');
        } else if matches!(c, '.' | ',') && prev_digit && next_digit {
            cleaned.push(c);
        } else {
            cleaned.push(' ');
        }
    }
    cleaned
        .split_whitespace()
        .map(|w| w.trim_matches('\'').to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Minimum-edit alignment between two word sequences (substitution preferred on ties).
pub fn align(reference: &[String], hypothesis: &[String]) -> Vec<DiffOp> {
    let (n, m) = (reference.len(), hypothesis.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=m {
        d[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(reference[i - 1] != hypothesis[j - 1]);
            d[i][j] = (d[i - 1][j - 1] + cost)
                .min(d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1);
        }
    }
    let (mut i, mut j) = (n, m);
    let mut ops = Vec::new();
    while i > 0 || j > 0 {
        if i > 0 && j > 0 {
            let same = reference[i - 1] == hypothesis[j - 1];
            let cost = usize::from(!same);
            if d[i][j] == d[i - 1][j - 1] + cost {
                ops.push(DiffOp {
                    kind: if same { DiffKind::Equal } else { DiffKind::Substitute },
                    reference: Some(reference[i - 1].clone()),
                    hypothesis: Some(hypothesis[j - 1].clone()),
                });
                i -= 1;
                j -= 1;
                continue;
            }
        }
        if i > 0 && d[i][j] == d[i - 1][j] + 1 {
            ops.push(DiffOp {
                kind: DiffKind::Delete,
                reference: Some(reference[i - 1].clone()),
                hypothesis: None,
            });
            i -= 1;
        } else {
            ops.push(DiffOp {
                kind: DiffKind::Insert,
                reference: None,
                hypothesis: Some(hypothesis[j - 1].clone()),
            });
            j -= 1;
        }
    }
    ops.reverse();
    ops
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

pub fn compare_texts(reference: &str, hypothesis: &str) -> TextComparison {
    let r = normalize_words(reference);
    let h = normalize_words(hypothesis);
    let diff = align(&r, &h);
    let count = |k: DiffKind| diff.iter().filter(|o| o.kind == k).count();
    let (s, dl, ins) = (count(DiffKind::Substitute), count(DiffKind::Delete), count(DiffKind::Insert));
    let nr = r.join(" ");
    let nh = h.join(" ");
    let rc: Vec<char> = nr.chars().collect();
    let hc: Vec<char> = nh.chars().collect();
    TextComparison {
        wer: (!r.is_empty()).then(|| (s + dl + ins) as f64 / r.len() as f64),
        cer: (!rc.is_empty()).then(|| levenshtein(&rc, &hc) as f64 / rc.len() as f64),
        reference_words: r.len(),
        hypothesis_words: h.len(),
        substitutions: s,
        deletions: dl,
        insertions: ins,
        diff,
        normalized_reference: nr,
        normalized_hypothesis: nh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_text_has_zero_error_even_with_case_and_punctuation() {
        let c = compare_texts("Bonjour, le monde !", "bonjour le  monde");
        assert_eq!(c.wer, Some(0.0));
        assert_eq!(c.cer, Some(0.0));
        assert!(c.diff.iter().all(|o| o.kind == DiffKind::Equal));
    }

    #[test]
    fn hyphen_and_apostrophe_variants_are_unified() {
        assert_eq!(compare_texts("Demandez-vous", "demandez vous").wer, Some(0.0));
        assert_eq!(compare_texts("l’homme", "l'homme").wer, Some(0.0));
    }

    #[test]
    fn counts_substitution_deletion_insertion() {
        let c = compare_texts("a b c d", "a x c d e");
        assert_eq!((c.substitutions, c.deletions, c.insertions), (1, 0, 1));
        assert_eq!(c.wer, Some(2.0 / 4.0));
        let c = compare_texts("a b c", "a c");
        assert_eq!((c.substitutions, c.deletions, c.insertions), (0, 1, 0));
    }

    #[test]
    fn a_changed_dosage_is_an_error_not_normalised_away() {
        let c = compare_texts("prendre 500 mg par jour", "prendre 50 mg par jour");
        assert_eq!(c.substitutions, 1);
        assert!(c.wer.unwrap() > 0.0);
        let c = compare_texts("dose de 2,5 mg", "dose de 2.5 mg");
        assert_eq!(c.substitutions, 1, "decimal separators are kept as written");
    }

    #[test]
    fn a_dropped_negation_is_counted() {
        let c = compare_texts("ne pas prendre ce médicament", "prendre ce médicament");
        assert_eq!(c.deletions, 2);
    }

    #[test]
    fn empty_reference_gives_no_rate() {
        let c = compare_texts("", "bonjour");
        assert_eq!(c.wer, None);
        assert_eq!(c.cer, None);
        assert_eq!(c.insertions, 1);
    }

    #[test]
    fn cer_counts_character_edits() {
        let c = compare_texts("chat", "chap");
        assert_eq!(c.cer, Some(0.25));
    }

    #[test]
    fn original_inputs_are_not_modified_and_normalised_forms_are_returned() {
        let c = compare_texts("Bonjour, Monde.", "bonjour monde");
        assert_eq!(c.normalized_reference, "bonjour monde");
    }
}
