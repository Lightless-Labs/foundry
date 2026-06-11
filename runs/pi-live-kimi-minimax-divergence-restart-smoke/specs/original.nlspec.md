# Slugify Smoke NLSpec v1

## Why

Produce predictable ASCII URL slugs from short content titles.

## What

Expose `slugify(input: &str) -> String`.

## How

- Trim surrounding whitespace.
- Lowercase ASCII letters.
- Keep ASCII digits.
- Treat ASCII whitespace and ASCII punctuation as separators.
- Collapse repeated separators into one hyphen.
- Strip leading and trailing hyphens.
- Return `untitled` when no letters or digits remain after normalization.

## Done

- ASCII words slugify deterministically.
- Repeated whitespace collapses.
- Punctuation becomes separators.
- Leading/trailing separators are stripped.
- Numeric tokens are preserved.
- Already-slugged input is stable.
- Empty input falls back to `untitled`.
