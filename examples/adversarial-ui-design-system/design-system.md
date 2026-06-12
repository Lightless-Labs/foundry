---
title: Tiny Adversarial UI Design System Fixture
created: 2026-06-12
---

# Tiny Adversarial UI Design System Fixture

This fixture is intentionally small. It is a documentation-only spike for testing whether a design system can act as a UI NLSpec before the public plugin takes on browser/image dependencies.

## Tokens

### Spacing

| Token | Value | Use |
|-------|-------|-----|
| `space-1` | `4px` | tight inline gaps |
| `space-2` | `8px` | button padding and compact gaps |
| `space-4` | `16px` | card padding and grid gap |

### Color

| Token | Value | Use |
|-------|-------|-----|
| `color-surface` | `#ffffff` | card background |
| `color-border` | `#d0d7de` | card/button border |
| `color-accent` | `#2563eb` | primary button background |

### Typography

| Token | Value | Use |
|-------|-------|-----|
| `type-title` | `20px/28px 600` | card title |
| `type-body` | `14px/20px 400` | body text |
| `type-button` | `14px/20px 600` | button label |

## Components

### Card

- Width fills its grid column.
- Padding is `space-4` on every side.
- Border is `1px solid color-border`.
- Border radius is `8px`.
- Title uses `type-title` and wraps after two lines with ellipsis.
- Body uses `type-body` and may wrap naturally.

### Button

- Primary button background is `color-accent`.
- Primary button text is white.
- Padding is `space-2 space-4`.
- Border radius is `6px`.
- Disabled state uses `color-border` as background and body text color.

### Grid

- At viewport widths below `640px`, render one column.
- At viewport widths `640px` and above, render two equal columns.
- Grid gap is `space-4`.

## UI Information Barrier Notes

- Red/comparator may see hidden content, reference mocks, rendered screenshots, visual diffs, and comparator rationales.
- Green may see this design system and public examples, then only opaque PASS/FAIL outcome labels for hidden/generated cases.
- Human-readable mappings for hidden labels such as `T-101` stay with the orchestrator/red side.
