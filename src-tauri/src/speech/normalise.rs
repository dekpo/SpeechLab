//! Text normaliser for character-based voices (M6d, D-042).
//!
//! Some voices (the Coqui css10 French voice) read the characters they are given and say nothing for
//! digits, symbols or abbreviations. This module rewrites a SENTENCE into words just before synthesis.
//! The displayed text is never changed. It is switched on per voice by the manifest flag
//! `normalizeText`; phonemizer voices already speak digits and are left alone.
//!
//! Hand-written scanner, no dependency. What it does (French and English):
//! - cardinals (French 70/80/90 as "soixante-dix", "quatre-vingts" with the plural only when nothing
//!   follows, "quatre-vingt-un", "et un"/"et onze" forms), ordinals (1er, 2e, 12th), decimals,
//!   thousands separators (1 000, 1.000 in French, 1,000 in English), negatives, ranges (5-10)
//! - percentages, currency (€ $ £ CHF), times (14 h 30, 9h00, 14:30, 10.30 after a time word, 9 am),
//!   dates (12/03/2026, 12 mars 2026, March 12th, 2026), fractions (1/2, 3/4), "3 x par jour"
//! - units (mg, ml, kg, °C, mmHg, bpm, UI, mg/kg, mg/j ...) with the singular/plural rule of the language
//! - abbreviations (M., Mme, Dr, Pr, St, etc., n°, vs, e.g. ...)
//! - phone numbers and identifiers: groups of two digits are read as numbers ("06 12 34" ->
//!   "zéro six, douze, trente-quatre"), other groups digit by digit; long digit strings digit by digit
//!
//! Known limits (recorded in D-042): grammatical gender of "un/une" is a short list of common feminine
//! nouns; acronyms (IRM, ECG) are not spelled; Roman numerals are not converted; "10.30" is a time only
//! after a time word; an invoice number such as "numéro 2045" is read as a cardinal.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Fr,
    En,
}

impl Lang {
    pub fn from_code(code: &str) -> Option<Lang> {
        match code {
            "fr" => Some(Lang::Fr),
            "en" => Some(Lang::En),
            _ => None,
        }
    }
}

/// Normalises `text` for the language code ("fr" or "en"); any other code returns the text unchanged.
pub fn normalise_for(text: &str, language: &str) -> String {
    match Lang::from_code(language) {
        Some(lang) => normalise(text, lang),
        None => text.to_string(),
    }
}

pub fn normalise(text: &str, lang: Lang) -> String {
    let mut n = Norm::new(text, lang);
    n.run();
    collapse_spaces(&n.out)
}

fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for ch in s.chars() {
        let is_space = ch == ' ';
        if !(is_space && last_space) {
            out.push(ch);
        }
        last_space = is_space;
    }
    out.trim().to_string()
}

// ------------------------------------------------------------------------------------------------
// Number words
// ------------------------------------------------------------------------------------------------

const FR_UNITS: [&str; 17] = [
    "zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze", "douze", "treize",
    "quatorze", "quinze", "seize",
];
const FR_TENS: [&str; 7] = ["", "", "vingt", "trente", "quarante", "cinquante", "soixante"];

fn fr_lt100(n: u32, plural: bool) -> String {
    match n {
        0..=16 => FR_UNITS[n as usize].to_string(),
        17..=19 => format!("dix-{}", FR_UNITS[(n - 10) as usize]),
        20..=69 => {
            let (t, u) = ((n / 10) as usize, n % 10);
            match u {
                0 => FR_TENS[t].to_string(),
                1 => format!("{} et un", FR_TENS[t]),
                _ => format!("{}-{}", FR_TENS[t], FR_UNITS[u as usize]),
            }
        }
        71 => "soixante et onze".to_string(),
        70..=79 => format!("soixante-{}", fr_lt100(n - 60, plural)),
        80 => if plural { "quatre-vingts" } else { "quatre-vingt" }.to_string(),
        _ => format!("quatre-vingt-{}", fr_lt100(n - 80, plural)),
    }
}

fn fr_lt1000(n: u32, plural: bool) -> String {
    let (h, r) = (n / 100, n % 100);
    if h == 0 {
        return fr_lt100(r, plural);
    }
    let mut s = if h == 1 { "cent".to_string() } else { format!("{} cent", FR_UNITS[h as usize]) };
    if r == 0 {
        if h > 1 && plural {
            s.push('s');
        }
    } else {
        s.push(' ');
        s.push_str(&fr_lt100(r, plural));
    }
    s
}

/// French cardinal below one trillion. `plural` = "quatre-vingts" / "deux cents" take their s at the end.
fn fr_cardinal_with(n: u64, plural: bool) -> String {
    if n == 0 {
        return "zéro".to_string();
    }
    let (b, m, t, r) = ((n / 1_000_000_000) as u32, ((n / 1_000_000) % 1000) as u32, ((n / 1000) % 1000) as u32, (n % 1000) as u32);
    let mut parts: Vec<String> = Vec::new();
    if b > 0 {
        parts.push(format!("{} milliard{}", fr_lt1000(b, true), if b > 1 { "s" } else { "" }));
    }
    if m > 0 {
        parts.push(format!("{} million{}", fr_lt1000(m, true), if m > 1 { "s" } else { "" }));
    }
    if t > 0 {
        parts.push(if t == 1 { "mille".to_string() } else { format!("{} mille", fr_lt1000(t, false)) });
    }
    if r > 0 {
        parts.push(fr_lt1000(r, plural));
    }
    parts.join(" ")
}

fn fr_cardinal(n: u64) -> String {
    fr_cardinal_with(n, true)
}

fn fr_feminine(mut s: String) -> String {
    if s == "un" || s.ends_with(" un") || s.ends_with("-un") {
        s.push('e');
    }
    s
}

/// Last word of a number name (after the last space or hyphen) and what precedes it.
fn split_last_word(s: &str) -> (&str, &str) {
    match s.rfind([' ', '-']) {
        Some(p) => (&s[..=p], &s[p + 1..]),
        None => ("", s),
    }
}

fn fr_ordinal(n: u64, feminine: bool) -> String {
    if n == 1 {
        return if feminine { "première" } else { "premier" }.to_string();
    }
    let base = fr_cardinal_with(n, false);
    let (head, last) = split_last_word(&base);
    let ord = match last {
        "cinq" => "cinquième".to_string(),
        "neuf" => "neuvième".to_string(),
        w if w.ends_with('e') => format!("{}ième", &w[..w.len() - 1]),
        w => format!("{w}ième"),
    };
    format!("{head}{ord}")
}

const EN_ONES: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen",
];
const EN_TENS: [&str; 10] = ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];

fn en_lt100(n: u32) -> String {
    if n < 20 {
        return EN_ONES[n as usize].to_string();
    }
    let (t, u) = ((n / 10) as usize, n % 10);
    if u == 0 { EN_TENS[t].to_string() } else { format!("{}-{}", EN_TENS[t], EN_ONES[u as usize]) }
}

fn en_lt1000(n: u32) -> String {
    let (h, r) = (n / 100, n % 100);
    match (h, r) {
        (0, _) => en_lt100(r),
        (_, 0) => format!("{} hundred", EN_ONES[h as usize]),
        _ => format!("{} hundred {}", EN_ONES[h as usize], en_lt100(r)),
    }
}

fn en_cardinal(n: u64) -> String {
    if n == 0 {
        return "zero".to_string();
    }
    let (b, m, t, r) = ((n / 1_000_000_000) as u32, ((n / 1_000_000) % 1000) as u32, ((n / 1000) % 1000) as u32, (n % 1000) as u32);
    let mut parts: Vec<String> = Vec::new();
    if b > 0 {
        parts.push(format!("{} billion", en_lt1000(b)));
    }
    if m > 0 {
        parts.push(format!("{} million", en_lt1000(m)));
    }
    if t > 0 {
        parts.push(format!("{} thousand", en_lt1000(t)));
    }
    if r > 0 {
        parts.push(en_lt1000(r));
    }
    parts.join(" ")
}

fn en_ordinal(n: u64) -> String {
    let base = en_cardinal(n);
    let (head, last) = split_last_word(&base);
    let ord = match last {
        "one" => "first".to_string(),
        "two" => "second".to_string(),
        "three" => "third".to_string(),
        "five" => "fifth".to_string(),
        "eight" => "eighth".to_string(),
        "nine" => "ninth".to_string(),
        "twelve" => "twelfth".to_string(),
        w if w.ends_with('y') => format!("{}ieth", &w[..w.len() - 1]),
        w => format!("{w}th"),
    };
    format!("{head}{ord}")
}

/// English year: 2026 -> "twenty twenty-six", 1905 -> "nineteen oh five", 2005 -> "two thousand five".
fn en_year(y: u64) -> String {
    if y % 1000 == 0 {
        return en_cardinal(y);
    }
    if (2001..=2009).contains(&y) {
        return en_cardinal(y);
    }
    let (hi, lo) = (y / 100, y % 100);
    match lo {
        0 => format!("{} hundred", en_lt100(hi as u32)),
        1..=9 => format!("{} oh {}", en_lt100(hi as u32), en_lt100(lo as u32)),
        _ => format!("{} {}", en_lt100(hi as u32), en_lt100(lo as u32)),
    }
}

fn digit_word(d: char, lang: Lang) -> &'static str {
    let i = d.to_digit(10).unwrap_or(0) as usize;
    match lang {
        Lang::Fr => ["zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf"][i],
        Lang::En => ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"][i],
    }
}

fn digits_one_by_one(digits: &str, lang: Lang) -> String {
    digits.chars().filter(|c| c.is_ascii_digit()).map(|c| digit_word(c, lang)).collect::<Vec<_>>().join(" ")
}

const FR_MONTHS: [&str; 12] = [
    "janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre",
];
const EN_MONTHS: [&str; 12] = [
    "january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december",
];

fn month_index(word: &str, lang: Lang) -> Option<usize> {
    let w = word.to_lowercase();
    let list = if lang == Lang::Fr { &FR_MONTHS } else { &EN_MONTHS };
    list.iter().position(|m| *m == w)
}

/// Nouns after which a French "1" is feminine ("une fois", "une gélule"). Anything else is masculine.
const FR_FEMININE_NOUNS: &[&str] = &[
    "fois", "heure", "heures", "minute", "minutes", "seconde", "secondes", "semaine", "semaines", "année", "années",
    "journée", "journées", "gélule", "gélules", "ampoule", "ampoules", "dose", "doses", "personne", "personnes",
    "facture", "factures", "page", "pages", "ligne", "lignes", "pièce", "pièces", "ordonnance", "ordonnances",
    "consultation", "consultations", "goutte", "gouttes", "injection", "injections", "cuillère", "cuillères",
    "prise", "prises", "boîte", "boîtes", "unité", "unités", "carte", "cartes", "tablette", "tablettes",
    "chambre", "chambres", "fois-ci", "nuit", "nuits", "dent", "dents", "pilule", "pilules", "capsule", "capsules",
];

// ------------------------------------------------------------------------------------------------
// Units and currencies
// ------------------------------------------------------------------------------------------------

/// (symbols, French singular, French plural, English singular, English plural)
type UnitDef = (&'static [&'static str], &'static str, &'static str, &'static str, &'static str);

const UNITS: &[UnitDef] = &[
    (&["mg"], "milligramme", "milligrammes", "milligram", "milligrams"),
    (&["g"], "gramme", "grammes", "gram", "grams"),
    (&["kg"], "kilogramme", "kilogrammes", "kilogram", "kilograms"),
    (&["µg", "μg", "mcg"], "microgramme", "microgrammes", "microgram", "micrograms"),
    (&["ml", "mL"], "millilitre", "millilitres", "milliliter", "milliliters"),
    (&["l", "L"], "litre", "litres", "liter", "liters"),
    (&["cl"], "centilitre", "centilitres", "centiliter", "centiliters"),
    (&["dl"], "décilitre", "décilitres", "deciliter", "deciliters"),
    (&["mm"], "millimètre", "millimètres", "millimeter", "millimeters"),
    (&["cm"], "centimètre", "centimètres", "centimeter", "centimeters"),
    (&["m"], "mètre", "mètres", "meter", "meters"),
    (&["km"], "kilomètre", "kilomètres", "kilometer", "kilometers"),
    (&["ms"], "milliseconde", "millisecondes", "millisecond", "milliseconds"),
    (&["s"], "seconde", "secondes", "second", "seconds"),
    (&["min"], "minute", "minutes", "minute", "minutes"),
    (&["h"], "heure", "heures", "hour", "hours"),
    (&["j"], "jour", "jours", "day", "days"),
    (&["UI", "IU"], "unité internationale", "unités internationales", "international unit", "international units"),
    (&["mmHg"], "millimètre de mercure", "millimètres de mercure", "millimeter of mercury", "millimeters of mercury"),
    (&["bpm"], "battement par minute", "battements par minute", "beat per minute", "beats per minute"),
    (&["kcal"], "kilocalorie", "kilocalories", "kilocalorie", "kilocalories"),
    (&["mmol"], "millimole", "millimoles", "millimole", "millimoles"),
    (&["mol"], "mole", "moles", "mole", "moles"),
    (&["Hz"], "hertz", "hertz", "hertz", "hertz"),
    (&["kHz"], "kilohertz", "kilohertz", "kilohertz", "kilohertz"),
    (&["MHz"], "mégahertz", "mégahertz", "megahertz", "megahertz"),
    (&["GHz"], "gigahertz", "gigahertz", "gigahertz", "gigahertz"),
    (&["Ko", "KB", "kB"], "kilooctet", "kilooctets", "kilobyte", "kilobytes"),
    (&["Mo", "MB"], "mégaoctet", "mégaoctets", "megabyte", "megabytes"),
    (&["Go", "GB"], "gigaoctet", "gigaoctets", "gigabyte", "gigabytes"),
    (&["To", "TB"], "téraoctet", "téraoctets", "terabyte", "terabytes"),
    (&["cp"], "comprimé", "comprimés", "tablet", "tablets"),
    (&["lb", "lbs"], "livre", "livres", "pound", "pounds"),
    (&["oz"], "once", "onces", "ounce", "ounces"),
];

/// Words that can follow a "/" in a rate ("2 fois/jour", "mg/semaine").
const PER_WORDS: &[(&str, &str, &str)] = &[
    ("jour", "jour", "day"),
    ("semaine", "semaine", "week"),
    ("mois", "mois", "month"),
    ("day", "jour", "day"),
    ("week", "semaine", "week"),
    ("month", "mois", "month"),
    ("heure", "heure", "hour"),
    ("hour", "heure", "hour"),
];

fn find_unit(symbol: &str) -> Option<&'static UnitDef> {
    UNITS.iter().find(|u| u.0.contains(&symbol))
}

fn unit_name(u: &UnitDef, plural: bool, lang: Lang) -> &'static str {
    match (lang, plural) {
        (Lang::Fr, false) => u.1,
        (Lang::Fr, true) => u.2,
        (Lang::En, false) => u.3,
        (Lang::En, true) => u.4,
    }
}

/// (symbols and codes, French singular, plural, English singular, plural, French cents, English cents)
type CurrencyDef = (&'static [&'static str], &'static str, &'static str, &'static str, &'static str, &'static str, &'static str);

const CURRENCIES: &[CurrencyDef] = &[
    (&["€", "EUR"], "euro", "euros", "euro", "euros", "centime", "cent"),
    (&["$", "USD"], "dollar", "dollars", "dollar", "dollars", "cent", "cent"),
    (&["£", "GBP"], "livre sterling", "livres sterling", "pound", "pounds", "penny", "pence"),
    (&["CHF"], "franc", "francs", "Swiss franc", "Swiss francs", "centime", "centime"),
];

fn find_currency(symbol: &str) -> Option<&'static CurrencyDef> {
    CURRENCIES.iter().find(|c| c.0.contains(&symbol))
}

const CURRENCY_WORDS: &[&str] = &["euro", "euros", "franc", "francs", "dollar", "dollars", "livre", "livres"];

// ------------------------------------------------------------------------------------------------
// Abbreviations
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Need {
    Nothing,
    /// The next word must start with a capital letter (a name): "Dr Martin".
    Name,
    /// A digit must follow: "p. 12".
    Digit,
}

/// (written form, French, English, condition). An empty expansion = not for that language.
/// Forms that start in lower case also match with a capital first letter.
const ABBREVIATIONS: &[(&str, &str, &str, Need)] = &[
    ("av. J.-C.", "avant Jésus-Christ", "", Need::Nothing),
    ("c.-à-d.", "c'est-à-dire", "", Need::Nothing),
    ("e.g.", "", "for example", Need::Nothing),
    ("i.e.", "", "that is", Need::Nothing),
    ("M.", "monsieur", "", Need::Nothing),
    ("MM.", "messieurs", "", Need::Nothing),
    ("Mme", "madame", "", Need::Nothing),
    ("Mmes", "mesdames", "", Need::Nothing),
    ("Mlle", "mademoiselle", "", Need::Nothing),
    ("Mlles", "mesdemoiselles", "", Need::Nothing),
    ("Dr.", "docteur", "doctor", Need::Nothing),
    ("Dr", "docteur", "doctor", Need::Name),
    ("Pr.", "professeur", "professor", Need::Nothing),
    ("Pr", "professeur", "", Need::Name),
    ("Prof.", "professeur", "professor", Need::Nothing),
    ("Prof", "professeur", "professor", Need::Name),
    ("St.", "saint", "saint", Need::Nothing),
    ("St", "saint", "saint", Need::Name),
    ("Ste.", "sainte", "", Need::Nothing),
    ("Ste", "sainte", "", Need::Name),
    ("Mr.", "", "mister", Need::Nothing),
    ("Mr", "", "mister", Need::Name),
    ("Mrs.", "", "missus", Need::Nothing),
    ("Mrs", "", "missus", Need::Name),
    ("Ms.", "", "miss", Need::Nothing),
    ("Ms", "", "miss", Need::Name),
    ("Jr.", "", "junior", Need::Nothing),
    ("Sr.", "", "senior", Need::Nothing),
    ("etc.", "et cetera", "et cetera", Need::Nothing),
    ("env.", "environ", "", Need::Nothing),
    ("approx.", "environ", "approximately", Need::Nothing),
    ("vs.", "versus", "versus", Need::Nothing),
    ("vs", "versus", "versus", Need::Nothing),
    ("cf.", "voir", "see", Need::Nothing),
    ("p.", "page", "page", Need::Digit),
    ("pp.", "pages", "pages", Need::Digit),
    ("tél.", "téléphone", "", Need::Nothing),
    ("tel.", "téléphone", "telephone", Need::Nothing),
    ("av.", "avenue", "avenue", Need::Name),
    ("bd", "boulevard", "", Need::Name),
    ("rdv", "rendez-vous", "appointment", Need::Nothing),
    ("n°", "numéro", "number", Need::Nothing),
    ("nº", "numéro", "number", Need::Nothing),
    ("no.", "", "number", Need::Digit),
    ("a.m.", "", "a m", Need::Nothing),
    ("p.m.", "", "p m", Need::Nothing),
    ("janv.", "janvier", "", Need::Nothing),
    ("févr.", "février", "", Need::Nothing),
    ("avr.", "avril", "", Need::Nothing),
    ("juil.", "juillet", "", Need::Nothing),
    ("sept.", "septembre", "September", Need::Nothing),
    ("oct.", "octobre", "October", Need::Nothing),
    ("nov.", "novembre", "November", Need::Nothing),
    ("déc.", "décembre", "", Need::Nothing),
    ("Jan.", "", "January", Need::Nothing),
    ("Feb.", "", "February", Need::Nothing),
    ("Mar.", "", "March", Need::Nothing),
    ("Apr.", "", "April", Need::Nothing),
    ("Jun.", "", "June", Need::Nothing),
    ("Jul.", "", "July", Need::Nothing),
    ("Aug.", "", "August", Need::Nothing),
    ("Sep.", "", "September", Need::Nothing),
    ("Dec.", "", "December", Need::Nothing),
];

// ------------------------------------------------------------------------------------------------
// Scanner
// ------------------------------------------------------------------------------------------------

fn is_blank(c: char) -> bool {
    c == ' ' || c == '\u{a0}' || c == '\u{202f}'
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic()
}

struct Num {
    /// Integer digits without separators.
    int: String,
    frac: Option<String>,
    end: usize,
}

impl Num {
    fn value(&self) -> Option<u64> {
        (self.int.len() <= 12).then(|| self.int.parse().ok()).flatten()
    }
    fn frac_is_zero(&self) -> bool {
        self.frac.as_deref().map(|f| f.chars().all(|c| c == '0')).unwrap_or(true)
    }
}

struct Norm {
    c: Vec<char>,
    lang: Lang,
    out: String,
    i: usize,
    /// Last word written (lower case); cleared after a number. Used for "à 10.30" and French "1 fois".
    last_word: String,
    /// English dates: 1 after a month name (a day or a year may follow), 2 after the day (a year may follow).
    month_stage: u8,
    /// The previous piece was a spoken number: a letter right after it needs a space.
    just_spoke: bool,
}

impl Norm {
    fn new(text: &str, lang: Lang) -> Norm {
        // Non-breaking spaces are ordinary spaces for the scanner.
        let c = text.chars().map(|c| if c == '\u{a0}' || c == '\u{202f}' { ' ' } else { c }).collect();
        Norm { c, lang, out: String::new(), i: 0, last_word: String::new(), month_stage: 0, just_spoke: false }
    }

    fn n(&self) -> usize {
        self.c.len()
    }

    fn at(&self, p: usize) -> Option<char> {
        self.c.get(p).copied()
    }

    fn digits_len(&self, p: usize) -> usize {
        let mut q = p;
        while q < self.n() && self.c[q].is_ascii_digit() {
            q += 1;
        }
        q - p
    }

    fn digits(&self, p: usize, len: usize) -> String {
        self.c[p..p + len].iter().collect()
    }

    fn word_end(&self, p: usize) -> usize {
        let mut q = p;
        while q < self.n() && (is_word_char(self.c[q])) {
            q += 1;
        }
        q
    }

    fn boundary_after(&self, p: usize) -> bool {
        match self.at(p) {
            None => true,
            Some(ch) => !(ch.is_alphanumeric() || ch == '\'' || ch == '’'),
        }
    }

    fn rest_is_blank(&self, p: usize) -> bool {
        self.c[p.min(self.n())..].iter().all(|c| c.is_whitespace())
    }

    fn skip_one_blank(&self, p: usize) -> usize {
        if self.at(p).map(is_blank).unwrap_or(false) { p + 1 } else { p }
    }

    fn matches_at(&self, p: usize, lit: &str) -> bool {
        let lit: Vec<char> = lit.chars().collect();
        p + lit.len() <= self.n() && self.c[p..p + lit.len()] == lit[..]
    }

    fn matches_ci_at(&self, p: usize, lit: &str) -> bool {
        let lit: Vec<char> = lit.chars().collect();
        p + lit.len() <= self.n() && self.c[p..p + lit.len()].iter().zip(lit.iter()).all(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
    }

    // ---- output helpers -------------------------------------------------------------------------

    /// Appends spoken words, with a space before them when they would stick to a letter or digit.
    fn speak(&mut self, s: &str) {
        if let Some(last) = self.out.chars().last() {
            if last.is_alphanumeric() {
                self.out.push(' ');
            }
        }
        self.out.push_str(s);
        self.just_spoke = true;
        self.last_word.clear();
    }

    fn copy(&mut self, ch: char) {
        if self.just_spoke && ch.is_alphanumeric() {
            self.out.push(' ');
        }
        self.just_spoke = false;
        self.out.push(ch);
    }

    fn cardinal(&self, n: u64) -> String {
        match self.lang {
            Lang::Fr => fr_cardinal(n),
            Lang::En => en_cardinal(n),
        }
    }

    // ---- main loop ------------------------------------------------------------------------------

    fn run(&mut self) {
        while self.i < self.n() {
            let ch = self.c[self.i];
            let prev = if self.i > 0 { Some(self.c[self.i - 1]) } else { None };
            let prev_alnum = prev.map(|p| p.is_alphanumeric()).unwrap_or(false);
            let next_is_digit = self.at(self.i + 1).map(|c| c.is_ascii_digit()).unwrap_or(false);

            if ch.is_ascii_digit() {
                if prev_alnum {
                    let len = self.digits_len(self.i);
                    let word = self.read_cardinal_digits(&self.digits(self.i, len));
                    self.speak(&word);
                    self.i += len;
                } else {
                    self.handle_digit(self.i, false);
                }
            } else if (ch == '-' || ch == '−') && next_is_digit && prev.map(|p| p.is_whitespace() || p == '(').unwrap_or(true) {
                self.handle_digit(self.i + 1, true);
            } else if ch == '+' && next_is_digit && !prev_alnum {
                if let Some((spoken, end)) = self.try_phone(self.i) {
                    self.speak(&spoken);
                    self.i = end;
                } else {
                    self.speak("plus");
                    self.i += 1;
                }
            } else if matches!(ch, '€' | '$' | '£') && self.currency_prefix(self.i) {
                // handled
            } else if ch == '&' && prev.map(|p| p.is_whitespace()).unwrap_or(true) && self.at(self.i + 1).map(|c| c.is_whitespace()).unwrap_or(true) {
                self.speak(if self.lang == Lang::Fr { "et" } else { "and" });
                self.i += 1;
            } else if ch == '/' && self.try_per_word() {
                // handled
            } else if ch.is_alphabetic() && !prev_alnum {
                self.handle_word();
            } else {
                if !(ch == ',' || ch.is_whitespace()) {
                    self.month_stage = 0;
                }
                self.copy(ch);
                self.i += 1;
            }
        }
    }

    // ---- words ----------------------------------------------------------------------------------

    fn handle_word(&mut self) {
        let p = self.i;
        let lang = self.lang;
        // Abbreviation table, longest written form first.
        let mut best: Option<(usize, &str, usize)> = None; // (form length, expansion, end)
        for (form, fr, en, need) in ABBREVIATIONS {
            let expansion = if lang == Lang::Fr { *fr } else { *en };
            if expansion.is_empty() {
                continue;
            }
            let flen = form.chars().count();
            let exact = self.matches_at(p, form);
            let first = form.chars().next().unwrap();
            let capitalised = first.is_lowercase() && {
                let mut f2: Vec<char> = form.chars().collect();
                f2[0] = first.to_uppercase().next().unwrap_or(first);
                self.matches_at(p, &f2.iter().collect::<String>())
            };
            if !(exact || capitalised) {
                continue;
            }
            let end = p + flen;
            // The written form must end the word ("Dr" is not the start of "Drogue").
            let form_ends_with_alnum = form.chars().last().map(|c| c.is_alphanumeric()).unwrap_or(false);
            if form_ends_with_alnum && !self.boundary_after(end) {
                continue;
            }
            let ok = match need {
                Need::Nothing => true,
                Need::Name => {
                    let q = self.skip_one_blank(end);
                    q > end && self.at(q).map(|c| c.is_uppercase()).unwrap_or(false)
                }
                Need::Digit => {
                    let q = self.skip_one_blank(end);
                    self.at(q).map(|c| c.is_ascii_digit()).unwrap_or(false)
                }
            };
            if ok && best.map(|b| flen > b.0).unwrap_or(true) {
                best = Some((flen, expansion, end));
            }
        }
        if let Some((_, expansion, end)) = best {
            let dotted = self.c[end - 1] == '.';
            let sentence_end = dotted && self.rest_is_blank(end);
            self.speak(expansion);
            if sentence_end {
                self.out.push('.');
            }
            self.last_word = expansion.to_lowercase();
            if lang == Lang::En && month_index(expansion, lang).is_some() {
                self.month_stage = 1;
            }
            self.i = end;
            return;
        }

        // A plain word (letters, with inner apostrophes and hyphens).
        let mut q = p;
        while q < self.n() {
            let ch = self.c[q];
            let inner = (ch == '\'' || ch == '’' || ch == '-') && self.at(q + 1).map(|c| c.is_alphabetic()).unwrap_or(false);
            if ch.is_alphabetic() || inner {
                q += 1;
            } else {
                break;
            }
        }
        let word: String = self.c[p..q].iter().collect();
        for ch in word.chars() {
            self.copy(ch);
        }
        self.last_word = word.to_lowercase();
        self.month_stage = if lang == Lang::En && month_index(&word, lang).is_some() { 1 } else { 0 };
        self.i = q;
    }

    /// "/jour", "/day" after a count: "3 fois/jour".
    fn try_per_word(&mut self) -> bool {
        let p = self.i + 1;
        let e = self.word_end(p);
        if e == p || !self.boundary_after(e) {
            return false;
        }
        let w: String = self.c[p..e].iter().collect::<String>().to_lowercase();
        if let Some((_, fr, en)) = PER_WORDS.iter().find(|(k, _, _)| *k == w) {
            let name = if self.lang == Lang::Fr { *fr } else { *en };
            let word = if self.lang == Lang::Fr { "par" } else { "per" };
            if self.out.ends_with(|c: char| !c.is_whitespace()) {
                self.out.push(' ');
            }
            self.out.push_str(&format!("{word} {name}"));
            self.just_spoke = true;
            self.i = e;
            return true;
        }
        false
    }

    // ---- digits ---------------------------------------------------------------------------------

    /// Reads whatever starts with a digit at `p` (after the sign when `negative`) and moves `self.i`.
    fn handle_digit(&mut self, p: usize, negative: bool) {
        let stage = self.month_stage;
        if !negative {
            if let Some((s, e)) = self.try_numeric_date(p) {
                return self.finish(&s, e, 0);
            }
            if let Some((s, e)) = self.try_phone(p) {
                return self.finish(&s, e, 0);
            }
            if let Some((s, e)) = self.try_time(p) {
                return self.finish(&s, e, 0);
            }
            if let Some((s, e)) = self.try_fraction(p) {
                return self.finish(&s, e, 0);
            }
            if let Some((s, e)) = self.try_ordinal(p) {
                // An English day after a month name ("March 12th") lets a year follow.
                let next = if stage > 0 { 2 } else { 0 };
                return self.finish(&s, e, next);
            }
        }
        let (mut spoken, e) = self.number_with_tail(p);
        if negative {
            spoken = format!("{} {spoken}", if self.lang == Lang::Fr { "moins" } else { "minus" });
        }
        // The date logic inside `number_with_tail` moves the stage itself; otherwise the date context ends here.
        let next = if self.month_stage == stage { 0 } else { self.month_stage };
        self.finish(&spoken, e, next);
    }

    fn finish(&mut self, spoken: &str, end: usize, month_stage: u8) {
        self.speak(spoken);
        self.i = end;
        self.month_stage = month_stage;
    }

    fn read_cardinal_digits(&self, digits: &str) -> String {
        if digits.len() > 12 || (digits.len() > 1 && digits.starts_with('0')) {
            return digits_one_by_one(digits, self.lang);
        }
        self.cardinal(digits.parse().unwrap_or(0))
    }

    // ---- numbers --------------------------------------------------------------------------------

    fn parse_number(&self, p: usize) -> Option<Num> {
        let l0 = self.digits_len(p);
        if l0 == 0 {
            return None;
        }
        let mut int = self.digits(p, l0);
        let mut e = p + l0;
        // Thousands groups: exactly three digits after a first group of one to three, never "0".
        let grouping = l0 <= 3 && !int.starts_with('0');
        while grouping {
            let sep = match self.at(e) {
                Some(s) => s,
                None => break,
            };
            let is_sep = match self.lang {
                Lang::Fr => is_blank(sep) || sep == '.',
                Lang::En => sep == ',',
            };
            if is_sep && self.digits_len(e + 1) == 3 {
                int.push_str(&self.digits(e + 1, 3));
                e += 4;
            } else {
                break;
            }
        }
        let mut frac = None;
        if let Some(sep) = self.at(e) {
            let dec = match self.lang {
                Lang::Fr => sep == ',' || sep == '.',
                Lang::En => sep == '.',
            };
            if dec {
                let lf = self.digits_len(e + 1);
                if lf > 0 {
                    frac = Some(self.digits(e + 1, lf));
                    e += 1 + lf;
                }
            }
        }
        Some(Num { int, frac, end: e })
    }

    /// Plural rule of the unit that follows a quantity.
    fn plural(&self, num: &Num) -> bool {
        match self.lang {
            Lang::Fr => num.value().map(|v| v >= 2).unwrap_or(true),
            Lang::En => !(num.value() == Some(1) && num.frac_is_zero()),
        }
    }

    fn quantity_words(&self, num: &Num, feminine: bool) -> String {
        let mut s = match num.value() {
            Some(v) if !(num.int.len() > 1 && num.int.starts_with('0')) => {
                let w = self.cardinal(v);
                if feminine && self.lang == Lang::Fr && num.frac.is_none() { fr_feminine(w) } else { w }
            }
            _ => digits_one_by_one(&num.int, self.lang),
        };
        if let Some(f) = &num.frac {
            let sep = if self.lang == Lang::Fr { "virgule" } else { "point" };
            s.push(' ');
            s.push_str(sep);
            s.push(' ');
            s.push_str(&self.fraction_digits(f));
        }
        s
    }

    fn fraction_digits(&self, f: &str) -> String {
        if self.lang == Lang::En || f.len() > 3 {
            return digits_one_by_one(f, self.lang);
        }
        // French: leading zeros are said one by one, the rest as a number ("0,05" -> "zéro cinq").
        let zeros = f.chars().take_while(|c| *c == '0').count();
        let rest = &f[zeros..];
        let mut parts: Vec<String> = (0..zeros).map(|_| digit_word('0', self.lang).to_string()).collect();
        if !rest.is_empty() {
            parts.push(self.cardinal(rest.parse().unwrap_or(0)));
        }
        parts.join(" ")
    }

    /// A number with whatever follows it: percent, degrees, currency, unit, multiplier, range, day.
    fn number_with_tail(&mut self, p: usize) -> (String, usize) {
        let Some(first) = self.parse_number(p) else {
            return (String::new(), p + 1);
        };
        // Range "5-10" (no spaces): "cinq à dix".
        if matches!(self.at(first.end), Some('-') | Some('–')) && self.digits_len(first.end + 1) > 0 {
            if let Some(second) = self.parse_number(first.end + 1) {
                let a = self.quantity_words(&first, false);
                let (b, e) = self.tail(&second);
                let to = if self.lang == Lang::Fr { "à" } else { "to" };
                return (format!("{a} {to} {b}"), e);
            }
        }
        self.tail(&first)
    }

    fn tail(&mut self, num: &Num) -> (String, usize) {
        let lang = self.lang;
        let end = num.end;
        let q = self.skip_one_blank(end);

        // Percent.
        if self.at(q) == Some('%') {
            let pc = if lang == Lang::Fr { "pour cent" } else { "percent" };
            return (format!("{} {pc}", self.quantity_words(num, false)), q + 1);
        }
        // Degrees.
        if self.at(q) == Some('°') || self.at(q) == Some('º') {
            let r = self.skip_one_blank(q + 1);
            let (scale, e) = match self.at(r) {
                Some('C') if self.boundary_after(r + 1) => (" Celsius", r + 1),
                Some('F') if self.boundary_after(r + 1) => (" Fahrenheit", r + 1),
                _ => ("", q + 1),
            };
            let word = match (lang, self.plural(num)) {
                (Lang::Fr, false) => "degré",
                (Lang::Fr, true) => "degrés",
                (Lang::En, false) => "degree",
                (Lang::En, true) => "degrees",
            };
            return (format!("{} {word}{scale}", self.quantity_words(num, false)), e);
        }
        // Currency after the amount.
        if let Some((sym, e)) = self.currency_at(q) {
            if let Some(s) = self.currency_words(num, sym) {
                return (s, e);
            }
        }
        // Multiplier "3 x par jour", "2x500 mg".
        if matches!(self.at(q), Some('x') | Some('×')) && !self.at(q + 1).map(|c| c.is_alphabetic()).unwrap_or(false) {
            let times = if lang == Lang::Fr { "fois" } else { "times" };
            return (format!("{} {times}", self.quantity_words(num, false)), q + 1);
        }
        // Unit ("500 mg", "2,5 mg/kg", "5 min").
        if let Some((s, e)) = self.unit_at(num, q) {
            return (s, e);
        }
        // French "1 mars" -> "premier mars"; English "12 March" -> "twelfth March".
        if let Some(day) = num.value().filter(|d| (1..=31).contains(d) && num.frac.is_none()) {
            let r = self.skip_one_blank(end);
            if r > end {
                let e2 = self.word_end(r);
                if e2 > r {
                    let w: String = self.c[r..e2].iter().collect();
                    if month_index(&w, lang).is_some() {
                        let spoken = match lang {
                            Lang::Fr if day == 1 => "premier".to_string(),
                            Lang::Fr => self.cardinal(day),
                            Lang::En => en_ordinal(day),
                        };
                        return (spoken, end);
                    }
                }
            }
        }
        // English dates after a month name: "March 12" -> twelfth, "March 2026" -> twenty twenty-six.
        if lang == Lang::En && self.month_stage > 0 && num.frac.is_none() {
            if let Some(v) = num.value() {
                if self.month_stage == 1 && (1..=31).contains(&v) && num.int.len() <= 2 {
                    self.month_stage = 2;
                    return (en_ordinal(v), end);
                }
                if num.int.len() == 4 && (1100..=2099).contains(&v) {
                    self.month_stage = 0;
                    return (en_year(v), end);
                }
            }
        }
        // French: "1 fois", "21 gélules" take the feminine.
        let feminine = lang == Lang::Fr && self.next_word_is_feminine(end);
        (self.quantity_words(num, feminine), end)
    }

    fn next_word_is_feminine(&self, end: usize) -> bool {
        let r = self.skip_one_blank(end);
        let e = self.word_end(r);
        if e == r {
            return false;
        }
        let w: String = self.c[r..e].iter().collect::<String>().to_lowercase();
        FR_FEMININE_NOUNS.contains(&w.as_str())
    }

    fn unit_at(&self, num: &Num, q: usize) -> Option<(String, usize)> {
        let e = self.word_end(q);
        if e == q {
            return None;
        }
        let sym: String = self.c[q..e].iter().collect();
        let unit = find_unit(&sym)?;
        // "5 m'a dit", "3 s'est" are not units; "5 g" before a letter-less boundary is.
        if !self.boundary_after(e) && self.at(e) != Some('/') {
            return None;
        }
        let plural = self.plural(num);
        let mut name = unit_name(unit, plural, self.lang).to_string();
        let mut end = e;
        // Rates: "mg/kg", "mg/jour".
        if self.at(e) == Some('/') {
            let e2 = self.word_end(e + 1);
            if e2 > e + 1 && self.boundary_after(e2) {
                let sym2: String = self.c[e + 1..e2].iter().collect();
                let per = if self.lang == Lang::Fr { "par" } else { "per" };
                if let Some(u2) = find_unit(&sym2) {
                    name = format!("{name} {per} {}", unit_name(u2, false, self.lang));
                    end = e2;
                } else if let Some((_, fr, en)) = PER_WORDS.iter().find(|(k, _, _)| *k == sym2.to_lowercase()) {
                    name = format!("{name} {per} {}", if self.lang == Lang::Fr { fr } else { en });
                    end = e2;
                }
            }
        }
        let qty = self.quantity_words(num, false);
        // French "deux millions de grammes".
        let glue = if self.lang == Lang::Fr && num.frac.is_none() && num.value().map(|v| v >= 1_000_000 && v % 1_000_000 == 0).unwrap_or(false) {
            if name.starts_with(|c: char| "aeiouyéèh".contains(c)) { " d'" } else { " de " }
        } else {
            " "
        };
        Some((format!("{qty}{glue}{name}"), end))
    }

    // ---- currency -------------------------------------------------------------------------------

    fn currency_at(&self, q: usize) -> Option<(&'static str, usize)> {
        match self.at(q)? {
            '€' => return Some(("€", q + 1)),
            '$' => return Some(("$", q + 1)),
            '£' => return Some(("£", q + 1)),
            _ => {}
        }
        for code in ["EUR", "USD", "GBP", "CHF"] {
            if self.matches_at(q, code) && self.boundary_after(q + 3) {
                return Some((code, q + 3));
            }
        }
        None
    }

    fn currency_prefix(&mut self, p: usize) -> bool {
        let sym = self.c[p].to_string();
        let q = self.skip_one_blank(p + 1);
        if self.digits_len(q) == 0 {
            return false;
        }
        let Some(num) = self.parse_number(q) else { return false };
        match self.currency_words(&num, &sym) {
            Some(s) => {
                self.speak(&s);
                self.i = num.end;
                true
            }
            None => false,
        }
    }

    fn currency_words(&self, num: &Num, sym: &str) -> Option<String> {
        let cur = find_currency(sym)?;
        let value = num.value()?;
        let cents: u64 = match &num.frac {
            None => 0,
            Some(f) if f.len() <= 2 => format!("{f:0<2}").parse().ok()?,
            Some(_) => return None,
        };
        let fr = self.lang == Lang::Fr;
        let (one, many, c_word) = if fr { (cur.1, cur.2, cur.5) } else { (cur.3, cur.4, cur.6) };
        let big_plural = if fr { value >= 2 } else { value != 1 };
        let mut s = String::new();
        if value > 0 || cents == 0 {
            let name = if big_plural { many } else { one };
            let glue = if fr && value >= 1_000_000 && value % 1_000_000 == 0 {
                if name.starts_with(|c: char| "aeiouyéèh".contains(c)) { "d'" } else { "de " }
            } else {
                ""
            };
            s = if glue.is_empty() { format!("{} {name}", self.cardinal(value)) } else { format!("{} {glue}{name}", self.cardinal(value)) };
        }
        if cents > 0 {
            if !s.is_empty() {
                s.push(' ');
                if !fr {
                    s.push_str("and ");
                }
            }
            s.push_str(&self.cardinal(cents));
            if value == 0 || !fr {
                let plural_c = cents != 1;
                let w = if fr {
                    if plural_c { format!("{c_word}s") } else { c_word.to_string() }
                } else if cur.6 == "penny" || cur.6 == "pence" {
                    if plural_c { "pence".to_string() } else { "penny".to_string() }
                } else if plural_c {
                    format!("{c_word}s")
                } else {
                    c_word.to_string()
                };
                s.push(' ');
                s.push_str(&w);
            }
        }
        Some(s)
    }

    // ---- ordinals -------------------------------------------------------------------------------

    fn try_ordinal(&self, p: usize) -> Option<(String, usize)> {
        let l = self.digits_len(p);
        if l == 0 || l > 9 {
            return None;
        }
        let n: u64 = self.digits(p, l).parse().ok()?;
        let e = p + l;
        let suffix_end = {
            let mut q = e;
            while q < self.n() && self.c[q].is_alphabetic() {
                q += 1;
            }
            q
        };
        if suffix_end == e || !self.boundary_after(suffix_end) {
            return None;
        }
        let suffix: String = self.c[e..suffix_end].iter().collect::<String>().to_lowercase();
        match self.lang {
            Lang::Fr => {
                let (ok, fem) = match suffix.as_str() {
                    "er" => (n >= 1, false),
                    "re" | "ère" => (n == 1, true),
                    "e" | "ème" | "eme" => (n >= 2, false),
                    "nd" => (n == 2, false),
                    "nde" => (n == 2, true),
                    "es" | "èmes" => (n >= 2, false),
                    _ => (false, false),
                };
                if !ok {
                    return None;
                }
                let mut s = if n == 2 && (suffix == "nd" || suffix == "nde") {
                    if fem { "seconde".to_string() } else { "second".to_string() }
                } else {
                    fr_ordinal(n, fem)
                };
                if suffix == "es" || suffix == "èmes" {
                    s.push('s');
                }
                Some((s, suffix_end))
            }
            Lang::En => matches!(suffix.as_str(), "st" | "nd" | "rd" | "th").then(|| (en_ordinal(n), suffix_end)),
        }
    }

    // ---- fractions ------------------------------------------------------------------------------

    fn try_fraction(&self, p: usize) -> Option<(String, usize)> {
        let a = self.digits_len(p);
        if a == 0 || a > 3 || self.at(p + a) != Some('/') {
            return None;
        }
        let b = self.digits_len(p + a + 1);
        if b == 0 || b > 3 || !self.boundary_after(p + a + 1 + b) || self.at(p + a + 1 + b) == Some('/') {
            return None;
        }
        let x: u64 = self.digits(p, a).parse().ok()?;
        let y: u64 = self.digits(p + a + 1, b).parse().ok()?;
        let end = p + a + 1 + b;
        let fr = self.lang == Lang::Fr;
        // French "le 12/03" after a date word is a day and a month, not a fraction.
        const DATE_WORDS: &[&str] = &["le", "du", "au", "dès", "depuis", "jusqu'au", "avant", "après", "ce"];
        if fr && (1..=31).contains(&x) && (1..=12).contains(&y) && DATE_WORDS.contains(&self.last_word.as_str()) {
            let day = if x == 1 { "premier".to_string() } else { fr_cardinal(x) };
            return Some((format!("{day} {}", self.month_name(y as usize)), end));
        }
        // A rate or a ratio followed by a unit: "120/80 mmHg".
        if self.is_generic_ratio(x, y) {
            let q = self.skip_one_blank(end);
            let denominator = Num { int: y.to_string(), frac: None, end };
            if let Some((with_unit, e)) = self.unit_at(&denominator, q) {
                let over = if fr { "sur" } else { "over" };
                let numerator = self.cardinal(x);
                return Some((format!("{numerator} {over} {with_unit}"), e));
            }
        }
        let spoken = match (x, y, fr) {
            (1, 2, true) => "un demi".to_string(),
            (1, 3, true) => "un tiers".to_string(),
            (2, 3, true) => "deux tiers".to_string(),
            (1, 4, true) => "un quart".to_string(),
            (3, 4, true) => "trois quarts".to_string(),
            (1, 2, false) => "one half".to_string(),
            (1, 3, false) => "one third".to_string(),
            (2, 3, false) => "two thirds".to_string(),
            (1, 4, false) => "one quarter".to_string(),
            (3, 4, false) => "three quarters".to_string(),
            (x, y, true) => format!("{} sur {}", fr_cardinal(x), fr_cardinal(y)),
            (x, y, false) => format!("{} over {}", en_cardinal(x), en_cardinal(y)),
        };
        Some((spoken, end))
    }

    fn is_generic_ratio(&self, x: u64, y: u64) -> bool {
        !matches!((x, y), (1, 2) | (1, 3) | (2, 3) | (1, 4) | (3, 4))
    }

    // ---- dates ----------------------------------------------------------------------------------

    fn month_name(&self, m: usize) -> String {
        match self.lang {
            Lang::Fr => FR_MONTHS[m - 1].to_string(),
            Lang::En => {
                let s = EN_MONTHS[m - 1];
                let mut ch = s.chars();
                ch.next().map(|f| f.to_uppercase().collect::<String>() + ch.as_str()).unwrap_or_default()
            }
        }
    }

    /// 12/03/2026, 12.03.2026, 12-03-2026 and 2026-03-12.
    fn try_numeric_date(&self, p: usize) -> Option<(String, usize)> {
        let l1 = self.digits_len(p);
        if !(l1 == 1 || l1 == 2 || l1 == 4) {
            return None;
        }
        let sep = self.at(p + l1)?;
        if !matches!(sep, '/' | '.' | '-') {
            return None;
        }
        let l2 = self.digits_len(p + l1 + 1);
        if !(l2 == 1 || l2 == 2) || self.at(p + l1 + 1 + l2) != Some(sep) {
            return None;
        }
        let l3 = self.digits_len(p + l1 + l2 + 2);
        let end = p + l1 + l2 + 2 + l3;
        let v1: u64 = self.digits(p, l1).parse().ok()?;
        let v2: u64 = self.digits(p + l1 + 1, l2).parse().ok()?;
        let (day, month, year) = if l1 == 4 && (l3 == 1 || l3 == 2) && sep == '-' {
            (self.digits(p + l1 + l2 + 2, l3).parse::<u64>().ok()?, v2, v1)
        } else if l3 == 4 && l1 <= 2 {
            let y: u64 = self.digits(p + l1 + l2 + 2, 4).parse().ok()?;
            match self.lang {
                Lang::Fr => (v1, v2, y),
                Lang::En => {
                    if v1 > 12 { (v1, v2, y) } else { (v2, v1, y) }
                }
            }
        } else {
            return None;
        };
        if !(1..=31).contains(&day) || !(1..=12).contains(&month) {
            return None;
        }
        // Not the start of a longer group of numbers.
        if matches!(self.at(end), Some(c) if c.is_ascii_digit() || (matches!(c, '/' | '.' | '-') && self.digits_len(end + 1) > 0)) {
            return None;
        }
        let spoken = match self.lang {
            Lang::Fr => {
                let d = if day == 1 { "premier".to_string() } else { fr_cardinal(day) };
                format!("{d} {} {}", self.month_name(month as usize), fr_cardinal(year))
            }
            Lang::En => format!("{} {}, {}", self.month_name(month as usize), en_ordinal(day), en_year(year)),
        };
        Some((spoken, end))
    }

    // ---- times ----------------------------------------------------------------------------------

    /// "am", "p.m." ... after a time; a final dot that also ends the sentence is kept.
    fn ampm_at(&self, q: usize) -> Option<(String, usize)> {
        let r = self.skip_one_blank(q);
        for (lit, word) in [("a.m.", "a m"), ("p.m.", "p m")] {
            if self.matches_ci_at(r, lit) {
                let dot = if self.rest_is_blank(r + 4) { "." } else { "" };
                return Some((format!("{word}{dot}"), r + 4));
            }
        }
        for (lit, word) in [("am", "a m"), ("pm", "p m")] {
            if self.matches_ci_at(r, lit) && self.boundary_after(r + 2) {
                return Some((word.to_string(), r + 2));
            }
        }
        None
    }

    fn time_words(&self, h: u64, m: u64, ampm: Option<&str>) -> String {
        match self.lang {
            Lang::Fr => {
                let mut s = if h == 1 { "une heure".to_string() } else { format!("{} heures", fr_cardinal(h)) };
                if h == 0 {
                    s = "zéro heure".to_string();
                }
                if m > 0 {
                    s.push(' ');
                    s.push_str(&fr_cardinal(m));
                }
                s
            }
            Lang::En => {
                let mut s = en_cardinal(h);
                if m == 0 {
                    if ampm.is_none() {
                        s.push_str(" o'clock");
                    }
                } else if m < 10 {
                    s.push_str(&format!(" oh {}", en_cardinal(m)));
                } else {
                    s.push_str(&format!(" {}", en_cardinal(m)));
                }
                if let Some(a) = ampm {
                    s.push(' ');
                    s.push_str(a);
                }
                s
            }
        }
    }

    fn follows_with_unit_or_money(&self, q: usize) -> bool {
        let r = self.skip_one_blank(q);
        if matches!(self.at(r), Some('%') | Some('€') | Some('$') | Some('£') | Some('°')) {
            return true;
        }
        let e = self.word_end(r);
        if e == r {
            return false;
        }
        let w: String = self.c[r..e].iter().collect();
        find_unit(&w).is_some() || find_currency(&w).is_some() || CURRENCY_WORDS.contains(&w.to_lowercase().as_str())
    }

    fn try_time(&self, p: usize) -> Option<(String, usize)> {
        let lh = self.digits_len(p);
        if lh == 0 || lh > 2 {
            return None;
        }
        let h: u64 = self.digits(p, lh).parse().ok()?;
        let after_h = p + lh;
        match self.lang {
            Lang::Fr => {
                // "14 h 30", "14h30", "9h00", "14h"
                let q = self.skip_one_blank(after_h);
                if matches!(self.at(q), Some('h') | Some('H')) && h <= 24 {
                    let r = q + 1;
                    let r2 = self.skip_one_blank(r);
                    let lm = self.digits_len(r2);
                    if lm == 2 && self.digits_len(r2) == 2 && self.boundary_after_digits(r2 + 2) {
                        let m: u64 = self.digits(r2, 2).parse().ok()?;
                        if m <= 59 {
                            return Some((self.time_words(h, m, None), r2 + 2));
                        }
                    }
                    if !self.at(r).map(|c| c.is_alphanumeric()).unwrap_or(false) {
                        return Some((self.time_words(h, 0, None), r));
                    }
                    return None;
                }
                // "14:30"
                if self.at(after_h) == Some(':') && self.digits_len(after_h + 1) == 2 && h <= 23 && self.at(after_h + 3) != Some(':') {
                    let m: u64 = self.digits(after_h + 1, 2).parse().ok()?;
                    if m <= 59 && self.boundary_after_digits(after_h + 3) {
                        return Some((self.time_words(h, m, None), after_h + 3));
                    }
                }
                // "10.30" only after a time word and not before a unit or an amount.
                if self.at(after_h) == Some('.') && self.digits_len(after_h + 1) == 2 && h <= 23 && self.boundary_after_digits(after_h + 3) {
                    let m: u64 = self.digits(after_h + 1, 2).parse().ok()?;
                    const PREP: &[&str] = &["à", "vers", "dès", "de", "entre", "et", "jusqu'à", "jusqu’à", "avant", "après", "depuis"];
                    if m <= 59 && PREP.contains(&self.last_word.as_str()) && !self.follows_with_unit_or_money(after_h + 3) {
                        return Some((self.time_words(h, m, None), after_h + 3));
                    }
                }
                None
            }
            Lang::En => {
                // "9 am", "9:30 pm", "14:30", "10.30 am", "at 10.30"
                if let Some((ap, e)) = self.ampm_at(after_h) {
                    if (1..=12).contains(&h) {
                        return Some((self.time_words(h, 0, Some(&ap)), e));
                    }
                }
                let sep = self.at(after_h)?;
                if (sep == ':' || sep == '.') && self.digits_len(after_h + 1) == 2 && self.boundary_after_digits(after_h + 3) && h <= 23 {
                    let m: u64 = self.digits(after_h + 1, 2).parse().ok()?;
                    if m > 59 {
                        return None;
                    }
                    if let Some((ap, e)) = self.ampm_at(after_h + 3) {
                        return Some((self.time_words(h, m, Some(&ap)), e));
                    }
                    const PREP: &[&str] = &["at", "by", "from", "to", "until", "till", "before", "after", "around", "since", "between", "and"];
                    if sep == ':' || (PREP.contains(&self.last_word.as_str()) && !self.follows_with_unit_or_money(after_h + 3)) {
                        return Some((self.time_words(h, m, None), after_h + 3));
                    }
                }
                None
            }
        }
    }

    /// True when the digits at `p` are not followed by another digit or a decimal/group separator + digit.
    fn boundary_after_digits(&self, p: usize) -> bool {
        match self.at(p) {
            None => true,
            Some(c) if c.is_ascii_digit() => false,
            Some(c) if c.is_alphabetic() => false,
            Some(',') | Some('.') | Some(':') => self.digits_len(p + 1) == 0,
            _ => true,
        }
    }

    // ---- phone numbers and identifiers -----------------------------------------------------------

    fn try_phone(&self, p: usize) -> Option<(String, usize)> {
        let plus = self.at(p) == Some('+');
        let mut q = if plus { p + 1 } else { p };
        let mut groups: Vec<(usize, usize)> = Vec::new(); // (start, len)
        loop {
            let l = self.digits_len(q);
            if l == 0 {
                break;
            }
            groups.push((q, l));
            q += l;
            match self.at(q) {
                Some(s) if (is_blank(s) || s == '.' || s == '-') && self.digits_len(q + 1) > 0 => q += 1,
                _ => break,
            }
        }
        if groups.is_empty() {
            return None;
        }
        if self.at(q).map(|c| c.is_alphabetic()).unwrap_or(false) {
            return None;
        }
        let total: usize = groups.iter().map(|g| g.1).sum();
        let first_zero = self.at(groups[0].0) == Some('0');
        let n_groups = groups.len();
        let rest_pairs = n_groups >= 3 && groups[1..].iter().all(|g| g.1 == 2) && groups[0].1 <= 3;
        let all_threes = n_groups >= 2 && groups.iter().skip(1).all(|g| g.1 == 3) && groups[0].1 <= 3 && !first_zero;
        // Several groups and at least nine digits that are not plain thousands groups: an identifier.
        let grouped_id = n_groups >= 3 && total >= 9 && !all_threes;
        let class_phone = (plus && total >= 7) || (first_zero && total >= 9) || (rest_pairs && total >= 8 && !all_threes);
        let class_id = !class_phone && (grouped_id || (n_groups == 1 && total >= 8));
        if !class_phone && !class_id {
            return None;
        }
        let mut parts: Vec<String> = Vec::new();
        for (s, l) in groups.iter() {
            let d = self.digits(*s, *l);
            let spoken = if class_phone && self.lang == Lang::Fr && n_groups == 1 && total == 10 {
                // "0612345678" -> pairs
                d.as_bytes().chunks(2).map(|c| self.pair_words(std::str::from_utf8(c).unwrap_or("0"))).collect::<Vec<_>>().join(", ")
            } else if class_phone && *l == 2 {
                self.pair_words(&d)
            } else {
                digits_one_by_one(&d, self.lang)
            };
            parts.push(spoken);
        }
        let mut spoken = parts.join(", ");
        if plus {
            spoken = format!("plus {spoken}");
        }
        Some((spoken, q))
    }

    /// Two digits read as a number; "06" -> "zéro six".
    fn pair_words(&self, d: &str) -> String {
        if d.starts_with('0') {
            digits_one_by_one(d, self.lang)
        } else {
            self.cardinal(d.parse().unwrap_or(0))
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fr(s: &str) -> String {
        normalise(s, Lang::Fr)
    }
    fn en(s: &str) -> String {
        normalise(s, Lang::En)
    }

    #[test]
    fn french_cardinals() {
        let cases: &[(u64, &str)] = &[
            (0, "zéro"), (1, "un"), (16, "seize"), (17, "dix-sept"), (21, "vingt et un"), (22, "vingt-deux"),
            (31, "trente et un"), (61, "soixante et un"), (70, "soixante-dix"), (71, "soixante et onze"),
            (72, "soixante-douze"), (77, "soixante-dix-sept"), (80, "quatre-vingts"), (81, "quatre-vingt-un"),
            (90, "quatre-vingt-dix"), (91, "quatre-vingt-onze"), (99, "quatre-vingt-dix-neuf"), (100, "cent"),
            (101, "cent un"), (180, "cent quatre-vingts"), (200, "deux cents"), (201, "deux cent un"),
            (280, "deux cent quatre-vingts"), (500, "cinq cents"), (1000, "mille"), (1001, "mille un"),
            (2000, "deux mille"), (2045, "deux mille quarante-cinq"), (2026, "deux mille vingt-six"),
            (80_000, "quatre-vingt mille"), (200_000, "deux cent mille"), (1_000_000, "un million"),
            (2_500_000, "deux millions cinq cent mille"), (1_998, "mille neuf cent quatre-vingt-dix-huit"),
        ];
        for (n, want) in cases {
            assert_eq!(fr_cardinal(*n), *want, "{n}");
        }
    }

    #[test]
    fn english_cardinals_and_years() {
        assert_eq!(en_cardinal(0), "zero");
        assert_eq!(en_cardinal(21), "twenty-one");
        assert_eq!(en_cardinal(100), "one hundred");
        assert_eq!(en_cardinal(1234), "one thousand two hundred thirty-four");
        assert_eq!(en_cardinal(2_000_005), "two million five");
        assert_eq!(en_year(2026), "twenty twenty-six");
        assert_eq!(en_year(1998), "nineteen ninety-eight");
        assert_eq!(en_year(1905), "nineteen oh five");
        assert_eq!(en_year(2005), "two thousand five");
        assert_eq!(en_year(2000), "two thousand");
        assert_eq!(en_year(2010), "twenty ten");
    }

    #[test]
    fn ordinals() {
        assert_eq!(fr("le 1er mars"), "le premier mars");
        assert_eq!(fr("la 1re fois"), "la première fois");
        assert_eq!(fr("le 2e étage, la 3ème porte, le 5e, le 9e"), "le deuxième étage, la troisième porte, le cinquième, le neuvième");
        assert_eq!(fr("le 21e, le 12e, le 80e, le 100e"), "le vingt et unième, le douzième, le quatre-vingtième, le centième");
        assert_eq!(en("the 1st, 2nd, 3rd and 12th"), "the first, second, third and twelfth");
        assert_eq!(en("the 21st and the 100th"), "the twenty-first and the one hundredth");
    }

    #[test]
    fn decimals_thousands_and_negatives() {
        assert_eq!(fr("12,5"), "douze virgule cinq");
        assert_eq!(fr("0,25"), "zéro virgule vingt-cinq");
        assert_eq!(fr("0,05"), "zéro virgule zéro cinq");
        assert_eq!(fr("3.5"), "trois virgule cinq");
        assert_eq!(fr("1 000"), "mille");
        assert_eq!(fr("2 500 patients"), "deux mille cinq cents patients");
        assert_eq!(fr("1.250"), "mille deux cent cinquante");
        assert_eq!(en("12.5"), "twelve point five");
        assert_eq!(en("0.25"), "zero point two five");
        assert_eq!(en("1,250 cases"), "one thousand two hundred fifty cases");
        assert_eq!(fr("-5"), "moins cinq");
        assert_eq!(fr("de 5-10 jours"), "de cinq à dix jours");
        assert_eq!(en("5-10 days"), "five to ten days");
        assert_eq!(fr("007"), "zéro zéro sept");
    }

    #[test]
    fn units_follow_the_plural_rule_of_each_language() {
        assert_eq!(fr("500 mg trois fois par jour"), "cinq cents milligrammes trois fois par jour");
        assert_eq!(fr("1 mg"), "un milligramme");
        assert_eq!(fr("1,5 g"), "un virgule cinq gramme");
        assert_eq!(fr("2,5 g"), "deux virgule cinq grammes");
        assert_eq!(fr("10 ml"), "dix millilitres");
        assert_eq!(fr("75 kg"), "soixante-quinze kilogrammes");
        assert_eq!(fr("37,5 °C"), "trente-sept virgule cinq degrés Celsius");
        assert_eq!(fr("-5 °C"), "moins cinq degrés Celsius");
        assert_eq!(fr("120 mmHg"), "cent vingt millimètres de mercure");
        assert_eq!(fr("72 bpm"), "soixante-douze battements par minute");
        assert_eq!(fr("5 mg/kg"), "cinq milligrammes par kilogramme");
        assert_eq!(fr("10 mg/jour"), "dix milligrammes par jour");
        assert_eq!(fr("2 000 000 g"), "deux millions de grammes");
        assert_eq!(en("500 mg"), "five hundred milligrams");
        assert_eq!(en("1 mg"), "one milligram");
        assert_eq!(en("1.5 g"), "one point five grams");
        assert_eq!(en("98.6 °F"), "ninety-eight point six degrees Fahrenheit");
        // Not units.
        assert_eq!(fr("5 m'a dit"), "cinq m'a dit");
        assert_eq!(fr("3 s'est"), "trois s'est");
    }

    #[test]
    fn percent_and_currency() {
        assert_eq!(fr("50 %"), "cinquante pour cent");
        assert_eq!(fr("12,5%"), "douze virgule cinq pour cent");
        assert_eq!(en("50%"), "fifty percent");
        assert_eq!(fr("50 €"), "cinquante euros");
        assert_eq!(fr("1 €"), "un euro");
        assert_eq!(fr("12,50 €"), "douze euros cinquante");
        assert_eq!(fr("0,50 €"), "cinquante centimes");
        assert_eq!(fr("€50"), "cinquante euros");
        assert_eq!(fr("100 CHF"), "cent francs");
        assert_eq!(fr("1 000 000 €"), "un million d'euros");
        assert_eq!(en("$12.50"), "twelve dollars and fifty cents");
        assert_eq!(en("$1"), "one dollar");
        assert_eq!(en("£5"), "five pounds");
    }

    #[test]
    fn french_times() {
        assert_eq!(fr("à 14 h 30"), "à quatorze heures trente");
        assert_eq!(fr("à 14h30"), "à quatorze heures trente");
        assert_eq!(fr("à 9h00"), "à neuf heures");
        assert_eq!(fr("à 9h05"), "à neuf heures cinq");
        assert_eq!(fr("à 14h"), "à quatorze heures");
        assert_eq!(fr("à 1 h"), "à une heure");
        assert_eq!(fr("à 14:30"), "à quatorze heures trente");
        assert_eq!(fr("rendez-vous à 10.30"), "rendez-vous à dix heures trente");
        assert_eq!(fr("entre 10.30 et 11.30"), "entre dix heures trente et onze heures trente");
        // A dot after no time word, or before an amount, is a decimal.
        assert_eq!(fr("version 2.15"), "version deux virgule quinze");
        assert_eq!(fr("de 12.50 euros"), "de douze virgule cinquante euros");
        assert_eq!(fr("5 heures"), "cinq heures");
    }

    #[test]
    fn english_times() {
        assert_eq!(en("at 9 am"), "at nine a m");
        assert_eq!(en("at 9:30 pm"), "at nine thirty p m");
        assert_eq!(en("at 14:30"), "at fourteen thirty");
        assert_eq!(en("at 9:00"), "at nine o'clock");
        assert_eq!(en("at 9:05"), "at nine oh five");
        assert_eq!(en("at 10.30"), "at ten thirty");
        assert_eq!(en("version 2.15"), "version two point one five");
        assert_eq!(en("5 p.m."), "five p m.");
        assert_eq!(en("the meeting is at 5 p.m."), "the meeting is at five p m.");
    }

    #[test]
    fn dates() {
        assert_eq!(fr("le 12 mars 2026"), "le douze mars deux mille vingt-six");
        assert_eq!(fr("le 1 mars 2026"), "le premier mars deux mille vingt-six");
        assert_eq!(fr("le 12/03/2026"), "le douze mars deux mille vingt-six");
        assert_eq!(fr("le 1/3/2026"), "le premier mars deux mille vingt-six");
        assert_eq!(fr("le 12.03.2026"), "le douze mars deux mille vingt-six");
        assert_eq!(fr("le 2026-03-12"), "le douze mars deux mille vingt-six");
        assert_eq!(fr("le 12 janv. 2026"), "le douze janvier deux mille vingt-six");
        assert_eq!(en("March 12th, 2026"), "March twelfth, twenty twenty-six");
        assert_eq!(en("March 12, 2026"), "March twelfth, twenty twenty-six");
        assert_eq!(en("12 March 2026"), "twelfth March twenty twenty-six");
        assert_eq!(fr("le 12/03"), "le douze mars");
        assert_eq!(en("on 03/12/2026"), "on March twelfth, twenty twenty-six");
        assert_eq!(en("on 25/12/2026"), "on December twenty-fifth, twenty twenty-six");
        assert_eq!(en("March 2026"), "March twenty twenty-six");
    }

    #[test]
    fn fractions_and_multipliers() {
        assert_eq!(fr("1/2 comprimé"), "un demi comprimé");
        assert_eq!(fr("3/4 de litre"), "trois quarts de litre");
        assert_eq!(fr("note 12/20"), "note douze sur vingt");
        assert_eq!(en("1/2 tablet"), "one half tablet");
        assert_eq!(fr("3 x par jour"), "trois fois par jour");
        assert_eq!(fr("2x/jour"), "deux fois par jour");
        assert_eq!(fr("3 x 500 mg"), "trois fois cinq cents milligrammes");
        assert_eq!(en("3x daily"), "three times daily");
    }

    #[test]
    fn feminine_one_before_common_feminine_nouns() {
        assert_eq!(fr("1 fois par jour"), "une fois par jour");
        assert_eq!(fr("21 gélules"), "vingt et une gélules");
        assert_eq!(fr("1 comprimé"), "un comprimé");
        assert_eq!(fr("2 fois"), "deux fois");
    }

    #[test]
    fn abbreviations() {
        assert_eq!(fr("M. Martin et Mme Roux voient le Dr Dupont"), "monsieur Martin et madame Roux voient le docteur Dupont");
        assert_eq!(fr("le Pr. Leroy à St. Gall"), "le professeur Leroy à saint Gall");
        assert_eq!(fr("des pommes, etc."), "des pommes, et cetera.");
        assert_eq!(fr("des pommes, etc. et des poires"), "des pommes, et cetera et des poires");
        assert_eq!(fr("facture n° 2045"), "facture numéro deux mille quarante-cinq");
        assert_eq!(fr("facture N°2045"), "facture numéro deux mille quarante-cinq");
        assert_eq!(fr("voir p. 12"), "voir page douze");
        assert_eq!(fr("env. 5 jours"), "environ cinq jours");
        assert_eq!(fr("tél. 021"), "téléphone zéro deux un");
        assert_eq!(fr("c.-à-d. demain"), "c'est-à-dire demain");
        assert_eq!(en("Mr. Smith and Dr. Jones, e.g. today"), "mister Smith and doctor Jones, for example today");
        assert_eq!(en("Dr Jones"), "doctor Jones");
        // Look-alikes are untouched.
        assert_eq!(fr("la drogue et le prix"), "la drogue et le prix");
        assert_eq!(fr("Pris le matin"), "Pris le matin");
    }

    #[test]
    fn phone_numbers_and_identifiers() {
        assert_eq!(fr("06 12 34 56 78"), "zéro six, douze, trente-quatre, cinquante-six, soixante-dix-huit");
        assert_eq!(fr("0612345678"), "zéro six, douze, trente-quatre, cinquante-six, soixante-dix-huit");
        assert_eq!(fr("079 123 45 67"), "zéro sept neuf, un deux trois, quarante-cinq, soixante-sept");
        assert_eq!(fr("+41 79 123 45 67"), "plus quarante et un, soixante-dix-neuf, un deux trois, quarante-cinq, soixante-sept");
        assert_eq!(fr("756.1234.5678.97"), "sept cinq six, un deux trois quatre, cinq six sept huit, neuf sept");
        assert_eq!(en("call 555 123 4567 now"), "call five five five, one two three, four five six seven now");
        // A plain amount with thousands groups is a number, not an identifier.
        assert_eq!(fr("1 000 000"), "un million");
        assert_eq!(fr("123 456"), "cent vingt-trois mille quatre cent cinquante-six");
    }

    #[test]
    fn owner_test_sentences() {
        assert_eq!(fr("Prendre 500 mg trois fois par jour."), "Prendre cinq cents milligrammes trois fois par jour.");
        assert_eq!(fr("La facture numéro 2045 est payable sous 30 jours."), "La facture numéro deux mille quarante-cinq est payable sous trente jours.");
        assert_eq!(fr("Rendez-vous le 12 mars à 9h00."), "Rendez-vous le douze mars à neuf heures.");
        assert_eq!(fr("Le patient pèse 72,5 kg et mesure 1,80 m."), "Le patient pèse soixante-douze virgule cinq kilogrammes et mesure un virgule quatre-vingts mètre.");
        assert_eq!(fr("La tension est de 120/80 mmHg."), "La tension est de cent vingt sur quatre-vingts millimètres de mercure.");
    }

    #[test]
    fn spaces_and_unknown_text_are_kept_sensible() {
        assert_eq!(fr("Bonjour tout le monde"), "Bonjour tout le monde");
        assert_eq!(fr(""), "");
        assert_eq!(fr("  vitamine B12  "), "vitamine B douze");
        assert_eq!(normalise_for("Il a 3 ans", "de"), "Il a 3 ans");
        assert_eq!(normalise_for("Il a 3 ans", "fr"), "Il a trois ans");
        // Punctuation next to numbers stays.
        assert_eq!(fr("(3)"), "(trois)");
        assert_eq!(fr("3, 4 et 5."), "trois, quatre et cinq.");
    }
}
