# NLSpec: Slugify Rust Library

## Why

Product URLs and identifiers need stable, readable slugs derived from user-facing titles. The conversion must be deterministic and free from hidden locale or filesystem dependencies.

## What

Build a Rust library crate named `slugify_smoke` exposing:

```rust
pub fn slugify(input: &str) -> String
```

The function returns an ASCII, lowercase slug.

## Data Model

- Input: any UTF-8 string slice.
- Output: an owned `String`.
- A slug token is a non-empty run of ASCII lowercase letters or ASCII digits.
- Tokens are joined by one ASCII hyphen (`-`).

## How

Implement a Rust library crate named `slugify_smoke` with `pub fn slugify(input: &str) -> String`.

Use these implementation rules:

1. Convert ASCII letters to lowercase.
2. Keep ASCII digits unchanged.
3. Transliterate these Latin-1 characters before tokenization:
   - `à á â ã ä å ā` -> `a`
   - `ç ć` -> `c`
   - `è é ê ë ē` -> `e`
   - `ì í î ï ī` -> `i`
   - `ñ` -> `n`
   - `ò ó ô õ ö ø ō` -> `o`
   - `ù ú û ü ū` -> `u`
   - `ý ÿ` -> `y`
   - `æ` -> `ae`
   - `œ` -> `oe`
   - `ß` -> `ss`
4. Treat any other non-alphanumeric character as a separator.
5. Collapse adjacent separators to a single hyphen.
6. Trim leading and trailing separators.
7. Return an empty string when no slug tokens remain.
8. Do not panic for empty strings, whitespace-only strings, punctuation-only strings, emoji, or mixed Unicode input.

Reference examples:

| Input | Output |
|---|---|
| `Hello, World!` | `hello-world` |
| `  Multiple---spaces___and punctuation!!! ` | `multiple-spaces-and-punctuation` |
| `Crème brûlée déjà vu` | `creme-brulee-deja-vu` |
| `Æther & Straße` | `aether-strasse` |
| `version 2.0.1` | `version-2-0-1` |
| `💡🔥` | `` |

## Definition of Done

- [ ] Exposes a Rust library crate named `slugify_smoke`.
- [ ] `slugify("Hello, World!")` returns `hello-world`.
- [ ] Collapses repeated whitespace, punctuation, underscores, and hyphens into single separators.
- [ ] Trims leading and trailing separators.
- [ ] Preserves ASCII digits in place.
- [ ] Lowercases ASCII letters.
- [ ] Handles the listed Latin-1 transliterations, including multi-character `æ`, `œ`, and `ß` expansions.
- [ ] Returns an empty string for empty, whitespace-only, punctuation-only, and emoji-only inputs.
- [ ] Does not panic on arbitrary valid UTF-8 input.

## Integration Smoke Test

A test crate should be able to depend on `slugify_smoke` and assert:

```rust
assert_eq!(slugify_smoke::slugify("Crème brûlée déjà vu"), "creme-brulee-deja-vu");
assert_eq!(slugify_smoke::slugify("Æther & Straße"), "aether-strasse");
```
