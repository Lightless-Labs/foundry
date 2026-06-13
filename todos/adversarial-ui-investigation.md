---
title: Investigate adversarial red/green for UI via design systems
origin: 2026-04-04 brainstorm — adversarial-ui-design-system
priority: future
status: completed first spike — screenshot/vision hardening remains future work
updated: 2026-06-13
---

# Investigate Adversarial UI

**Addendum:** 2026-06-12 — local research landed in `docs/research/2026-06-12-adversarial-ui-investigation-research.md`, and a dedicated spike plan was completed at `docs/plans/2026-06-12-001-test-adversarial-ui-design-system-investigation-plan.md`. Learnings/feasibility child agents recommended a tiny manual design-system fixture before adding browser/image dependencies. The first fixture now lives at `examples/adversarial-ui-design-system/`: Level 1/2 are represented as static fixture/outcome artifacts, and Level 3 was trialed through a PromptEnvelope-backed text measurement-snapshot comparator. It proves the barrier shape, not screenshot/vision reliability.

**Addendum:** 2026-06-12 — broadened the spike beyond browser assumptions via `docs/plans/2026-06-12-002-test-adversarial-ui-capture-modalities-plan.md`. Added `fixtures/capture-surfaces.json` covering web browsers, simulators/emulators, and physical-device captures, plus `tests/validate-adversarial-ui-capture-surfaces.sh` for modality and leak guardrails.

**Addendum:** 2026-06-12 — added executable synthetic visual comparison controls via `docs/plans/2026-06-12-003-test-adversarial-ui-visual-comparison-controls-plan.md`. `fixtures/visual-comparison-controls.json` now includes PASS/FAIL controls for every capture surface ID, and `tests/validate-adversarial-ui-visual-controls.sh` cross-checks those IDs against `capture-surfaces.json`, verifies rerun agreement, and uses only Python stdlib image arrays (no Pillow/browser/device dependency).

**Addendum:** 2026-06-13 — wired the UI validators into the broader fast validation path via `docs/plans/2026-06-13-001-test-public-plugin-validation-entrypoint-plan.md`. Added `tests/validate-public-plugin.sh` and `npm run validate`, both of which run the capture-surface and visual-control validators alongside existing fast public-plugin checks.

**Addendum:** 2026-06-13 — added file-backed raster controls via `docs/plans/2026-06-13-002-test-adversarial-ui-file-backed-raster-controls-plan.md`. `fixtures/screenshots/*.ppm` now provides tiny screenshot-like ASCII PPM artifacts, and `tests/validate-adversarial-ui-visual-controls.sh` verifies artifact existence, SHA-256, parseability, dimensions, comparison outcome, and rerun agreement. This is still a stdlib-only surrogate, not live browser/device capture or vision-model validation.

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
- [x] Add modality-agnostic capture contracts for browser, simulator/emulator, and physical-device screenshots.
- [x] Add synthetic visual comparison controls with positive/negative cases and cross-file surface validation.
- [x] Wire adversarial UI validators into the aggregate fast validation path.
- [x] Add file-backed raster controls with artifact hash validation and positive/negative cases.
- [ ] Future hardening: run real screenshot/vision comparison with negative controls and rerun-agreement measurement.

See: `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`
