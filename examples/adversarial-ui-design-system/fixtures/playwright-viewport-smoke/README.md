# Playwright Viewport Smoke Fixtures

Optional/manual viewport screenshot smoke for the adversarial UI design-system spike.

This directory contains red/orchestrator/comparator-only HTML fixtures plus `manifest.json` for `tests/validate-adversarial-ui-playwright-viewport-smoke.sh`. The lane is intentionally excluded from `tests/validate-public-plugin.sh` and `npm run validate` because Playwright and browser binaries are optional dependencies.

## Cases

- `T-601` — unchanged reference/rendered HTML, expected `PASS`.
- `T-602` — rendered HTML changes only the primary button background from `#2563eb` to `#d0d7de`, expected `FAIL`.

The validator captures two 800×600 viewport screenshot reruns per role when Playwright is installed and checks:

- PNG viewport dimensions,
- SHA-256 hashes,
- rerun agreement,
- expected PASS/FAIL outcome,
- green-visible outcome redaction.

## Run

Default behavior skips cleanly when Playwright is unavailable:

```bash
tests/validate-adversarial-ui-playwright-viewport-smoke.sh
```

Install and require the real viewport lane:

```bash
npm install --no-save --no-package-lock playwright
npx playwright install chromium
REQUIRE_PLAYWRIGHT=1 PLAYWRIGHT_BROWSER=chromium tests/validate-adversarial-ui-playwright-viewport-smoke.sh
```

Optional browser choices after installing their binaries:

```bash
PLAYWRIGHT_BROWSER=webkit tests/validate-adversarial-ui-playwright-viewport-smoke.sh
PLAYWRIGHT_BROWSER=firefox tests/validate-adversarial-ui-playwright-viewport-smoke.sh
```

By default screenshots are written to a temporary directory and removed. To inspect artifacts locally:

```bash
PLAYWRIGHT_OUTPUT_DIR=examples/adversarial-ui-design-system/artifacts/playwright-viewport-smoke \
  REQUIRE_PLAYWRIGHT=1 \
  tests/validate-adversarial-ui-playwright-viewport-smoke.sh
```

That output directory is git-ignored.

## Barrier Rules

These files and generated screenshots are not green-visible for hidden cases. Green may receive only opaque outcome labels, for example:

```text
T-601: PASS
T-602: FAIL
```

Do not send green screenshot paths, screenshot hashes, hidden HTML, browser metadata, renderer metadata, diffs, failure reasons, or comparator rationale.

## Scope Notes

This smoke proves Playwright viewport capture mechanics for controlled fixtures. It is not a calibrated visual-regression threshold, a vision-model reliability claim, a native simulator/device capture, or a replacement for Foundry's information barrier checks.
