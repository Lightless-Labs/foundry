/// Returns true for Unicode combining marks that should be absorbed into the
/// preceding base letter rather than treated as separators.
fn is_combining_mark(c: char) -> bool {
    matches!(c,
        '\u{0300}'..='\u{036F}' | // Combining Diacritical Marks
        '\u{1AB0}'..='\u{1AFF}' | // Combining Diacritical Marks Extended
        '\u{1DC0}'..='\u{1DFF}' | // Combining Diacritical Marks Supplement
        '\u{20D0}'..='\u{20FF}' | // Combining Diacritical Marks for Symbols
        '\u{FE20}'..='\u{FE2F}'   // Combining Half Marks
    )
}

/// Transliterate a precomposed Latin accented letter to its obvious ASCII base.
/// Letters without an obvious single-letter equivalent (ligatures, eth, thorn,
/// sharp s, etc.) return None and are treated as separators.
fn latin_base(c: char) -> Option<char> {
    match c {
        // Latin-1 Supplement: accented vowels, cedilla c, tilde n, stroke o.
        '\u{00C0}'..='\u{00C5}' | '\u{00E0}'..='\u{00E5}' => Some('a'),
        '\u{00C7}' | '\u{00E7}' => Some('c'),
        '\u{00C8}'..='\u{00CB}' | '\u{00E8}'..='\u{00EB}' => Some('e'),
        '\u{00CC}'..='\u{00CF}' | '\u{00EC}'..='\u{00EF}' => Some('i'),
        '\u{00D1}' | '\u{00F1}' => Some('n'),
        '\u{00D2}'..='\u{00D6}' | '\u{00F2}'..='\u{00F6}' => Some('o'),
        '\u{00D8}' | '\u{00F8}' => Some('o'),
        '\u{00D9}'..='\u{00DC}' | '\u{00F9}'..='\u{00FC}' => Some('u'),
        '\u{00DD}' | '\u{00FD}' | '\u{00FF}' => Some('y'),

        // Latin Extended-A.
        '\u{0100}'..='\u{0105}' => Some('a'),
        '\u{0106}'..='\u{010D}' => Some('c'),
        '\u{010E}'..='\u{0111}' => Some('d'),
        '\u{0112}'..='\u{011B}' => Some('e'),
        '\u{011C}'..='\u{0123}' => Some('g'),
        '\u{0124}'..='\u{0127}' => Some('h'),
        '\u{0128}'..='\u{0131}' => Some('i'),
        '\u{0134}'..='\u{0135}' => Some('j'),
        '\u{0136}'..='\u{0138}' => Some('k'),
        '\u{0139}'..='\u{0142}' => Some('l'),
        '\u{0143}'..='\u{014B}' => Some('n'),
        '\u{014C}'..='\u{0151}' => Some('o'),
        '\u{0154}'..='\u{0159}' => Some('r'),
        '\u{015A}'..='\u{0161}' => Some('s'),
        '\u{0162}'..='\u{0167}' => Some('t'),
        '\u{0168}'..='\u{0173}' => Some('u'),
        '\u{0174}'..='\u{0175}' => Some('w'),
        '\u{0176}'..='\u{0178}' => Some('y'),
        '\u{0179}'..='\u{017E}' => Some('z'),
        '\u{017F}' => Some('s'),

        // Latin Extended-B: Vietnamese horn letters.
        'Ơ' | 'ơ' => Some('o'),
        'Ư' | 'ư' => Some('u'),

        // Latin Extended Additional: Vietnamese tone/diacritic variants.
        '\u{1EA0}'..='\u{1EB7}' => Some('a'),
        '\u{1EB8}'..='\u{1EC7}' => Some('e'),
        '\u{1EC8}'..='\u{1ECB}' => Some('i'),
        '\u{1ECC}'..='\u{1EE3}' => Some('o'),
        '\u{1EE4}'..='\u{1EF1}' => Some('u'),
        '\u{1EF2}'..='\u{1EF9}' => Some('y'),

        _ => None,
    }
}

fn ascii_base_or_transliterate(c: char) -> Option<char> {
    if c.is_ascii_alphabetic() {
        Some(c.to_ascii_lowercase())
    } else if c.is_ascii_digit() {
        Some(c)
    } else {
        latin_base(c)
    }
}

pub fn slugify(input: &str) -> String {
    let mut result = String::new();
    let mut prev_was_sep = true;
    let mut prev_output_was_alnum = false;

    for c in input.trim().chars() {
        if let Some(base) = ascii_base_or_transliterate(c) {
            result.push(base);
            prev_was_sep = false;
            prev_output_was_alnum = true;
        } else if is_combining_mark(c) && prev_output_was_alnum {
            // Absorb combining mark into the preceding letter.
        } else {
            // Separator: ASCII whitespace/punctuation, non-Latin, emoji,
            // ligatures, or a stray combining mark with no preceding letter.
            prev_output_was_alnum = false;
            if !prev_was_sep {
                result.push('-');
                prev_was_sep = true;
            }
        }
    }

    if result.ends_with('-') {
        result.pop();
    }

    if result.is_empty() || !result.chars().any(|c| c.is_ascii_alphanumeric()) {
        return "untitled".to_string();
    }

    result
}
