---
title: Investigate adversarial red/green for UI via design systems
origin: 2026-04-04 brainstorm — adversarial-ui-design-system
priority: future
status: ready
---

# Investigate Adversarial UI

Explore whether the three-level testing strategy (mock matching, held-back instances, generative composition) works in practice.

## Key unknowns to resolve

- LLM comparator reliability: how consistent is a lightweight model at evaluating design system compliance from screenshots?
- Threshold calibration: what's the right tolerance for "matches the spec" given browser rendering differences?
- Level 3 feasibility: can the red team reliably compose novel test layouts from design system rules and predict what they should look like?
- Tooling: what screenshot/comparison stack works best (pixelmatch vs perceptual hash vs LLM evaluation)?

## Suggested approach

Pick a small design system (e.g., a card + button + grid with 3 tokens each for spacing, color, typography) and try all three levels manually before building any tooling.

See: `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`
