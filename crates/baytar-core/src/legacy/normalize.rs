//! Pure, lossless normalization of legacy SQLite scalar values.
//!
//! Every non-NULL input retains its original text, including surrounding whitespace.
//! SQLite numbers use Rust's round-trippable display representation. Missing or invalid
//! dates and phones are unparsed; missing species and sex carry no warning.

pub use chrono::NaiveDate as Date;

/// The SQLite storage classes used by legacy fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Raw<'a> {
    Null,
    Integer(i64),
    Real(f64),
    Text(&'a str),
}

impl Raw<'_> {
    fn original(self) -> Option<String> {
        match self {
            Self::Null => None,
            Self::Integer(value) => Some(value.to_string()),
            Self::Real(value) => Some(value.to_string()),
            Self::Text(value) => Some(value.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized<T> {
    pub value: Option<T>,
    pub raw: Option<String>,
    pub warning: Option<Warning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warning {
    Unparsed,
    /// Reserved for values that have more than one plausible interpretation.
    Ambiguous,
    Adjusted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Species {
    Cat,
    Dog,
    Bird,
    Rabbit,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

/// Parse documented legacy calendar dates without guessing two-digit years or timestamps.
pub fn date(raw: Raw<'_>) -> Normalized<Date> {
    let original = raw.original();
    let value = match raw {
        Raw::Text(text) => parse_date(text.trim()),
        Raw::Null | Raw::Integer(_) | Raw::Real(_) => None,
    };
    Normalized {
        value,
        raw: original,
        warning: value.is_none().then_some(Warning::Unparsed),
    }
}

fn parse_date(text: &str) -> Option<Date> {
    if let Some(date) = numeric_date(text) {
        return Some(date);
    }

    // Validate the entire timestamp, rather than accepting a valid date prefix
    // followed by arbitrary text. The prefix check also enforces a four-digit year.
    if let Some((prefix, _)) = text.split_once(['T', ' ']) {
        if prefix.split('-').next()?.len() == 4 && numeric_date(prefix).is_some() {
            for format in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f"] {
                if let Ok(datetime) = chrono::NaiveDateTime::parse_from_str(text, format) {
                    return Some(datetime.date());
                }
            }
            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(text) {
                return Some(datetime.date_naive());
            }
        }
    }

    let folded = fold(text);
    // A long date has at most four words. Bound token collection on malformed input.
    let words: Vec<_> = folded
        .split_whitespace()
        .map(|word| word.trim_end_matches(','))
        .take(5)
        .collect();
    match words.as_slice() {
        [day, month, year] if month_number(month).is_some() => long_date(day, month, year),
        [month, day, year] => long_date(day, month, year),
        [weekday, month, day, year] if is_weekday(weekday) => long_date(day, month, year),
        [day, month, year, weekday] if is_weekday(weekday) => long_date(day, month, year),
        _ => None,
    }
}

fn numeric_date(text: &str) -> Option<Date> {
    for separator in ['-', '.', '/'] {
        let mut parts = text.split(separator);
        let (Some(first), Some(second), Some(third), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let (year, month, day) = if separator == '-' && first.len() == 4 {
            (
                year_number(first)?,
                small_number(second)?,
                small_number(third)?,
            )
        } else {
            (
                year_number(third)?,
                small_number(second)?,
                small_number(first)?,
            )
        };
        return Date::from_ymd_opt(year, month, day);
    }
    None
}

fn long_date(day: &str, month: &str, year: &str) -> Option<Date> {
    Date::from_ymd_opt(year_number(year)?, month_number(month)?, small_number(day)?)
}

fn year_number(text: &str) -> Option<i32> {
    (text.len() == 4 && text.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

fn small_number(text: &str) -> Option<u32> {
    ((1..=2).contains(&text.len()) && text.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

fn month_number(month: &str) -> Option<u32> {
    match month {
        "ocak" | "january" => Some(1),
        "subat" | "february" => Some(2),
        "mart" | "march" => Some(3),
        "nisan" | "april" => Some(4),
        "mayis" | "may" => Some(5),
        "haziran" | "june" => Some(6),
        "temmuz" | "july" => Some(7),
        "agustos" | "august" => Some(8),
        "eylul" | "september" => Some(9),
        "ekim" | "october" => Some(10),
        "kasim" | "november" => Some(11),
        "aralik" | "december" => Some(12),
        _ => None,
    }
}

fn is_weekday(word: &str) -> bool {
    matches!(
        word,
        "pazartesi"
            | "sali"
            | "carsamba"
            | "persembe"
            | "cuma"
            | "cumartesi"
            | "pazar"
            | "monday"
            | "tuesday"
            | "wednesday"
            | "thursday"
            | "friday"
            | "saturday"
            | "sunday"
    )
}

/// Normalize Turkish mobile and area-code landline numbers to digits for display.
/// Removing display separators alone does not produce an adjustment warning.
pub fn phone(raw: Raw<'_>) -> Normalized<String> {
    let original = raw.original();
    let result = match raw {
        // A fractional REAL must not become a phone by stripping its decimal point.
        Raw::Real(number)
            if !number.is_finite() || number.fract() != 0.0 || number.is_sign_negative() =>
        {
            None
        }
        Raw::Integer(number) if number < 0 => None,
        _ => original.as_deref().and_then(parse_phone),
    };
    let (value, warning) = match result {
        Some((value, adjusted)) => (Some(value), adjusted.then_some(Warning::Adjusted)),
        None => (None, Some(Warning::Unparsed)),
    };
    Normalized {
        value,
        raw: original,
        warning,
    }
}

fn parse_phone(text: &str) -> Option<(String, bool)> {
    let digits: String = text
        .chars()
        .filter(|character| {
            !character.is_whitespace() && !matches!(character, '-' | '.' | '(' | ')')
        })
        .collect();
    let national = digits.strip_prefix("+90").or_else(|| {
        (digits.len() == 12)
            .then(|| digits.strip_prefix("90"))
            .flatten()
    });
    if let Some(national) = national {
        if national.len() == 10 && national.bytes().all(|byte| byte.is_ascii_digit()) {
            return Some((format!("0{national}"), true));
        }
        return None;
    }
    match digits.len() {
        10 if valid_national(&digits) => Some((format!("0{digits}"), true)),
        11 if digits.starts_with("05") && digits.bytes().all(|byte| byte.is_ascii_digit()) => {
            Some((digits, false))
        }
        _ => None,
    }
}

fn valid_national(text: &str) -> bool {
    text.starts_with(['2', '3', '4', '5']) && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Recognize legacy species labels, preserving free text as an adjusted Other.
pub fn species(raw: Raw<'_>) -> Normalized<Species> {
    let original = raw.original();
    let folded = fold(original.as_deref().unwrap_or_default().trim());
    let (value, warning) = match folded.as_str() {
        "" => (None, None),
        "kedi" | "cat" => (Some(Species::Cat), None),
        "kopek" | "dog" => (Some(Species::Dog), None),
        "kus" | "muhabbet kusu" => (Some(Species::Bird), None),
        "tavsan" => (Some(Species::Rabbit), None),
        "diger" => (Some(Species::Other), None),
        _ => (Some(Species::Other), Some(Warning::Adjusted)),
    };
    Normalized {
        value,
        raw: original,
        warning,
    }
}

/// Recognize legacy sex labels, preserving free text as an adjusted Unknown.
pub fn sex(raw: Raw<'_>) -> Normalized<Sex> {
    let original = raw.original();
    let folded = fold(original.as_deref().unwrap_or_default().trim());
    let (value, warning) = match folded.as_str() {
        "" => (Sex::Unknown, None),
        "erkek" | "e" | "male" => (Sex::Male, None),
        "disi" | "d" | "female" => (Sex::Female, None),
        _ => (Sex::Unknown, Some(Warning::Adjusted)),
    };
    Normalized {
        value: Some(value),
        raw: original,
        warning,
    }
}

/// Fold both Turkish I pairs and Turkish accents to their ASCII matching keys.
fn fold(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            'I' | 'İ' | 'ı' => 'i',
            'Ç' | 'ç' => 'c',
            'Ğ' | 'ğ' => 'g',
            'Ö' | 'ö' => 'o',
            'Ş' | 'ş' => 's',
            'Ü' | 'ü' => 'u',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    fn check_text<T: Debug + PartialEq>(
        actual: Normalized<T>,
        input: &str,
        value: Option<T>,
        warning: Option<Warning>,
    ) {
        assert_eq!(actual.value, value, "input: {input:?}");
        assert_eq!(actual.warning, warning, "input: {input:?}");
        assert_eq!(actual.raw.as_deref(), Some(input));
    }

    fn calendar(year: i32, month: u32, day: u32) -> Date {
        Date::from_ymd_opt(year, month, day).expect("valid synthetic test date")
    }

    #[test]
    fn numeric_dates_and_complete_iso_timestamps() {
        for input in [
            "07-01-2025",
            "7-1-2025",
            "07.01.2025",
            "07/01/2025",
            "2025-01-07",
            "2025-01-07 00:00:00",
            "2025-01-07T10:20:30",
            "2025-01-07T10:20:30.123456",
            "2025-01-07T23:20:30Z",
            "2025-01-07T00:20:30+03:00",
            " \t2025-01-07\r\n",
        ] {
            check_text(
                date(Raw::Text(input)),
                input,
                Some(calendar(2025, 1, 7)),
                None,
            );
        }
        check_text(
            date(Raw::Text("29-02-2024")),
            "29-02-2024",
            Some(calendar(2024, 2, 29)),
            None,
        );
        // The documented day-month order wins even when both numbers are <= 12.
        check_text(
            date(Raw::Text("04-05-2026")),
            "04-05-2026",
            Some(calendar(2026, 5, 4)),
            None,
        );
    }

    #[test]
    fn turkish_long_dates_in_every_month_and_case() {
        let months = [
            ["Ocak", "ocak", "OCAK", "Ocak"],
            ["Şubat", "şubat", "ŞUBAT", "Subat"],
            ["Mart", "mart", "MART", "Mart"],
            ["Nisan", "nisan", "NİSAN", "Nisan"],
            ["Mayıs", "mayıs", "MAYIS", "Mayis"],
            ["Haziran", "haziran", "HAZİRAN", "Haziran"],
            ["Temmuz", "temmuz", "TEMMUZ", "Temmuz"],
            ["Ağustos", "ağustos", "AĞUSTOS", "Agustos"],
            ["Eylül", "eylül", "EYLÜL", "Eylul"],
            ["Ekim", "ekim", "EKİM", "Ekim"],
            ["Kasım", "kasım", "KASIM", "Kasim"],
            ["Aralık", "aralık", "ARALIK", "Aralik"],
        ];
        for (index, variants) in months.iter().enumerate() {
            for month in variants {
                let input = format!("04 {month} 2026");
                check_text(
                    date(Raw::Text(&input)),
                    &input,
                    Some(calendar(2026, index as u32 + 1, 4)),
                    None,
                );
            }
        }
        for input in [
            "4 Kasım 2026 Çarşamba",
            "04 Kasım 2026",
            "4 kasım 2026 çarşamba",
            "4 KASIM 2026 ÇARŞAMBA",
            "4 KASİM 2026 CARSAMBA",
            " 4\tKasım   2026\nÇarşamba ",
        ] {
            check_text(
                date(Raw::Text(input)),
                input,
                Some(calendar(2026, 11, 4)),
                None,
            );
        }
    }

    #[test]
    fn recognizes_all_turkish_and_english_weekdays() {
        let weekdays = [
            ("Pazartesi", "Monday"),
            ("Salı", "Tuesday"),
            ("Çarşamba", "Wednesday"),
            ("Perşembe", "Thursday"),
            ("Cuma", "Friday"),
            ("Cumartesi", "Saturday"),
            ("Pazar", "Sunday"),
        ];
        for (index, (turkish, english)) in weekdays.iter().enumerate() {
            let day = index as u32 + 5;
            for input in [
                format!("{day} Ocak 2026 {turkish}"),
                format!("{english}, January {day}, 2026"),
            ] {
                check_text(
                    date(Raw::Text(&input)),
                    &input,
                    Some(calendar(2026, 1, day)),
                    None,
                );
            }
        }
    }

    #[test]
    fn english_long_dates_in_every_month() {
        for (index, month) in [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ]
        .iter()
        .enumerate()
        {
            for input in [format!("{month} 4, 2026"), format!("4 {month} 2026")] {
                check_text(
                    date(Raw::Text(&input)),
                    &input,
                    Some(calendar(2026, index as u32 + 1, 4)),
                    None,
                );
            }
        }
        let input = "Wednesday, November 4, 2026";
        check_text(
            date(Raw::Text(input)),
            input,
            Some(calendar(2026, 11, 4)),
            None,
        );
    }

    #[test]
    fn impossible_incomplete_and_malformed_dates_are_unparsed() {
        for input in [
            "",
            " \t\n",
            "🐈",
            "31-02-2026",
            "29-02-2026",
            "29-02-1900",
            "31.04.2026",
            "00/01/2026",
            "01/00/2026",
            "01/13/2026",
            "2026-02-31",
            "1-1-26",
            "01.01.26",
            "01/01/26",
            "26-01-01",
            "4 Kasım 26",
            "November 4, 26",
            "26-01-07T10:20:30",
            "2025-01-07T99:20:30",
            "2025-01-07 10:99:30",
            "2025-01-07T10:20:99",
            "2025-01-07Tgarbage",
            "2025-01-07 00:00:00 trailing text",
            "2025-01-07T10:20:30+99:00",
            "31 Şubat 2026",
            "4 Smarch 2026",
            "4 Kasım 2026 garbage",
            "garbage, November 4, 2026",
            "4 Kasım 2026 Çarşamba extra",
            "2025-01-07-extra",
            "7/1/2025/extra",
            "-001-01-01",
            "10000-01-01",
            "２０２６-０１-０１",
        ] {
            check_text(date(Raw::Text(input)), input, None, Some(Warning::Unparsed));
        }
        check_text(
            date(Raw::Text("29-02-2000")),
            "29-02-2000",
            Some(calendar(2000, 2, 29)),
            None,
        );
    }

    #[test]
    fn phone_display_separators_do_not_need_an_adjustment_warning() {
        for input in [
            "05321234567",
            "0 (532) 123-45-67",
            "0532.123.45.67",
            " \t0532\u{a0}123 45 67\n",
        ] {
            check_text(
                phone(Raw::Text(input)),
                input,
                Some("05321234567".to_owned()),
                None,
            );
        }
    }

    #[test]
    fn phones_restore_lost_zero_and_replace_country_code() {
        for (input, expected) in [
            ("5321234567", "05321234567"),
            ("532 123 45 67", "05321234567"),
            ("+905321234567", "05321234567"),
            ("905321234567", "05321234567"),
            ("+90 (532) 123-45-67", "05321234567"),
            ("90 532.123.45.67", "05321234567"),
            ("2121234567", "02121234567"),
            ("3121234567", "03121234567"),
            ("4121234567", "04121234567"),
            ("+90 (212) 123 45 67", "02121234567"),
            ("903121234567", "03121234567"),
            ("+904121234567", "04121234567"),
        ] {
            check_text(
                phone(Raw::Text(input)),
                input,
                Some(expected.to_owned()),
                Some(Warning::Adjusted),
            );
        }
    }

    #[test]
    fn country_code_requires_exactly_ten_ascii_digits() {
        for input in ["+901234567890", "901234567890"] {
            check_text(
                phone(Raw::Text(input)),
                input,
                Some("01234567890".to_owned()),
                Some(Warning::Adjusted),
            );
        }
    }

    #[test]
    fn malformed_phones_keep_raw_and_report_unparsed() {
        for input in [
            "",
            " \t\n",
            "🐕",
            "call me",
            "1234567890",
            "6121234567",
            "532123456",
            "53212345678",
            "005321234567",
            "+9005321234567",
            "+90532123456",
            "+9053212345678",
            "+90532123x567",
            "+44 532 123 45 67",
            "++905321234567",
            "0532/123/45/67",
            "05321234567 ext 1",
            "０５３２１２３４５６７",
        ] {
            check_text(
                phone(Raw::Text(input)),
                input,
                None,
                Some(Warning::Unparsed),
            );
        }
    }

    #[test]
    fn integer_and_integral_real_phones_are_supported() {
        for raw in [Raw::Integer(5_321_234_567), Raw::Real(5_321_234_567.0)] {
            assert_eq!(
                phone(raw),
                Normalized {
                    value: Some("05321234567".to_owned()),
                    raw: Some("5321234567".to_owned()),
                    warning: Some(Warning::Adjusted),
                }
            );
        }
        assert_eq!(
            phone(Raw::Integer(905_321_234_567)).value.as_deref(),
            Some("05321234567")
        );
        assert_eq!(
            phone(Raw::Integer(2_121_234_567)).value.as_deref(),
            Some("02121234567")
        );
    }

    #[test]
    fn numbers_cannot_be_misread_as_dates_or_fractional_phones() {
        for (raw, expected_raw) in [
            (Raw::Integer(i64::MIN), "-9223372036854775808"),
            (Raw::Integer(i64::MAX), "9223372036854775807"),
            (Raw::Integer(-5_321_234_567), "-5321234567"),
            (Raw::Integer(0), "0"),
            (Raw::Integer(20260107), "20260107"),
            (Raw::Real(532_123_456.7), "532123456.7"),
            (Raw::Real(-5_321_234_567.0), "-5321234567"),
            (Raw::Real(-0.0), "-0"),
            (Raw::Real(f64::NAN), "NaN"),
            (Raw::Real(f64::INFINITY), "inf"),
            (Raw::Real(f64::NEG_INFINITY), "-inf"),
        ] {
            let date = date(raw);
            assert_eq!(date.value, None);
            assert_eq!(date.warning, Some(Warning::Unparsed));
            assert_eq!(date.raw.as_deref(), Some(expected_raw));
            let phone = phone(raw);
            assert_eq!(phone.value, None);
            assert_eq!(phone.warning, Some(Warning::Unparsed));
            assert_eq!(phone.raw.as_deref(), Some(expected_raw));
        }
    }

    #[test]
    fn species_aliases_include_turkish_case_and_ascii_forms() {
        for (input, expected) in [
            ("Kedi", Species::Cat),
            ("kedi", Species::Cat),
            ("KEDİ", Species::Cat),
            ("KEDI", Species::Cat),
            ("cat", Species::Cat),
            ("CAT", Species::Cat),
            ("Köpek", Species::Dog),
            ("Kopek", Species::Dog),
            ("KÖPEK", Species::Dog),
            ("dog", Species::Dog),
            ("Kuş", Species::Bird),
            ("Kus", Species::Bird),
            ("KUŞ", Species::Bird),
            ("muhabbet kuşu", Species::Bird),
            ("MUHABBET KUŞU", Species::Bird),
            ("muhabbet kusu", Species::Bird),
            ("Tavşan", Species::Rabbit),
            ("Tavsan", Species::Rabbit),
            ("TAVŞAN", Species::Rabbit),
            ("Diğer", Species::Other),
            ("Diger", Species::Other),
            ("DİĞER", Species::Other),
            ("DIGER", Species::Other),
            (" \tKedi\n", Species::Cat),
        ] {
            check_text(species(Raw::Text(input)), input, Some(expected), None);
        }
    }

    #[test]
    fn free_text_species_become_other_with_a_note() {
        for input in ["At", "iguana", "🐈", "  unknown species  "] {
            check_text(
                species(Raw::Text(input)),
                input,
                Some(Species::Other),
                Some(Warning::Adjusted),
            );
        }
    }

    #[test]
    fn sex_aliases_include_turkish_case_and_ascii_forms() {
        for (input, expected) in [
            ("Erkek", Sex::Male),
            ("ERKEK", Sex::Male),
            ("E", Sex::Male),
            ("e", Sex::Male),
            ("male", Sex::Male),
            ("MALE", Sex::Male),
            ("Dişi", Sex::Female),
            ("Disi", Sex::Female),
            ("DİŞİ", Sex::Female),
            ("DIŞI", Sex::Female),
            ("D", Sex::Female),
            ("d", Sex::Female),
            ("female", Sex::Female),
            ("FEMALE", Sex::Female),
            (" \tDişi\n", Sex::Female),
        ] {
            check_text(sex(Raw::Text(input)), input, Some(expected), None);
        }
    }

    #[test]
    fn free_text_sex_becomes_unknown_with_a_note() {
        for input in ["unknown", "?", "🐇", "  unspecified  "] {
            check_text(
                sex(Raw::Text(input)),
                input,
                Some(Sex::Unknown),
                Some(Warning::Adjusted),
            );
        }
    }

    #[test]
    fn empty_labels_and_nulls_are_lossless() {
        for input in ["", " \t\n", "\u{2003}"] {
            check_text(species(Raw::Text(input)), input, None, None);
            check_text(sex(Raw::Text(input)), input, Some(Sex::Unknown), None);
        }
        assert_eq!(
            species(Raw::Null),
            Normalized {
                value: None,
                raw: None,
                warning: None
            }
        );
        assert_eq!(
            sex(Raw::Null),
            Normalized {
                value: Some(Sex::Unknown),
                raw: None,
                warning: None
            }
        );
        assert_eq!(
            date(Raw::Null),
            Normalized {
                value: None,
                raw: None,
                warning: Some(Warning::Unparsed)
            }
        );
        assert_eq!(
            phone(Raw::Null),
            Normalized {
                value: None,
                raw: None,
                warning: Some(Warning::Unparsed)
            }
        );
    }

    #[test]
    fn numeric_labels_are_preserved_and_adjusted() {
        for (raw, expected_raw) in [
            (Raw::Integer(42), "42"),
            (Raw::Real(42.5), "42.5"),
            (Raw::Real(f64::NAN), "NaN"),
        ] {
            assert_eq!(
                species(raw),
                Normalized {
                    value: Some(Species::Other),
                    raw: Some(expected_raw.to_owned()),
                    warning: Some(Warning::Adjusted),
                }
            );
            assert_eq!(
                sex(raw),
                Normalized {
                    value: Some(Sex::Unknown),
                    raw: Some(expected_raw.to_owned()),
                    warning: Some(Warning::Adjusted),
                }
            );
        }
    }

    #[test]
    fn very_long_inputs_do_not_panic_or_lose_raw_text() {
        for input in ["🐈İ\0 ".repeat(100_000), "5".repeat(1_000_000)] {
            assert_eq!(date(Raw::Text(&input)).raw.as_deref(), Some(input.as_str()));
            assert_eq!(
                phone(Raw::Text(&input)).raw.as_deref(),
                Some(input.as_str())
            );
            assert_eq!(
                species(Raw::Text(&input)).raw.as_deref(),
                Some(input.as_str())
            );
            assert_eq!(sex(Raw::Text(&input)).raw.as_deref(), Some(input.as_str()));
        }
    }

    #[test]
    fn one_thousand_deterministic_random_strings_never_panic() {
        let alphabet = [
            'a', 'Z', 'İ', 'ı', 'Ş', 'ğ', 'ü', '0', '1', '5', '9', '-', '.', '/', '+', '(', ')',
            ',', ' ', '\t', '\n', '\0', '🐈', '🐕', '中', '\u{301}', '\u{2003}',
        ];
        let mut state = 0x5eed_u64;
        for _ in 0..1_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let length = (state % 256) as usize;
            let mut input = String::new();
            for _ in 0..length {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                input.push(alphabet[(state % alphabet.len() as u64) as usize]);
            }
            let result = std::panic::catch_unwind(|| {
                date(Raw::Text(&input));
                phone(Raw::Text(&input));
                species(Raw::Text(&input));
                sex(Raw::Text(&input));
            });
            assert!(result.is_ok(), "normalizers panicked on {input:?}");
        }
    }
}
