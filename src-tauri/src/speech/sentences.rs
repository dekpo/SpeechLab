//! Sentence splitter for the text-to-speech read-along (M6d, D-042).
//!
//! The synthesis library gives samples only, so the application synthesises the text sentence by
//! sentence and measures each one. This module decides where the sentences are and returns, for each
//! one, its span in the ORIGINAL text so the interface can highlight it. Offsets are counted in
//! UTF-16 code units, because that is what JavaScript string indices use (`text.slice(start, end)`).
//!
//! Rules (French and English, no language flag needed):
//! - `. ! ? …` end a sentence when followed by a space or the end of the text, and when the next visible
//!   word does not start with a lower-case letter ("Je pense... peut-être" stays in one piece).
//!   Closing quotes, guillemets (also after the French thin space) and brackets belong to the sentence.
//! - A dot after a known abbreviation (M., Dr, St, etc.), after a lone capital (an initial) or after a
//!   list number at the start of a sentence does not end it. Decimals (12,5 / 12.5), times (14:30,
//!   9h00, 10.30) and dotted acronyms (e.g.) have no space after the dot, so they never end it.
//! - A line break ends a sentence (lists, one item per line); a blank line also ends the paragraph.
//! - A "sentence" without any letter or digit is merged into its neighbour, never spoken alone.

/// One sentence of the original text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// The sentence as written (no leading or trailing white space).
    pub text: String,
    /// Start offset in the original text, in UTF-16 code units.
    pub start: usize,
    /// End offset (exclusive), in UTF-16 code units.
    pub end: usize,
    /// True when a blank line (or the end of the text) follows: the pause after it is longer.
    pub paragraph_end: bool,
}

/// Abbreviations after which a dot never ends the sentence (lower case, without the dot).
const NEVER_END: &[&str] = &[
    "m", "mm", "mme", "mmes", "mlle", "mlles", "dr", "dre", "drs", "pr", "pre", "prof", "me", "st", "ste", "sts",
    "mr", "mrs", "ms", "jr", "sr", "vs", "cf", "env", "approx", "e.g", "i.e", "c.-à-d", "fig", "réf", "ref",
    "ex", "p", "pp", "tel", "tél", "av", "bd", "dept", "inc", "ltd", "n", "no", "janv", "févr", "fevr", "avr",
    "juil", "sept", "oct", "nov", "déc", "dec", "jan", "feb", "mar", "apr", "aug", "sep",
];

/// "etc." ends a sentence only when the next word starts with a capital letter.
const ENDS_BEFORE_CAPITAL: &[&str] = &["etc", "al"];

fn is_terminal(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '…')
}

fn is_closer(c: char) -> bool {
    matches!(c, '"' | '\'' | '»' | '”' | '’' | ')' | ']' | '}')
}

fn is_opener(c: char) -> bool {
    matches!(c, '"' | '\'' | '«' | '“' | '‘' | '(' | '[' | '{' | '—' | '–' | '-' | '¿' | '¡')
}

/// First visible character at or after `from`, skipping white space and opening quotes or dashes.
fn next_visible(chars: &[char], mut from: usize) -> Option<char> {
    while from < chars.len() {
        let c = chars[from];
        if !(c.is_whitespace() || is_opener(c)) {
            return Some(c);
        }
        from += 1;
    }
    None
}

/// The word glued to the left of `dot` (white space ends it), without opening punctuation.
fn token_before(chars: &[char], dot: usize) -> String {
    let mut s = dot;
    while s > 0 && !chars[s - 1].is_whitespace() {
        s -= 1;
    }
    let raw: String = chars[s..dot].iter().collect();
    raw.trim_start_matches(|c: char| is_opener(c)).to_string()
}

/// Is the dot at `dot` part of an abbreviation, an initial or a list number (so it ends nothing)?
fn dot_is_not_final(chars: &[char], dot: usize, sentence_start: usize, after: usize) -> bool {
    let token = token_before(chars, dot);
    if token.is_empty() {
        return false;
    }
    let lower = token.to_lowercase();
    if NEVER_END.contains(&lower.as_str()) {
        return true;
    }
    if ENDS_BEFORE_CAPITAL.contains(&lower.as_str()) {
        return !matches!(next_visible(chars, after), Some(c) if c.is_uppercase());
    }
    let mut it = token.chars();
    // An initial: "J. Dupont" (one capital letter).
    if let (Some(c), None) = (it.next(), it.next()) {
        if c.is_alphabetic() && c.is_uppercase() {
            return true;
        }
    }
    // A list number at the start of the sentence: "1. Prendre le matin".
    if token.chars().all(|c| c.is_ascii_digit()) {
        let token_start = dot - token.chars().count();
        let only_blanks_before = chars[sentence_start..token_start].iter().all(|c| c.is_whitespace() || is_opener(*c));
        if only_blanks_before {
            return true;
        }
    }
    false
}

/// Splits `text` into sentences. Never fails: text without any sentence mark is one sentence.
pub fn split_sentences(text: &str) -> Vec<Sentence> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    // UTF-16 offset of every char index (plus the end), for the spans handed to the interface.
    let mut utf16 = Vec::with_capacity(n + 1);
    let mut acc = 0usize;
    for c in &chars {
        utf16.push(acc);
        acc += c.len_utf16();
    }
    utf16.push(acc);

    // (first char, end char exclusive, paragraph end)
    let mut spans: Vec<(usize, usize, bool)> = Vec::new();

    let emit = |spans: &mut Vec<(usize, usize, bool)>, from: usize, to: usize| {
        let (mut a, mut b) = (from, to);
        while a < b && chars[a].is_whitespace() {
            a += 1;
        }
        while b > a && chars[b - 1].is_whitespace() {
            b -= 1;
        }
        if a >= b {
            return;
        }
        if chars[a..b].iter().any(|c| c.is_alphanumeric()) {
            spans.push((a, b, false));
        } else if let Some(prev) = spans.last_mut() {
            // Only punctuation: glued to the previous sentence when it is on the same line, else dropped.
            if !chars[prev.1..a].contains(&'\n') {
                prev.1 = b;
            }
        }
    };

    let mut start = 0usize;
    let mut i = 0usize;
    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            let gap_start = i;
            let mut newlines = 0;
            while i < n && chars[i].is_whitespace() {
                if chars[i] == '\n' {
                    newlines += 1;
                }
                i += 1;
            }
            if newlines >= 1 {
                emit(&mut spans, start, gap_start);
                if newlines >= 2 {
                    if let Some(last) = spans.last_mut() {
                        last.2 = true;
                    }
                }
                start = i;
            } else if gap_start == start {
                start = i;
            }
            continue;
        }
        if is_terminal(c) {
            let mut j = i;
            while j < n && is_terminal(chars[j]) {
                j += 1;
            }
            let run_end = j;
            // Closing quotes and brackets belong to the sentence; only a guillemet may follow a space
            // (French typography: "Viens ! »").
            let mut end = run_end;
            loop {
                let mut k = end;
                while k < n && chars[k].is_whitespace() && chars[k] != '\n' {
                    k += 1;
                }
                if k < n && is_closer(chars[k]) && (k == end || chars[k] == '»') {
                    end = k + 1;
                } else {
                    break;
                }
            }
            let at_end = end >= n;
            let mut boundary = at_end || chars[end].is_whitespace();
            if boundary && !at_end {
                // A line break always closes the sentence, whatever comes next.
                let gap_has_newline = chars[end..].iter().take_while(|c| c.is_whitespace()).any(|c| *c == '\n');
                if !gap_has_newline {
                    if matches!(next_visible(&chars, end), Some(next) if next.is_lowercase()) {
                        boundary = false;
                    }
                    let single_dot = run_end - i == 1 && c == '.';
                    if boundary && single_dot && dot_is_not_final(&chars, i, start, run_end) {
                        boundary = false;
                    }
                }
            }
            if boundary {
                emit(&mut spans, start, end);
                start = end;
                i = end;
            } else {
                i = run_end;
            }
            continue;
        }
        i += 1;
    }
    emit(&mut spans, start, n);

    if let Some(last) = spans.last_mut() {
        last.2 = true;
    }
    spans
        .into_iter()
        .map(|(a, b, paragraph_end)| Sentence {
            text: chars[a..b].iter().collect(),
            start: utf16[a],
            end: utf16[b],
            paragraph_end,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(s: &str) -> Vec<String> {
        split_sentences(s).into_iter().map(|x| x.text).collect()
    }

    #[test]
    fn plain_sentences_french_and_english() {
        assert_eq!(texts("Bonjour. Comment allez-vous ? Très bien !"), ["Bonjour.", "Comment allez-vous ?", "Très bien !"]);
        assert_eq!(texts("Hello there. How are you? Fine!"), ["Hello there.", "How are you?", "Fine!"]);
        assert_eq!(texts("Sans ponctuation finale"), ["Sans ponctuation finale"]);
        assert!(split_sentences("   \n\n  ").is_empty());
        assert!(split_sentences("").is_empty());
    }

    #[test]
    fn abbreviations_do_not_end_a_sentence() {
        assert_eq!(texts("M. Martin voit le Dr Dupont. Mme Roux arrive."), ["M. Martin voit le Dr Dupont.", "Mme Roux arrive."]);
        assert_eq!(texts("Voir le Pr. Leroy, St. Gall, env. 5 jours. Fin."), ["Voir le Pr. Leroy, St. Gall, env. 5 jours.", "Fin."]);
        assert_eq!(texts("Mr. Smith met Dr. Jones vs. Mrs. Lee. Done."), ["Mr. Smith met Dr. Jones vs. Mrs. Lee.", "Done."]);
        assert_eq!(texts("Use a tool, e.g. a hammer. Then rest."), ["Use a tool, e.g. a hammer.", "Then rest."]);
        assert_eq!(texts("Le 12 janv. 2026, il pleut. Fin."), ["Le 12 janv. 2026, il pleut.", "Fin."]);
        assert_eq!(texts("J. Dupont est là. Oui."), ["J. Dupont est là.", "Oui."]);
    }

    #[test]
    fn etc_ends_a_sentence_only_before_a_capital() {
        assert_eq!(texts("Des pommes, des poires, etc. Puis il partit."), ["Des pommes, des poires, etc.", "Puis il partit."]);
        assert_eq!(texts("Des pommes, etc. et des poires."), ["Des pommes, etc. et des poires."]);
        assert_eq!(texts("Des pommes etc."), ["Des pommes etc."]);
    }

    #[test]
    fn decimals_times_and_numbers_stay_inside() {
        assert_eq!(texts("Prendre 12,5 mg à 14:30. Puis 9h00 ou 10.30 demain."), ["Prendre 12,5 mg à 14:30.", "Puis 9h00 ou 10.30 demain."]);
        assert_eq!(texts("Take 12.5 mg at 14.30. Then stop."), ["Take 12.5 mg at 14.30.", "Then stop."]);
        assert_eq!(texts("La facture n° 2045 est payée. Merci."), ["La facture n° 2045 est payée.", "Merci."]);
        // A number that ends a sentence does end it.
        assert_eq!(texts("Il y en a 3. Puis 4."), ["Il y en a 3.", "Puis 4."]);
    }

    #[test]
    fn list_numbers_at_the_start_are_not_sentence_ends() {
        assert_eq!(texts("1. Prendre le matin\n2. Prendre le soir"), ["1. Prendre le matin", "2. Prendre le soir"]);
    }

    #[test]
    fn ellipses_follow_the_next_word() {
        assert_eq!(texts("Je pense... peut-être. Oui."), ["Je pense... peut-être.", "Oui."]);
        assert_eq!(texts("Alors... Il partit."), ["Alors...", "Il partit."]);
        assert_eq!(texts("Alors… Il partit."), ["Alors…", "Il partit."]);
        assert_eq!(texts("Vraiment?! Oui."), ["Vraiment?!", "Oui."]);
    }

    #[test]
    fn quotes_and_guillemets_belong_to_the_sentence() {
        assert_eq!(texts("Il dit : « Viens ! » Puis il partit."), ["Il dit : « Viens ! »", "Puis il partit."]);
        assert_eq!(texts("Il dit : « Viens ! », puis il partit."), ["Il dit : « Viens ! », puis il partit."]);
        assert_eq!(texts("He said \"Come here.\" Then left."), ["He said \"Come here.\"", "Then left."]);
        assert_eq!(texts("(Voir ci-dessous.) Suite."), ["(Voir ci-dessous.)", "Suite."]);
        assert_eq!(texts("« Tu viens ? » demanda-t-il. Oui."), ["« Tu viens ? » demanda-t-il.", "Oui."]);
    }

    #[test]
    fn line_breaks_and_paragraphs() {
        let s = split_sentences("Ordonnance\nAmoxicilline 500 mg\n\nSuite du dossier. Fin.\r\n\r\nDernier");
        let t: Vec<_> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(t, ["Ordonnance", "Amoxicilline 500 mg", "Suite du dossier.", "Fin.", "Dernier"]);
        let p: Vec<_> = s.iter().map(|x| x.paragraph_end).collect();
        assert_eq!(p, [false, true, false, true, true]);
    }

    #[test]
    fn punctuation_only_pieces_are_not_sentences() {
        assert_eq!(texts("Bonjour. ... Merci."), ["Bonjour. ...", "Merci."]);
        assert_eq!(texts("- \n Texte."), ["- \n Texte."][..1].iter().map(|_| "Texte.".to_string()).collect::<Vec<_>>());
    }

    #[test]
    fn offsets_are_utf16_positions_in_the_original_text() {
        let original = "  Été chaud. 😀 Bien, oui ?\n\nFin";
        let units: Vec<u16> = original.encode_utf16().collect();
        for s in split_sentences(original) {
            let slice = String::from_utf16(&units[s.start..s.end]).unwrap();
            assert_eq!(slice, s.text, "span {}..{}", s.start, s.end);
        }
        let s = split_sentences(original);
        assert_eq!(s[0].start, 2, "leading blanks are skipped");
        assert_eq!(s.len(), 3);
        assert!(s[1].text.starts_with('😀') && s[1].paragraph_end);
    }

    #[test]
    fn a_blank_text_gives_no_sentence_and_the_last_one_closes_the_paragraph() {
        let s = split_sentences("Un seul.");
        assert_eq!(s.len(), 1);
        assert!(s[0].paragraph_end);
    }
}
