#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange,
    Empty,
    InvalidCharacter(char),
    NonCanonical,
}

const PAIRS: [(&str, u16); 13] = [
    ("M", 1000),
    ("CM", 900),
    ("D", 500),
    ("CD", 400),
    ("C", 100),
    ("XC", 90),
    ("L", 50),
    ("XL", 40),
    ("X", 10),
    ("IX", 9),
    ("V", 5),
    ("IV", 4),
    ("I", 1),
];

pub fn to_roman(n: u16) -> Result<String, RomanError> {
    if !(1..=3999).contains(&n) {
        return Err(RomanError::OutOfRange);
    }

    let mut remaining = n;
    let mut out = String::new();

    for &(symbol, value) in &PAIRS {
        while remaining >= value {
            out.push_str(symbol);
            remaining -= value;
        }
    }

    Ok(out)
}

pub fn from_roman(s: &str) -> Result<u16, RomanError> {
    if s.is_empty() {
        return Err(RomanError::Empty);
    }

    let first = s.chars().next().expect("non-empty string has a first char");
    if !is_roman_char(first) {
        return Err(RomanError::InvalidCharacter(first));
    }

    let mut idx = 0;
    let mut value: u32 = 0;

    while idx < s.len() {
        let rest = &s[idx..];
        let mut matched = false;

        for &(symbol, token_value) in &PAIRS {
            if rest.starts_with(symbol) {
                value += token_value as u32;
                idx += symbol.len();
                matched = true;
                break;
            }
        }

        if !matched {
            let ch = rest.chars().next().expect("idx is within the string");
            if !is_roman_char(ch) {
                return Err(RomanError::InvalidCharacter(ch));
            }
            return Err(RomanError::NonCanonical);
        }
    }

    if !(1..=3999).contains(&value) {
        return Err(RomanError::OutOfRange);
    }

    let regenerated = to_roman(value as u16)?;
    if regenerated == s {
        Ok(value as u16)
    } else {
        Err(RomanError::NonCanonical)
    }
}

fn is_roman_char(ch: char) -> bool {
    matches!(ch, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M')
}
