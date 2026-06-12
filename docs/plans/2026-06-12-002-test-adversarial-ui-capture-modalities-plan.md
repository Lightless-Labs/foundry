---
title: Broaden adversarial UI capture modalities beyond web screenshots
created: 2026-06-12
status: completed
completed: 2026-06-12
todo: todos/adversarial-ui-investigation.md
---

# Broaden Adversarial UI Capture Modalities Beyond Web Screenshots

## Goal

Update the adversarial UI spike so "screenshot" and "UI capture" are explicitly modality-agnostic: web browser screenshots, device simulator/emulator captures, and physical-device screen captures all fit the same Foundry barrier model.

## Scope

- Add a capture-surface fixture covering at least:
  - `web_browser`,
  - `simulator_emulator`,
  - `physical_device`.
- Document common and modality-specific metadata needed for stable comparison.
- Preserve UI information-barrier rules across all modalities:
  - green sees design-system/public examples plus opaque PASS/FAIL labels only for hidden/generated cases,
  - hidden screenshots, visual diffs, OCR text, capture metadata, comparator prompts, and rationales remain red/orchestrator/comparator-only.
- Add a lightweight validator for the capture-surface fixture.

## Non-goals

- Do not add Playwright, Appium, XCTest, Android emulator, device-farm, or image-processing dependencies.
- Do not perform real screenshot capture in this slice.
- Do not claim screenshot/vision reliability from static modality fixtures.
- Do not create a dedicated `foundry:adversarial-ui` skill or private engine changes.

## Acceptance

- [x] `examples/adversarial-ui-design-system/fixtures/capture-surfaces.json` covers web browser, simulator/emulator, and physical-device capture.
- [x] The fixture includes modality-specific stability, privacy, and coordinate-space metadata.
- [x] A validator rejects missing modalities and obvious green-visible leaks.
- [x] README/docs explain that UI capture is not browser-only.
- [x] Existing UI PromptEnvelope validation still passes.
- [x] Agent validation still passes.

## Validation Log

2026-06-12:

- Dispatched barrier-integrity and feasibility child agents through PromptEnvelope-backed `foundry_team`.
- Both recommended a minimal static fixture plus validator, avoiding heavyweight screenshot/device dependencies for this slice.
- Added `examples/adversarial-ui-design-system/fixtures/capture-surfaces.json` with web browser, simulator/emulator, and physical-device contracts.
- Added `tests/validate-adversarial-ui-capture-surfaces.sh` and wired it into the handoff validator list so it is part of the documented check surface.
- `python3 -m json.tool examples/adversarial-ui-design-system/fixtures/capture-surfaces.json` — passed.
- `tests/validate-adversarial-ui-capture-surfaces.sh` — passed for 3 capture surfaces.
- `tests/validate-barrier-envelopes.sh examples/adversarial-ui-design-system/dispatch` — passed.
- `tests/validate-agents.sh` — passed 224/224.
