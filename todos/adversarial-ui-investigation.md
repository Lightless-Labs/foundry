---
title: Investigate adversarial red/green for UI via design systems
origin: 2026-04-04 brainstorm — adversarial-ui-design-system
priority: future
status: completed first spike — screenshot/vision hardening remains future work
updated: 2026-06-12
---

# Investigate Adversarial UI

**Addendum:** 2026-06-12 — local research landed in `docs/research/2026-06-12-adversarial-ui-investigation-research.md`, and a dedicated spike plan was completed at `docs/plans/2026-06-12-001-test-adversarial-ui-design-system-investigation-plan.md`. Learnings/feasibility child agents recommended a tiny manual design-system fixture before adding browser/image dependencies. The first fixture now lives at `examples/adversarial-ui-design-system/`: Level 1/2 are represented as static fixture/outcome artifacts, and Level 3 was trialed through a PromptEnvelope-backed text measurement-snapshot comparator. It proves the barrier shape, not screenshot/vision reliability.

Explore whether the three-level testing strategy (mock matching, held-back instances, generative composition) works in practice.

## Key unknowns to resolve

- LLM comparator reliability: how consistent is a lightweight model at evaluating design system compliance from screenshots?
- Threshold calibration: what's the right tolerance for "matches the spec" given browser rendering differences?
- Level 3 feasibility: can the red team reliably compose novel test layouts from design system rules and predict what they should look like?
- Tooling: what screenshot/comparison stack works best (pixelmatch vs perceptual hash vs LLM evaluation)?

## Suggested approach

Pick a small design system (e.g., a card + button + grid with 3 tokens each for spacing, color, typography) and try all three levels manually before building any tooling.

## Progress

- [x] Research local prior art and risks.
- [x] Create a dedicated tiny-spike plan.
- [x] Build or document the tiny design-system fixture.
- [x] Trial Level 1 mock matching and Level 2 held-back instances with PASS/FAIL-only labels.
- [x] Trial one Level 3 generative composition and record comparator reliability caveats.
- [ ] Future hardening: run real screenshot/vision comparison with negative controls and rerun-agreement measurement.

See: `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`
