---
title: Add adversarial UI visual comparison controls
created: 2026-06-12
status: completed
completed: 2026-06-12
todo: todos/adversarial-ui-investigation.md
---

# Add Adversarial UI Visual Comparison Controls

## Goal

Move the adversarial UI spike from static capture contracts to an executable, dependency-light visual comparison check with positive cases, negative controls, rerun-agreement, and cross-file validation against capture-surface IDs.

## Scope

- Add a visual-comparison case manifest under `examples/adversarial-ui-design-system/fixtures/`.
- Cover multiple capture modalities from `capture-surfaces.json`, including at least:
  - web browser,
  - simulator/emulator,
  - physical device.
- Use generated PNG-like image comparisons via Python/Pillow so the public plugin does not need browser, emulator, Appium, XCTest, or device-farm dependencies.
- Include negative controls where rendered captures intentionally differ from references.
- Validate rerun agreement by comparing each case more than once and requiring identical outcomes.
- Validate that every `surface_id` and capture modality in the visual-comparison manifest exists in `capture-surfaces.json`.

## Non-goals

- Do not claim this is a real app screenshot capture run.
- Do not add heavyweight UI runtime dependencies.
- Do not expose hidden case names, scene definitions, pixel diffs, or comparator rationales to green.
- Do not introduce private engine changes.

## Acceptance

- [x] Visual-comparison manifest covers web browser, simulator/emulator, and physical-device surfaces.
- [x] Validator executes actual image comparisons and includes PASS and FAIL controls.
- [x] Validator rejects orphaned `surface_id` or mismatched modality references.
- [x] Validator proves rerun agreement for deterministic image comparisons.
- [x] README, todo, and handoff are updated.
- [x] Existing UI capture-surface and PromptEnvelope validators still pass.
- [x] Agent validation still passes.

## Validation Log

2026-06-12:

- Started from user feedback that UI/screenshots span web browser, simulator/emulator, and physical-device capture rather than web-only screenshots.
- Added `examples/adversarial-ui-design-system/fixtures/visual-comparison-controls.json` with six opaque controls: PASS and FAIL controls for web browser, simulator/emulator, and physical-device surfaces.
- Added `tests/validate-adversarial-ui-visual-controls.sh` using only Python stdlib image arrays, so no Pillow/browser/device dependency is required.
- The validator cross-checks every `surface_id` against `capture-surfaces.json`, requires all capture-surface IDs to be referenced, requires both PASS and FAIL outcomes, and checks deterministic rerun agreement.
- `python3 -m json.tool examples/adversarial-ui-design-system/fixtures/visual-comparison-controls.json` — passed.
- `tests/validate-adversarial-ui-visual-controls.sh` — validated 6 controls across 3 modalities: 3 expected PASS controls and 3 expected FAIL negative controls.
- `tests/validate-adversarial-ui-capture-surfaces.sh` — passed.
- `tests/validate-barrier-envelopes.sh examples/adversarial-ui-design-system/dispatch` — passed.
- `tests/validate-agents.sh` — passed 224/224.
