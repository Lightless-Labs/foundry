# Slugify Smoke NLSpec v2

## Why

Produce predictable ASCII URL slugs from short content titles, including common accented Latin titles.

## What

Expose `slugify(input: &str) -> String`.

## How

- Normalize input before separator handling by transliterating accented Latin letters to ASCII base letters where there is an obvious one-letter equivalent (for example `é` -> `e`, `ü` -> `u`, `ã` -> `a`).
- Trim surrounding whitespace.
- Lowercase ASCII letters after transliteration.
- Keep ASCII digits.
- Treat ASCII whitespace and ASCII punctuation as separators.
- Treat non-Latin Unicode characters and emoji as separators unless a future NLSpec states a broader transliteration policy.
- Collapse repeated separators into one hyphen.
- Strip leading and trailing hyphens.
- Return `untitled` when no ASCII letters or digits remain after normalization.

## Done

- ASCII words slugify deterministically.
- Repeated whitespace collapses.
- Punctuation becomes separators.
- Leading/trailing separators are stripped.
- Numeric tokens are preserved.
- Already-slugged input is stable.
- Empty input falls back to `untitled`.
- Accented Latin input such as `Crème brûlée` slugifies to `creme-brulee`.
- Non-Latin-only or emoji-only input falls back to `untitled`.
