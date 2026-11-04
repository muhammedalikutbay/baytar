//! Search normalization shared by stored search text and query terms.

/// Lowercase with Turkish I rules, fold Turkish accents, and collapse whitespace.
/// Leading/trailing whitespace is removed; digits and other punctuation are retained.
pub fn fold(s: &str) -> String {
    let mut folded = String::with_capacity(s.len());
    let mut space_pending = false;
    for character in s.chars() {
        if character.is_whitespace() {
            space_pending = !folded.is_empty();
            continue;
        }
        if space_pending {
            folded.push(' ');
            space_pending = false;
        }
        // Apply the locale-specific I mapping before Unicode lowercasing so that
        // capital dotted I does not introduce a combining dot into the search key.
        let character = match character {
            'İ' => 'i',
            'I' => 'ı',
            other => other,
        };
        for lowercase in character.to_lowercase() {
            folded.push(match lowercase {
                'ç' => 'c',
                'ğ' => 'g',
                'ı' => 'i',
                'ö' => 'o',
                'ş' => 's',
                'ü' => 'u',
                other => other,
            });
        }
    }
    folded
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn turkish_case_accents_digits_and_whitespace() {
        for (input, expected) in [
            ("İSTANBUL", "istanbul"),
            ("IŞIK", "isik"),
            ("Çağrı Öztürk", "cagri ozturk"),
            ("ÇĞIİÖŞÜ çğıiöşü", "cgiiosu cgiiosu"),
            ("0532 123 45 67", "0532 123 45 67"),
            (" \t Çağrı  \n Öztürk\u{a0}42  ", "cagri ozturk 42"),
            ("", ""),
            (" \t\n\u{2003}", ""),
            ("KEDİ-42 (A)", "kedi-42 (a)"),
        ] {
            assert_eq!(fold(input), expected);
            assert_eq!(fold(&fold(input)), expected);
        }
    }
}
