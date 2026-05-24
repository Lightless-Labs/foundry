---
title: Roman numeral converter/parser
status: reviewed
---

# Roman Numeral Converter/Parser Spec

Build a small Rust library that converts between integers and canonical Roman numerals.

## Requirements

- Expose `to_roman(n: u16) -> Result<String, RomanError>`.
- Expose `from_roman(s: &str) -> Result<u16, RomanError>`.
- Support only the classical range `1..=3999`.
- Emit canonical uppercase subtractive notation.
- Parse only canonical uppercase notation; reject lowercase, empty strings, invalid symbols, out-of-range values, repeated symbols beyond canonical limits, and non-canonical additive/subtractive forms.
- Do not use external crates.

## Golden vectors

| Number | Roman |
|---:|---|
| 1 | I |
| 4 | IV |
| 9 | IX |
| 14 | XIV |
| 40 | XL |
| 44 | XLIV |
| 49 | XLIX |
| 58 | LVIII |
| 90 | XC |
| 99 | XCIX |
| 400 | CD |
| 944 | CMXLIV |
| 1987 | MCMLXXXVII |
| 2024 | MMXXIV |
| 3999 | MMMCMXCIX |

## Invalid examples

`0`, `4000`, empty string, `IIII`, `VV`, `IC`, `IL`, `XM`, `MCMC`, `VX`, `ix`, `ABC`.
