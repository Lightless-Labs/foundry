---
title: Roman numeral converter/parser NLSpec
status: reviewed
reviewed: 2026-05-24
source_spec: ../specs/roman-numeral-spec.md
---

# Roman Numeral Converter/Parser NLSpec

## Why

Provide a deterministic, dependency-free Rust API for converting between integers and canonical Roman numerals. The feature is small but convention-sensitive, so the spec pins exact golden vectors and invalid examples.

## What

Implement a Rust library API in `src/lib.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange,
    Empty,
    InvalidCharacter(char),
    NonCanonical,
}

pub fn to_roman(n: u16) -> Result<String, RomanError>;
pub fn from_roman(s: &str) -> Result<u16, RomanError>;
```

Supported range: `1..=3999` only.

Canonical output uses uppercase subtractive notation with symbols and values:

- `M=1000`
- `CM=900`
- `D=500`
- `CD=400`
- `C=100`
- `XC=90`
- `L=50`
- `XL=40`
- `X=10`
- `IX=9`
- `V=5`
- `IV=4`
- `I=1`

## How

Implement `to_roman` by greedily subtracting from the ordered canonical pair table above. Return `RomanError::OutOfRange` for values outside `1..=3999`.

Implement `from_roman` with a strict canonicality check:

1. Reject an empty string with `RomanError::Empty`.
2. Reject the first character outside `I`, `V`, `X`, `L`, `C`, `D`, `M` with `RomanError::InvalidCharacter(ch)`.
3. Parse the string left to right using the same ordered canonical pair table, accumulating the matched values.
4. Reject strings that cannot be fully matched with canonical tokens.
5. Reject values outside `1..=3999`.
6. Convert the parsed value back with `to_roman`; accept only if the regenerated string exactly equals the input. Otherwise return `RomanError::NonCanonical`.

Do not add dependencies.

## Done

- [ ] `to_roman` returns exact golden vectors:
  - `1 -> I`
  - `4 -> IV`
  - `9 -> IX`
  - `14 -> XIV`
  - `40 -> XL`
  - `44 -> XLIV`
  - `49 -> XLIX`
  - `58 -> LVIII`
  - `90 -> XC`
  - `99 -> XCIX`
  - `400 -> CD`
  - `944 -> CMXLIV`
  - `1987 -> MCMLXXXVII`
  - `2024 -> MMXXIV`
  - `3999 -> MMMCMXCIX`
- [ ] `from_roman` parses each golden vector back to its number.
- [ ] Round-trip holds for representative values `1, 2, 3, 4, 5, 8, 9, 10, 39, 44, 99, 400, 944, 1666, 2024, 3999`.
- [ ] `to_roman(0)` and `to_roman(4000)` return `RomanError::OutOfRange`.
- [ ] `from_roman("")` returns `RomanError::Empty`.
- [ ] `from_roman` rejects invalid/non-canonical strings: `IIII`, `VV`, `IC`, `IL`, `XM`, `MCMC`, `VX`, `ix`, `ABC`.

## Data Model

`RomanError` is the only public error type. Tests may compare errors by equality.

## Integration Smoke Test

A Rust test imports `to_roman`, `from_roman`, and `RomanError`, checks `to_roman(2024) == "MMXXIV"`, checks `from_roman("MMXXIV") == 2024`, and checks `from_roman("IIII") == RomanError::NonCanonical`.
