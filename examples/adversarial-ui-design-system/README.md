# Adversarial UI Design System Spike

This example is a tiny documentation-first spike for `todos/adversarial-ui-investigation.md`.

It tests the core idea from `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`: a design system can act as a UI NLSpec, allowing red/green UI work to produce mechanical PASS/FAIL outcomes while preserving Foundry's information barrier.

"UI capture" is deliberately broader than web screenshots. The same barrier model should cover browser-rendered apps, iOS/Android simulator or emulator captures, native desktop/mobile screenshots, and physical-device screen captures from OS APIs or calibrated camera rigs.

## Contents

- `design-system.md` — tiny token/component/layout rules.
- `fixtures/public-mocks.json` — green-visible public examples for Level 1 mock matching.
- `fixtures/hidden-red-cases.json` — red/orchestrator-only held-back Level 2 cases with opaque green labels.
- `fixtures/generative-composition.json` — red/orchestrator/comparator-only Level 3 generated layout.
- `fixtures/capture-surfaces.json` — modality-agnostic capture contracts for web browsers, simulators/emulators, and physical devices.
- `fixtures/visual-comparison-controls.json` — synthetic plus file-backed raster visual comparison PASS/FAIL controls that reference every capture surface.
- `fixtures/screenshots/*.ppm` — tiny screenshot-like ASCII PPM raster fixtures for file-backed comparator controls.
- `fixtures/webkit-thumbnail-smoke/` — static HTML controls and manifest for a macOS WebKit/QuickLook thumbnail smoke.
- `fixtures/playwright-viewport-smoke/` — static HTML controls and manifest for an optional/manual Playwright viewport screenshot smoke.
- `artifacts/webkit-thumbnail-smoke/*.png` — committed QuickLook/WebKit PNG thumbnail reruns.
- `artifacts/webkit-thumbnail-smoke/outcomes.toon` — green-visible opaque outcome labels only (`T-###,PASS/FAIL`).
- `artifacts/level1-level2-outcomes.toon` — example PASS/FAIL-only outcomes.
- `dispatch/level3/ui-comparator.json` — PromptEnvelope for the Level 3 comparator trial.
- `artifacts/level3-comparator-output.json` — persisted comparator output.

## Current Result

- Level 1/2 are represented as fixture artifacts and PASS/FAIL-only outcome examples; no screenshot runtime has been added yet.
- Level 3 was trialed as a text measurement-snapshot comparator dispatch through `foundry_team`; it returned `PASS` with residual risks.
- The trial intentionally does **not** claim screenshot or vision reliability yet.
- Capture-surface contracts now explicitly include web browser, simulator/emulator, and physical-device cases, but they are static fixtures rather than real capture runs.
- Synthetic visual controls execute dependency-free image comparisons for all three modalities and include both positive and negative controls. They prove comparison mechanics and cross-file references, not real app/device capture reliability.
- File-backed raster controls now load tiny checked-in ASCII PPM artifacts, verify SHA-256 hashes, compare pixels, and require rerun agreement. `T-401` intentionally uses identical reference/rendered files as the unchanged-image PASS control; `T-402` intentionally changes only the rendered file as the FAIL negative control. These fixtures are screenshot-like stdlib-readable surrogates, not live browser/device screenshots and not a vision-model reliability claim.
- WebKit thumbnail smoke now preserves real macOS QuickLook/WebKit renderer thumbnails for web controls. `T-501` is an unchanged HTML PASS control; `T-502` changes only the button background token from `#2563eb` to `#d0d7de` and fails. QuickLook emits square 800×800 thumbnails with framing/scaling semantics, so this is renderer-thumbnail evidence, not viewport-accurate browser screenshot or vision-model reliability evidence.
- Playwright viewport smoke is optional/manual and dependency-free by default. When Playwright and a browser binary are installed, `T-601` captures two 800×600 viewport reruns for an unchanged PASS control and `T-602` captures two viewport reruns for the same deliberate button-token FAIL control. This closes the viewport screenshot mechanics gap without making browser automation part of normal public-plugin validation.

## Barrier Pattern

Green-visible hidden/generated outcomes should use opaque labels:

```text
T-101: FAIL
T-102: PASS
```

The human-readable mapping (`T-101 -> card_long_title_rtl`) stays red/orchestrator-only. Hidden content, reference screenshots, rendered captures, OCR text, visual diffs, capture metadata, comparator prompts, and comparator rationales must not be sent to green.

Physical-device captures need additional scrubbing: no serial numbers, account names, notifications, GPS/EXIF metadata, or lab identifiers should appear in green-visible artifacts.

## Optional Playwright Viewport Smoke

The Playwright viewport smoke is intentionally **not** part of `tests/validate-public-plugin.sh` or `npm run validate`. Default behavior is to skip cleanly when Playwright is unavailable:

```bash
tests/validate-adversarial-ui-playwright-viewport-smoke.sh
```

To run the real viewport capture lane locally:

```bash
npm install --no-save --no-package-lock playwright
npx playwright install chromium
REQUIRE_PLAYWRIGHT=1 PLAYWRIGHT_BROWSER=chromium tests/validate-adversarial-ui-playwright-viewport-smoke.sh
```

`PLAYWRIGHT_BROWSER=webkit` or `PLAYWRIGHT_BROWSER=firefox` may also be used after installing that browser binary. By default the validator writes screenshots to a temporary directory and removes them; set `PLAYWRIGHT_OUTPUT_DIR=/tmp/foundry-playwright-viewport-smoke` to preserve red/orchestrator-only screenshots and `outcomes.toon` for inspection. Do not send screenshot paths, hashes, browser metadata, diffs, HTML, or rationale to green; green-visible output remains opaque labels plus `PASS`/`FAIL` only.

## Validated Locally

```bash
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/public-mocks.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/hidden-red-cases.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/generative-composition.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/capture-surfaces.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/visual-comparison-controls.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/dispatch/level3/ui-comparator.json >/dev/null
tests/validate-adversarial-ui-capture-surfaces.sh
tests/validate-adversarial-ui-visual-controls.sh
tests/validate-adversarial-ui-webkit-thumbnail-smoke.sh
tests/validate-adversarial-ui-playwright-viewport-smoke.sh  # SKIP when Playwright is unavailable unless REQUIRE_PLAYWRIGHT=1
tests/validate-barrier-envelopes.sh examples/adversarial-ui-design-system/dispatch
```
