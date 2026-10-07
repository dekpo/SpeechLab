//! Number and unit normalisation for scoring.
//!
//! Applied to BOTH the reference and the hypothesis, so formatting differences do not count as
//! errors while different values still do:
//!   - spoken numbers become digits ("quinze" = "15", "cinq cents" = "500", "twenty one" = "21"),
//!     following the real grammar: "ten thirty" is TWO numbers (10 and 30, a time), not 40;
//!   - "pour cent" / "percent" become "%";
//!   - digit groups are canonical: "12,000" = "12.000" = "12000" (thousands), "05" = "5",
//!     and adjacent digit tokens are joined ("10 30" = "1030", a time written without a colon);
//!   - unit spellings are unified ("milligrammes" = "mg").
//!
//! A standalone "un"/"une"/"one" stays a word (article or pronoun). Anything not understood is
//! left untouched, which can only add a reported difference, never hide one.
//! Not covered: Swiss number words (septante, huitante, nonante), ordinals, fractions.

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lang {
    Fr,
    En,
}

fn fr_value(w: &str) -> Option<u32> {
    Some(match w {
        "zéro" | "zero" => 0,
        "un" | "une" => 1,
        "deux" => 2,
        "trois" => 3,
        "quatre" => 4,
        "cinq" => 5,
        "six" => 6,
        "sept" => 7,
        "huit" => 8,
        "neuf" => 9,
        "dix" => 10,
        "onze" => 11,
        "douze" => 12,
        "treize" => 13,
        "quatorze" => 14,
        "quinze" => 15,
        "seize" => 16,
        "vingt" | "vingts" => 20,
        "trente" => 30,
        "quarante" => 40,
        "cinquante" => 50,
        "soixante" => 60,
        _ => return None,
    })
}

fn en_value(w: &str) -> Option<u32> {
    Some(match w {
        "zero" => 0,
        "one" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        "eleven" => 11,
        "twelve" => 12,
        "thirteen" => 13,
        "fourteen" => 14,
        "fifteen" => 15,
        "sixteen" => 16,
        "seventeen" => 17,
        "eighteen" => 18,
        "nineteen" => 19,
        "twenty" => 20,
        "thirty" => 30,
        "forty" => 40,
        "fifty" => 50,
        "sixty" => 60,
        "seventy" => 70,
        "eighty" => 80,
        "ninety" => 90,
        _ => return None,
    })
}

fn value(w: &str, lang: Lang) -> Option<u32> {
    match lang {
        Lang::Fr => fr_value(w),
        Lang::En => en_value(w),
    }
}

fn is_hundred(w: &str, lang: Lang) -> bool {
    match lang {
        Lang::Fr => matches!(w, "cent" | "cents"),
        Lang::En => w == "hundred",
    }
}

fn is_thousand(w: &str, lang: Lang) -> bool {
    match lang {
        Lang::Fr => w == "mille",
        Lang::En => w == "thousand",
    }
}

fn is_connector(w: &str, lang: Lang) -> bool {
    match lang {
        Lang::Fr => w == "et",
        Lang::En => w == "and",
    }
}

fn is_number_word(w: &str, lang: Lang) -> bool {
    value(w, lang).is_some() || is_hundred(w, lang) || is_thousand(w, lang)
}

fn is_article_like(w: &str) -> bool {
    matches!(w, "un" | "une" | "one")
}

/// State of the number being built: `total` holds finished thousands, `group` the current
/// value below 1000.
#[derive(Clone, Copy, Default)]
struct Builder {
    total: u32,
    group: u32,
    words: usize,
}

impl Builder {
    /// Adds `w` if it continues the number grammatically; None otherwise.
    fn extend(self, w: &str, lang: Lang) -> Option<Builder> {
        let tail = self.group % 100;
        let mut next = self;
        if is_thousand(w, lang) {
            // "mille" / "thousand": at most once, multiplies the current group (or stands alone).
            if self.total != 0 || (self.group == 0 && self.words > 0) {
                return None;
            }
            next.total = self.group.max(1) * 1000;
            next.group = 0;
        } else if is_hundred(w, lang) {
            // "cent" / "hundred": multiplies a single unit, or stands alone.
            if self.group == 0 && self.words == 0 {
                next.group = 100;
            } else if (1..=9).contains(&self.group) {
                next.group = self.group * 100;
            } else {
                return None;
            }
        } else {
            let v = value(w, lang)?;
            if v == 0 {
                return (self.words == 0).then_some(Builder { words: 1, ..Builder::default() });
            }
            if lang == Lang::Fr && tail == 4 && v == 20 && self.group < 10 {
                next.group = 80; // quatre vingt(s)
            } else if tail == 0 {
                next.group = self.group + v;
            } else {
                let tens_part = tail >= 20 && tail % 10 == 0;
                let allowed = (tens_part
                    && match (lang, tail) {
                        (Lang::Fr, 60 | 80) => (1..=19).contains(&v),
                        _ => (1..=9).contains(&v),
                    })
                    // French "dix-sept", "dix-huit", "dix-neuf" (17, 18, 19).
                    || (lang == Lang::Fr && tail == 10 && (7..=9).contains(&v));
                if !allowed {
                    return None;
                }
                next.group = self.group + v;
            }
        }
        next.words += 1;
        Some(next)
    }

    fn value(self) -> u32 {
        self.total + self.group
    }
}

/// Replaces spoken numbers by digits in a token list (tokens already lower-cased).
fn fold_spoken(tokens: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let w = tokens[i].as_str();
        let lang = if is_number_word(w, Lang::Fr) {
            Lang::Fr
        } else if is_number_word(w, Lang::En) {
            Lang::En
        } else {
            out.push(tokens[i].clone());
            i += 1;
            continue;
        };
        let mut b = Builder::default();
        let mut j = i;
        let mut consumed_words = 0;
        while j < tokens.len() {
            let t = tokens[j].as_str();
            if let Some(n) = b.extend(t, lang) {
                b = n;
                j += 1;
                consumed_words += 1;
            } else if is_connector(t, lang)
                && b.words > 0
                && tokens.get(j + 1).is_some_and(|n| b.extend(n, lang).is_some())
            {
                j += 1; // "vingt et un", "one hundred and five"
            } else {
                break;
            }
        }
        if consumed_words == 0 {
            // Cannot happen (the first word always starts a number), but never loop forever.
            out.push(tokens[i].clone());
            i += 1;
            continue;
        }
        if consumed_words == 1 && is_article_like(w) {
            out.push(tokens[i].clone());
        } else {
            out.push(b.value().to_string());
        }
        i = j;
    }
    out
}

fn is_digits(t: &str) -> bool {
    !t.is_empty() && t.chars().all(|c| c.is_ascii_digit())
}

/// True for "12", "2,5", "12.000", "3,141.5" (digits with separators between digits).
pub fn is_numeric_token(t: &str) -> bool {
    let mut seen_digit = false;
    let mut prev_sep = true;
    for c in t.chars() {
        match c {
            '0'..='9' => {
                seen_digit = true;
                prev_sep = false;
            }
            '.' | ',' if !prev_sep => prev_sep = true,
            _ => return false,
        }
    }
    seen_digit && !prev_sep
}

/// Canonical form of a numeric token: thousands groups removed ("12,000" -> "12000"), decimal
/// comma turned into a point ("2,5" -> "2.5"), leading zeros dropped ("05" -> "5").
pub fn canonical_number_token(t: &str) -> String {
    if !is_numeric_token(t) {
        return t.to_string();
    }
    let parts: Vec<&str> = t.split(['.', ',']).collect();
    let thousands = parts.len() >= 2
        && (1..=3).contains(&parts[0].len())
        && !parts[0].starts_with('0')
        && parts[1..].iter().all(|p| p.len() == 3)
        && (parts.len() >= 3 || parts[0].parse::<u32>().is_ok_and(|v| v >= 10));
    let joined = if thousands {
        parts.concat()
    } else if parts.len() == 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        parts.concat()
    };
    let (int, frac) = match joined.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (joined.as_str(), None),
    };
    let int = int.trim_start_matches('0');
    let int = if int.is_empty() { "0" } else { int };
    match frac {
        Some(f) => format!("{int}.{f}"),
        None => int.to_string(),
    }
}

/// Joins runs of adjacent pure-digit tokens ("10 30" -> "1030"): a time written with or
/// without a separator is the same text.
fn join_adjacent_digits(tokens: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    for t in tokens {
        match out.last_mut() {
            Some(last) if is_digits(last) && is_digits(&t) => last.push_str(&t),
            _ => out.push(t),
        }
    }
    out
}

/// "pour cent" / "per cent" / "percent" -> "%" (before "cent" would be read as 100).
fn unify_percent(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut k = 0;
    while k < tokens.len() {
        let next = tokens.get(k + 1).map(String::as_str);
        match (tokens[k].as_str(), next) {
            ("pour" | "per", Some("cent")) => {
                out.push("%".into());
                k += 2;
            }
            ("pourcent" | "percent", _) => {
                out.push("%".into());
                k += 1;
            }
            _ => {
                out.push(tokens[k].clone());
                k += 1;
            }
        }
    }
    out
}

/// "deux virgule cinq" -> "2.5", "two point five" -> "2.5": the separator word only counts
/// between two digit tokens, so ordinary uses of "point" are untouched.
fn merge_spoken_decimals(tokens: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let is_sep = matches!(tokens[i].as_str(), "virgule" | "point");
        if is_sep
            && out.last().is_some_and(|p| is_digits(p))
            && tokens.get(i + 1).is_some_and(|n| is_digits(n))
        {
            let last = out.pop().unwrap_or_default();
            out.push(format!("{last}.{}", tokens[i + 1]));
            i += 2;
        } else {
            out.push(tokens[i].clone());
            i += 1;
        }
    }
    out
}

/// Spoken numbers to digits, spoken decimals, canonical digit groups, adjacent digits joined,
/// percent unified.
pub fn fold_numbers(tokens: &[String]) -> Vec<String> {
    let folded = merge_spoken_decimals(fold_spoken(&unify_percent(tokens)));
    let canonical: Vec<String> = folded.into_iter().map(|t| canonical_number_token(&t)).collect();
    join_adjacent_digits(canonical)
}

/// Canonical form of a unit written or spoken in several ways ("milligrammes" -> "mg").
pub fn canonical_unit(token: &str) -> Option<&'static str> {
    Some(match token {
        "mg" | "milligramme" | "milligrammes" | "milligram" | "milligrams" => "mg",
        "g" | "gramme" | "grammes" | "gram" | "grams" => "g",
        "kg" | "kilo" | "kilos" | "kilogramme" | "kilogrammes" | "kilogram" | "kilograms" => "kg",
        "µg" | "ug" | "mcg" | "microgramme" | "microgrammes" | "microgram" | "micrograms" => "µg",
        "ml" | "millilitre" | "millilitres" | "milliliter" | "milliliters" => "ml",
        "cl" | "centilitre" | "centilitres" => "cl",
        "l" | "litre" | "litres" | "liter" | "liters" => "l",
        "%" => "%",
        "h" | "heure" | "heures" | "hour" | "hours" => "h",
        "mmhg" => "mmhg",
        "cm" | "centimètre" | "centimètres" | "centimeter" | "centimeters" => "cm",
        "mm" | "millimètre" | "millimètres" | "millimeter" | "millimeters" => "mm",
        "m" | "mètre" | "mètres" | "meter" | "meters" => "m",
        _ => return None,
    })
}

/// Replaces unit spellings by their canonical form (after number folding).
pub fn unify_units(tokens: Vec<String>) -> Vec<String> {
    tokens
        .into_iter()
        .map(|t| canonical_unit(&t).map(str::to_string).unwrap_or(t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fold(s: &str) -> String {
        let t: Vec<String> = s.split_whitespace().map(str::to_string).collect();
        fold_numbers(&t).join(" ")
    }

    #[test]
    fn french_simple_and_compound_numbers() {
        assert_eq!(fold("quinze heures trente"), "15 heures 30");
        assert_eq!(fold("vingt et un patients"), "21 patients");
        assert_eq!(fold("soixante dix"), "70");
        assert_eq!(fold("soixante et onze"), "71");
        assert_eq!(fold("quatre vingt dix"), "90");
        assert_eq!(fold("quatre vingts"), "80");
        assert_eq!(fold("cinq cents milligrammes"), "500 milligrammes");
        assert_eq!(fold("deux cent trente"), "230");
        assert_eq!(fold("deux mille vingt six"), "2026");
        assert_eq!(fold("cent"), "100");
        assert_eq!(fold("mille"), "1000");
    }

    #[test]
    fn english_numbers_follow_the_grammar() {
        assert_eq!(fold("twenty one"), "21");
        assert_eq!(fold("one hundred and five"), "105");
        assert_eq!(fold("three times a day"), "3 times a day");
        assert_eq!(fold("two thousand twenty six"), "2026");
        assert_eq!(fold("twelve thousand francs"), "12000 francs");
    }

    #[test]
    fn two_adjacent_numbers_that_do_not_combine_stay_two_numbers() {
        // "ten thirty" is a time (10:30), not 40. Joined digits then equal "1030".
        assert_eq!(fold("ten thirty"), "1030");
        assert_eq!(fold("10 30"), "1030");
        assert_eq!(fold("trois un"), "3 un");
        assert_eq!(fold("quinze trente"), "1530");
    }

    #[test]
    fn digit_groups_are_canonical_but_values_stay_distinct() {
        assert_eq!(fold("12,000"), "12000");
        assert_eq!(fold("12.000"), "12000");
        assert_eq!(fold("12000"), "12000");
        assert_eq!(fold("1,000,000"), "1000000");
        assert_eq!(fold("2,5"), "2.5");
        assert_eq!(fold("2.5"), "2.5");
        assert_eq!(fold("05"), "5");
        assert_eq!(fold("0,5"), "0.5");
        assert_ne!(fold("2,5"), fold("25"));
        assert_ne!(fold("2,5"), fold("2,500"));
        assert_ne!(fold("500"), fold("50"));
    }

    #[test]
    fn articles_and_pronouns_are_not_numbers() {
        assert_eq!(fold("un patient"), "un patient");
        assert_eq!(fold("une question"), "une question");
        assert_eq!(fold("one small step"), "one small step");
    }

    #[test]
    fn digits_are_untouched_and_different_numbers_stay_different() {
        assert_eq!(fold("500 mg"), "500 mg");
        assert_ne!(fold("quinze"), fold("cinquante"));
        assert_eq!(fold("quinze"), "15");
        assert_eq!(fold("cinquante"), "50");
    }

    #[test]
    fn french_seventeen_to_nineteen_and_spoken_decimals() {
        assert_eq!(fold("dix sept"), "17");
        assert_eq!(fold("dix huit"), "18");
        assert_eq!(fold("dix neuf"), "19");
        assert_eq!(fold("soixante dix sept"), "77");
        assert_eq!(fold("quatre vingt dix sept"), "97");
        assert_eq!(fold("deux virgule cinq milligrammes"), "2.5 milligrammes");
        assert_eq!(fold("zéro virgule cinq"), "0.5");
        assert_eq!(fold("two point five"), "2.5");
        assert_eq!(fold("le point de vue"), "le point de vue");
        assert_ne!(fold("dix huit"), fold("dix"));
    }

    #[test]
    fn percent_words_become_the_percent_sign() {
        assert_eq!(fold("vingt pour cent"), "20 %");
        assert_eq!(fold("ten percent"), "10 %");
        assert_eq!(fold("cent pour cent"), "100 %");
    }

    #[test]
    fn unit_spellings_are_unified_but_different_units_stay_different() {
        let v = |s: &str| unify_units(s.split_whitespace().map(str::to_string).collect()).join(" ");
        assert_eq!(v("500 milligrammes"), "500 mg");
        assert_eq!(v("2 grammes"), "2 g");
        assert_ne!(v("5 mg"), v("5 g"));
    }
}
