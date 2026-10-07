//! Spoken-number folding: "quinze" / "cinq cents" / "twenty one" -> digits.
//!
//! Used on BOTH the reference and the hypothesis before scoring, so "15" and "quinze" count as
//! the same thing, while "15" and "50" stay different. A standalone "un"/"une"/"one" is left
//! alone (it is usually an article or a pronoun). Supported: French and English up to 999 999,
//! including "vingt et un", "soixante dix", "quatre vingt dix", "cinq cents",
//! "one hundred and five". Anything the folder does not understand is left untouched, which
//! can only produce an extra reported difference, never hide one.

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

fn is_number_word(w: &str, lang: Lang) -> bool {
    match lang {
        Lang::Fr => fr_value(w).is_some() || matches!(w, "cent" | "cents" | "mille"),
        Lang::En => en_value(w).is_some() || matches!(w, "hundred" | "thousand"),
    }
}

fn is_article_like(w: &str) -> bool {
    matches!(w, "un" | "une" | "one")
}

/// Evaluates a run of number words. Returns None if the run is not a coherent number.
fn evaluate(run: &[&str], lang: Lang) -> Option<u32> {
    let mut total: u32 = 0;
    let mut current: u32 = 0;
    for w in run {
        match lang {
            Lang::Fr => match *w {
                "cent" | "cents" => current = current.max(1) * 100,
                "mille" => {
                    total += current.max(1) * 1000;
                    current = 0;
                }
                "vingt" | "vingts" if current == 4 => current = 80, // quatre vingt
                other => current += fr_value(other)?,
            },
            Lang::En => match *w {
                "hundred" => current = current.max(1) * 100,
                "thousand" => {
                    total += current.max(1) * 1000;
                    current = 0;
                }
                other => current += en_value(other)?,
            },
        }
    }
    Some(total + current)
}

/// Replaces spoken numbers by digits in a token list (already lower-cased by the caller).
pub fn fold_numbers(tokens: &[String]) -> Vec<String> {
    // "pour cent" / "per cent" / "percent" -> "%" (before "cent" would be read as 100).
    let mut unified: Vec<String> = Vec::with_capacity(tokens.len());
    let mut k = 0;
    while k < tokens.len() {
        let pair = tokens.get(k + 1).map(String::as_str);
        match (tokens[k].as_str(), pair) {
            ("pour" | "per", Some("cent")) => {
                unified.push("%".into());
                k += 2;
            }
            ("pourcent" | "percent", _) => {
                unified.push("%".into());
                k += 1;
            }
            _ => {
                unified.push(tokens[k].clone());
                k += 1;
            }
        }
    }
    let tokens = &unified[..];
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let w = tokens[i].as_str();
        let lang = if is_number_word(w, Lang::Fr) {
            Some(Lang::Fr)
        } else if is_number_word(w, Lang::En) {
            Some(Lang::En)
        } else {
            None
        };
        let Some(lang) = lang else {
            out.push(tokens[i].clone());
            i += 1;
            continue;
        };
        // Collect the run: number words, with "et"/"and" allowed between two number words.
        let mut run: Vec<&str> = Vec::new();
        let mut j = i;
        while j < tokens.len() {
            let t = tokens[j].as_str();
            if is_number_word(t, lang) {
                run.push(t);
                j += 1;
            } else if matches!(t, "et" | "and")
                && !run.is_empty()
                && tokens.get(j + 1).is_some_and(|n| is_number_word(n, lang))
            {
                j += 1;
            } else {
                break;
            }
        }
        if run.len() == 1 && is_article_like(run[0]) {
            out.push(tokens[i].clone());
            i = j;
            continue;
        }
        match evaluate(&run, lang) {
            Some(v) => out.push(v.to_string()),
            None => out.extend(tokens[i..j].iter().cloned()),
        }
        i = j;
    }
    out
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
        assert_eq!(fold("quatre vingt dix"), "90");
        assert_eq!(fold("cinq cents milligrammes"), "500 milligrammes");
        assert_eq!(fold("deux mille vingt six"), "2026");
        assert_eq!(fold("cent"), "100");
    }

    #[test]
    fn english_numbers() {
        assert_eq!(fold("twenty one"), "21");
        assert_eq!(fold("one hundred and five"), "105");
        assert_eq!(fold("three times a day"), "3 times a day");
        assert_eq!(fold("two thousand twenty six"), "2026");
    }

    #[test]
    fn unit_spellings_are_unified_but_different_units_stay_different() {
        let v = |s: &str| unify_units(s.split_whitespace().map(str::to_string).collect()).join(" ");
        assert_eq!(v("500 milligrammes"), "500 mg");
        assert_eq!(v("2 grammes"), "2 g");
        assert_ne!(v("5 mg"), v("5 g"));
    }

    #[test]
    fn percent_words_become_the_percent_sign() {
        assert_eq!(fold("vingt pour cent"), "20 %");
        assert_eq!(fold("ten percent"), "10 %");
        assert_eq!(fold("cent pour cent"), "100 %");
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
}
