---
title: Add optional Playwright viewport screenshot smoke for adversarial UI
origin: 2026-06-14 follow-up after WebKit/QuickLook thumbnail smoke
priority: future
status: planned
updated: 2026-06-14
---

# Add Optional Playwright Viewport Screenshot Smoke

## Goal

Add an optional, manually enabled Playwright-based smoke for viewport-accurate browser screenshots in `examples/adversarial-ui-design-system/`, closing the gap left by WebKit/QuickLook thumbnail evidence without making Playwright a required dependency for normal public-plugin validation.

## Why

The current UI spike can validate:

- design-system fixture shape,
- capture-surface contracts,
- synthetic image comparisons,
- file-backed raster controls,
- real WebKit/QuickLook renderer thumbnails.

It still does not validate DOM viewport screenshots. QuickLook emits square 800×800 thumbnails with framing/scaling semantics, so it is useful renderer evidence but not a substitute for Playwright/Chromium/WebKit/Firefox viewport captures.

## Scope

- Add a Playwright smoke that runs only when Playwright is available or explicitly requested.
- Keep it out of `tests/validate-public-plugin.sh` and `npm run validate` unless a future CI lane intentionally installs Playwright.
- Document setup in `examples/adversarial-ui-design-system/README.md`, including how to install Playwright/browser binaries.
- Reuse the existing adversarial UI fixture controls where possible.
- Capture at least two viewport screenshot reruns for:
  - an unchanged PASS control,
  - a deliberate FAIL negative control.
- Validate viewport dimensions, artifact hashes, rerun agreement, expected PASS/FAIL outcomes, and opaque-only green-visible results.
- Preserve UI information-barrier rules: screenshot paths, screenshot hashes, hidden HTML, diffs, renderer metadata, and comparator rationale stay red/orchestrator/comparator-only.

## Non-goals

- Do not make Playwright a required package dependency for installing the Foundry public plugin.
- Do not run Playwright in the fast aggregate validator by default.
- Do not add private engine changes.
- Do not expose hidden screenshot artifacts or rationale to green.
- Do not claim vision-model reliability unless a separate vision comparator is added and calibrated.

## Suggested implementation

- Add `tests/validate-adversarial-ui-playwright-viewport-smoke.sh`.
- Script behavior:
  - default: if Playwright is unavailable, print a clear `SKIP` with setup instructions and exit 0;
  - `REQUIRE_PLAYWRIGHT=1`: fail if unavailable;
  - when available, run the viewport screenshot smoke and validate artifacts.
- Prefer a self-contained example-local setup, such as `examples/adversarial-ui-design-system/package.json`, only if needed.
- Consider supporting `PLAYWRIGHT_BROWSER=chromium|webkit|firefox`, with Chromium as the first documented lane if installed.

## Acceptance

- [ ] README documents optional Playwright setup and run commands.
- [ ] Validator skips cleanly when Playwright is absent unless `REQUIRE_PLAYWRIGHT=1` is set.
- [ ] Validator runs real viewport screenshot capture when Playwright is available.
- [ ] At least one PASS and one FAIL control are captured with two reruns each.
- [ ] Validator checks viewport dimensions, artifact hashes, rerun agreement, and expected PASS/FAIL outcomes.
- [ ] Green-visible outcome artifact contains only opaque labels and PASS/FAIL values.
- [ ] Fast aggregate validation remains dependency-free and passing.
