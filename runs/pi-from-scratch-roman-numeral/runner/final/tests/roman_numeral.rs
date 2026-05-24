use foundry_pi_roman_numeral::{from_roman, to_roman, RomanError};

const GOLDEN_VECTORS: &[(u16, &str)] = &[
    (1, "I"),
    (4, "IV"),
    (9, "IX"),
    (14, "XIV"),
    (40, "XL"),
    (44, "XLIV"),
    (49, "XLIX"),
    (58, "LVIII"),
    (90, "XC"),
    (99, "XCIX"),
    (400, "CD"),
    (944, "CMXLIV"),
    (1987, "MCMLXXXVII"),
    (2024, "MMXXIV"),
    (3999, "MMMCMXCIX"),
];

#[test]
fn integration_smoke_test_uses_public_api() {
    assert_eq!(to_roman(2024), Ok("MMXXIV".to_string()));
    assert_eq!(from_roman("MMXXIV"), Ok(2024));
    assert_eq!(from_roman("IIII"), Err(RomanError::NonCanonical));
}

#[test]
fn to_roman_matches_all_golden_vectors_exactly() {
    for &(number, expected) in GOLDEN_VECTORS {
        assert_eq!(
            to_roman(number),
            Ok(expected.to_string()),
            "to_roman({number}) should be exactly {expected}"
        );
    }
}

#[test]
fn from_roman_parses_all_golden_vectors_back() {
    for &(expected, roman) in GOLDEN_VECTORS {
        assert_eq!(
            from_roman(roman),
            Ok(expected),
            "from_roman({roman:?}) should be {expected}"
        );
    }
}

#[test]
fn representative_values_round_trip_in_both_directions() {
    let values = [
        1_u16, 2, 3, 4, 5, 8, 9, 10, 39, 44, 99, 400, 944, 1666, 2024, 3999,
    ];

    for value in values {
        let roman = to_roman(value).expect("representative value should format");
        assert_eq!(from_roman(&roman), Ok(value), "{value} -> {roman} -> {value}");

        let reparsed = from_roman(&roman).expect("formatted numeral should parse");
        assert_eq!(to_roman(reparsed), Ok(roman.clone()), "{roman} should stay canonical");
    }
}

#[test]
fn to_roman_rejects_values_outside_supported_range() {
    assert_eq!(to_roman(0), Err(RomanError::OutOfRange));
    assert_eq!(to_roman(4000), Err(RomanError::OutOfRange));
}

#[test]
fn from_roman_rejects_empty_input() {
    assert_eq!(from_roman(""), Err(RomanError::Empty));
}

#[test]
fn from_roman_rejects_required_non_canonical_inputs() {
    let invalid_cases = ["IIII", "VV", "IC", "IL", "XM", "MCMC", "VX"];

    for roman in invalid_cases {
        assert_eq!(
            from_roman(roman),
            Err(RomanError::NonCanonical),
            "{roman:?} uses Roman symbols but is not canonical"
        );
    }
}

#[test]
fn from_roman_rejects_lowercase_and_non_roman_characters() {
    assert_eq!(from_roman("ix"), Err(RomanError::InvalidCharacter('i')));
    assert_eq!(from_roman("ABC"), Err(RomanError::InvalidCharacter('A')));
}
