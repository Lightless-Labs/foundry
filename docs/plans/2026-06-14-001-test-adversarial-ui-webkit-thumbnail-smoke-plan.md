---
title: Add adversarial UI WebKit thumbnail smoke
created: 2026-06-14
status: completed
completed: 2026-06-14
todo: todos/adversarial-ui-investigation.md
---

# Add Adversarial UI WebKit Thumbnail Smoke

## Goal

Move one step beyond raster surrogates by preserving a real WebKit/QuickLook renderer thumbnail smoke for the web-browser modality, with PASS/FAIL negative controls, rerun-agreement measurement, threshold metadata, and Foundry barrier notes.

## Scope

- Use the `web_browser` capture surface first; do not attempt simulator/emulator or physical-device capture in this slice.
- Prefer local macOS WebKit/QuickLook tooling already present in this environment (`qlmanage`, `sips`) over adding Playwright/Puppeteer/browser dependencies.
- Add a tiny static HTML fixture that implements the existing design-system card/button rules.
- Capture at least two reruns for a matching case and a deliberately mismatched case.
- Preserve red/orchestrator-only thumbnail metadata and only green-visible opaque PASS/FAIL outcomes.
- Add a validator/smoke script that records tool availability, artifact hashes, dimensions, thresholds, and rerun agreement.

## Non-goals

- Do not add private engine changes.
- Do not make platform-specific WebKit capture part of the fast aggregate validator unless it can run deterministically everywhere.
- Do not expose thumbnail paths, pixel diffs, HTML hidden-case content, renderer metadata, or comparator rationale to green.
- Do not claim broad browser screenshot/device reliability from one WebKit/QuickLook smoke.

## Acceptance

- [x] Real WebKit/QuickLook thumbnail artifacts exist for at least one PASS and one FAIL web case.
- [x] Rerun agreement is measured across at least two captures per case.
- [x] Negative control fails for a deliberate design-system mismatch.
- [x] Validator records/validates artifact hashes, thumbnail dimensions, threshold/framing metadata, and green-visible PASS/FAIL-only output.
- [x] README, todo, and handoff document the limitation boundary.
- [x] Fast aggregate validation still passes locally.

## Validation Log

2026-06-14:

- Found no local Chrome/Chromium/Firefox/Playwright/Puppeteer tooling; Safari plus macOS `qlmanage`/`sips` were available.
- Added static HTML fixtures under `examples/adversarial-ui-design-system/fixtures/webkit-thumbnail-smoke/`.
- Captured two QuickLook/WebKit thumbnail reruns each for `T-501` reference/rendered and `T-502` reference/rendered. QuickLook emits 800×800 square PNG thumbnails, so this remains thumbnail evidence rather than viewport-accurate browser screenshot evidence.
- Added `manifest.json` recording renderer command assumptions, expected hashes, dimensions, framing caveats, green-visible outcome contract, and PASS/FAIL controls.
- Added `tests/validate-adversarial-ui-webkit-thumbnail-smoke.sh`; it validates committed PNG artifacts, asserts `T-501` HTML is unchanged, asserts `T-502` differs only by `#2563eb -> #d0d7de` button background, verifies rerun hash agreement, and reruns `qlmanage` when available.
- `python3 -m json.tool examples/adversarial-ui-design-system/fixtures/webkit-thumbnail-smoke/manifest.json` — passed.
- `tests/validate-adversarial-ui-webkit-thumbnail-smoke.sh` — passed with live `qlmanage` rerun.
- `tests/validate-adversarial-ui-capture-surfaces.sh` — passed.
- `tests/validate-adversarial-ui-visual-controls.sh` — passed.
- `tests/validate-barrier-envelopes.sh examples/adversarial-ui-design-system/dispatch` — passed.
- `tests/validate-public-plugin.sh` — passed.
