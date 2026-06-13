---
title: Add file-backed adversarial UI raster controls
created: 2026-06-13
status: completed
completed: 2026-06-13
todo: todos/adversarial-ui-investigation.md
---

# Add File-Backed Adversarial UI Raster Controls

## Goal

Harden the adversarial UI spike beyond in-memory synthetic scene definitions by validating comparator behavior against checked-in screenshot-like raster artifacts with positive and negative controls, rerun agreement, and the same green-visible PASS/FAIL-only barrier.

## Scope

- Add small checked-in raster fixture artifacts for at least one capture modality.
- Reference those artifacts from `fixtures/visual-comparison-controls.json` with hashes, surface IDs, capture kind, and barrier/privacy metadata.
- Extend `tests/validate-adversarial-ui-visual-controls.sh` to load the image artifacts from disk, verify hashes, compare pixels, require rerun agreement, and enforce PASS/FAIL outcome controls.
- Keep the dependency profile stdlib-only; do not add browser, emulator, device-farm, Pillow, PNG parsing, or live vision-model requirements.
- Wire the hardening through the existing aggregate validation entrypoint by extending the visual-controls validator already called by `tests/validate-public-plugin.sh`.

## Non-goals

- Do not claim this is a live browser/simulator/device capture run or a real PNG/screenshot pipeline.
- Do not add private engine changes.
- Do not expose reference screenshots, rendered screenshots, diffs, capture metadata, or comparator rationale to green.

## Acceptance

- [x] File-backed raster controls include at least one PASS and one FAIL case.
- [x] Validator checks artifact existence, SHA-256, parseability, dimensions, comparison outcome, and rerun agreement.
- [x] T-401 is documented as the unchanged-image PASS control, while T-402 is documented as the changed-image FAIL control.
- [x] Validator rejects green-visible leakage around screenshot/capture/vision comparator details.
- [x] README, todo, and handoff document the new limitation boundary.
- [x] Fast aggregate validation passes locally.

## Validation Log

2026-06-13:

- Added `examples/adversarial-ui-design-system/fixtures/screenshots/*.ppm` as tiny screenshot-like ASCII PPM raster artifacts that remain red/orchestrator/comparator-only.
- Extended `fixtures/visual-comparison-controls.json` with `file_backed_controls` for `T-401` and `T-402`.
- `T-401` is intentionally the unchanged-image PASS control: reference and rendered artifacts have identical SHA-256 hashes.
- `T-402` is intentionally the changed-image FAIL control: it reuses the PASS reference raster and changes only the rendered button color.
- Extended `tests/validate-adversarial-ui-visual-controls.sh` to verify artifact paths, SHA-256 hashes, PPM parseability, dimensions, PASS/FAIL outcomes, and rerun agreement without external dependencies.
- `python3 -m json.tool examples/adversarial-ui-design-system/fixtures/visual-comparison-controls.json` — passed.
- `tests/validate-adversarial-ui-visual-controls.sh` — passed with 6 synthetic controls and 2 file-backed controls.
- `tests/validate-adversarial-ui-capture-surfaces.sh` — passed.
- `tests/validate-barrier-envelopes.sh examples/adversarial-ui-design-system/dispatch` — passed.
- `tests/validate-public-plugin.sh` — passed.
