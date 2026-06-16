#!/usr/bin/env bash
# validate-adversarial-ui-playwright-viewport-smoke.sh — optional/manual Playwright viewport screenshot smoke.
set -euo pipefail

MANIFEST="${1:-examples/adversarial-ui-design-system/fixtures/playwright-viewport-smoke/manifest.json}"
REQUIRE_PLAYWRIGHT="${REQUIRE_PLAYWRIGHT:-0}"
PLAYWRIGHT_BROWSER="${PLAYWRIGHT_BROWSER:-chromium}"
OUTPUT_DIR="${PLAYWRIGHT_OUTPUT_DIR:-}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

skip_or_fail() {
  local message="$1"
  if [[ "$REQUIRE_PLAYWRIGHT" == "1" || "$REQUIRE_PLAYWRIGHT" == "true" || "$REQUIRE_PLAYWRIGHT" == "TRUE" ]]; then
    printf '%s: FAIL — %s\n' "$MANIFEST" "$message" >&2
    exit 1
  fi
  printf '%s: SKIP — %s\n' "$MANIFEST" "$message"
  printf 'Install with: npm install --no-save --no-package-lock playwright && npx playwright install %s\n' "$PLAYWRIGHT_BROWSER"
  exit 0
}

python3 - "$MANIFEST" <<'PY'
import hashlib
import json
import re
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
fixture_dir = manifest_path.parent

EXPECTED_SCHEMA = "foundry.adversarial-ui.playwright-viewport-smoke.v1"
RED_ONLY_VISIBILITIES = {"red_orchestrator_only", "red_orchestrator_comparator_only", "red_orchestrator_comparator_only_for_hidden_cases"}
OPAQUE_IDS = {"T-601", "T-602"}


def fail(message):
    print(f"{manifest_path}: FAIL — {message}", file=sys.stderr)
    sys.exit(1)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_json(path):
    try:
        return json.loads(path.read_text())
    except Exception as exc:
        fail(f"invalid JSON: {exc}")


def require_relative_path(raw_path, base, label):
    if not isinstance(raw_path, str) or not raw_path:
        fail(f"{label} path must be a non-empty relative path")
    rel = Path(raw_path)
    if rel.is_absolute() or ".." in rel.parts:
        fail(f"{label} path must stay under its fixture directory")
    return base / rel


data = load_json(manifest_path)
if data.get("schema_version") != EXPECTED_SCHEMA:
    fail(f"schema_version must be {EXPECTED_SCHEMA!r}")
if data.get("visibility") not in RED_ONLY_VISIBILITIES:
    fail("manifest visibility must be red/orchestrator/comparator-only")
if data.get("surface_id") != "surface-web-001" or data.get("surface_class") != "web_browser":
    fail("this smoke must target the surface-web-001 web_browser capture surface")
if data.get("capture_kind") != "rendered_capture":
    fail("capture_kind must be rendered_capture")

runner = data.get("runner")
if not isinstance(runner, dict):
    fail("runner must be an object")
if str(runner.get("tool", "")).lower() != "playwright":
    fail("runner.tool must be playwright")
viewport = runner.get("viewport_px")
if not isinstance(viewport, dict) or viewport.get("width") != 800 or viewport.get("height") != 600:
    fail("runner.viewport_px must be 800x600 for this smoke")
if runner.get("reruns_per_role") != 2:
    fail("runner.reruns_per_role must be 2")
framing_note = str(runner.get("framing_note", "")).lower()
if "viewport" not in framing_note or "quicklook" not in framing_note or "thumbnail" not in framing_note:
    fail("runner.framing_note must distinguish Playwright viewports from QuickLook thumbnails")

contract = data.get("green_visible_results_contract")
if not isinstance(contract, dict) or contract.get("format") != "opaque_label_plus_PASS_or_FAIL_only":
    fail("green_visible_results_contract.format must be opaque_label_plus_PASS_or_FAIL_only")
for example in contract.get("allowed_examples", []):
    if not isinstance(example, str) or not re.match(r"^T-\d{3,}:\s*(PASS|FAIL)$", example):
        fail(f"green-visible example must be opaque PASS/FAIL-only, got {example!r}")
for forbidden in ["HTML hidden-case content", "screenshot artifact path", "screenshot hash", "browser metadata", "renderer metadata", "pixel diff", "failure reason", "comparator rationale"]:
    if forbidden not in "\n".join(str(item) for item in contract.get("forbidden", [])):
        fail(f"green_visible_results_contract.forbidden must mention {forbidden!r}")

thresholds = data.get("thresholds")
if not isinstance(thresholds, dict):
    fail("thresholds must be an object")
if thresholds.get("rerun_hash_agreement_required") is not True:
    fail("thresholds.rerun_hash_agreement_required must be true")
if thresholds.get("expected_viewport_width_px") != viewport["width"] or thresholds.get("expected_viewport_height_px") != viewport["height"]:
    fail("threshold expected viewport dimensions must match runner.viewport_px")
if "not_calibrated" not in str(thresholds.get("pixel_delta_threshold", "")):
    fail("pixel_delta_threshold must remain explicitly not calibrated for general visual regression")

source_hashes = {}
for source in data.get("source_html", []):
    path = require_relative_path(source.get("path"), fixture_dir, "source_html")
    if not path.is_file():
        fail(f"source HTML missing: {path}")
    expected = source.get("sha256")
    actual = sha256(path)
    if expected != actual:
        fail(f"source HTML hash mismatch for {path}: expected {expected}, got {actual}")
    if source.get("visibility") not in RED_ONLY_VISIBILITIES:
        fail(f"source HTML {path.name} must be red/orchestrator/comparator-only")
    source_hashes[path.name] = actual

seen = set()
for case in data.get("cases", []):
    case_id = case.get("id")
    if case_id not in OPAQUE_IDS:
        fail(f"unexpected case id {case_id!r}")
    if case_id in seen:
        fail(f"duplicate case id {case_id}")
    seen.add(case_id)
    if case.get("expected_outcome") not in {"PASS", "FAIL"}:
        fail(f"case {case_id} expected_outcome must be PASS or FAIL")
    ref = require_relative_path(case.get("reference_html"), fixture_dir, f"{case_id} reference_html")
    rendered = require_relative_path(case.get("rendered_html"), fixture_dir, f"{case_id} rendered_html")
    if not ref.is_file() or not rendered.is_file():
        fail(f"case {case_id} HTML fixtures must exist")
    if case_id == "T-601" and ref.read_text() != rendered.read_text():
        fail("T-601 must be an unchanged HTML PASS control")
    if case_id == "T-602":
        expected_rendered = ref.read_text().replace("background: #2563eb;", "background: #d0d7de;", 1)
        if rendered.read_text() != expected_rendered:
            fail("T-602 rendered HTML must differ only by changing the button background token #2563eb -> #d0d7de")
    notes = case.get("barrier_notes")
    if not isinstance(notes, list) or not notes:
        fail(f"case {case_id} must include barrier_notes")

if seen != OPAQUE_IDS:
    fail(f"expected cases {sorted(OPAQUE_IDS)}, got {sorted(seen)}")
outcomes = data.get("green_visible_outcomes")
if not isinstance(outcomes, dict) or outcomes.get("visibility") != "green_visible_opaque_pass_fail_only":
    fail("green_visible_outcomes.visibility must be green_visible_opaque_pass_fail_only")
PY

if ! command -v node >/dev/null 2>&1; then
  skip_or_fail "node is unavailable"
fi

if ! node -e "require('playwright')" >/dev/null 2>&1; then
  skip_or_fail "Playwright node module is unavailable"
fi

if [[ "$PLAYWRIGHT_BROWSER" != "chromium" && "$PLAYWRIGHT_BROWSER" != "webkit" && "$PLAYWRIGHT_BROWSER" != "firefox" ]]; then
  printf '%s: FAIL — PLAYWRIGHT_BROWSER must be chromium, webkit, or firefox\n' "$MANIFEST" >&2
  exit 1
fi

if [[ -z "$OUTPUT_DIR" ]]; then
  OUTPUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/foundry-playwright-viewport-smoke.XXXXXX")"
  CLEAN_OUTPUT_DIR=1
else
  mkdir -p "$OUTPUT_DIR"
  CLEAN_OUTPUT_DIR=0
fi
trap 'if [[ "${CLEAN_OUTPUT_DIR:-0}" == "1" ]]; then rm -rf "$OUTPUT_DIR"; fi' EXIT

set +e
MANIFEST_PATH="$MANIFEST" PLAYWRIGHT_BROWSER="$PLAYWRIGHT_BROWSER" OUTPUT_DIR="$OUTPUT_DIR" node <<'NODE'
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const { pathToFileURL } = require('url');
const playwright = require('playwright');

const manifestPath = path.resolve(process.env.MANIFEST_PATH);
const browserName = process.env.PLAYWRIGHT_BROWSER || 'chromium';
const outputDir = path.resolve(process.env.OUTPUT_DIR);
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const fixtureDir = path.dirname(manifestPath);
const browserType = playwright[browserName];
const requirePlaywright = ['1', 'true', 'TRUE'].includes(process.env.REQUIRE_PLAYWRIGHT || '0');

function fail(message) {
  console.error(`${manifestPath}: FAIL — ${message}`);
  process.exit(1);
}

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function pngDimensions(filePath) {
  const data = fs.readFileSync(filePath);
  if (data.length < 24 || data.slice(0, 8).toString('binary') !== '\x89PNG\r\n\x1a\n' || data.slice(12, 16).toString('ascii') !== 'IHDR') {
    fail(`${filePath} is not a PNG with an IHDR header`);
  }
  return { width: data.readUInt32BE(16), height: data.readUInt32BE(20) };
}

function relativePath(rawPath, base, label) {
  if (typeof rawPath !== 'string' || rawPath.length === 0) {
    fail(`${label} path must be a non-empty relative path`);
  }
  if (path.isAbsolute(rawPath) || rawPath.split(/[\\/]+/).includes('..')) {
    fail(`${label} path must stay under its fixture directory`);
  }
  return path.join(base, rawPath);
}

function ensureOpaqueOutcomes(outcomesPath, observedById) {
  const lines = Object.entries(observedById).map(([caseId, outcome]) => `${caseId},${outcome}`);
  fs.writeFileSync(outcomesPath, `${lines.join('\n')}\n`);
  const text = fs.readFileSync(outcomesPath, 'utf8');
  for (const forbidden of ['screenshot', 'sha256', 'browser', 'renderer', 'html', 'pixel', 'diff', 'path', 'Playwright', 'chromium', 'webkit', 'firefox']) {
    if (text.toLowerCase().includes(forbidden.toLowerCase())) {
      fail(`green-visible outcomes must not include forbidden detail ${forbidden}`);
    }
  }
  for (const [caseId, outcome] of Object.entries(observedById)) {
    if (!text.includes(`${caseId},${outcome}`)) {
      fail(`green-visible outcomes missing ${caseId},${outcome}`);
    }
  }
  return sha256(outcomesPath);
}

(async () => {
  fs.mkdirSync(outputDir, { recursive: true });
  const runner = manifest.runner || {};
  const viewport = runner.viewport_px || { width: 800, height: 600 };
  const deviceScaleFactor = runner.device_scale_factor || 1;
  const reruns = runner.reruns_per_role || 2;
  let browser;
  try {
    browser = await browserType.launch({ headless: true });
  } catch (error) {
    const message = `Playwright ${browserName} browser binary is unavailable or cannot launch: ${String(error.message).split('\n')[0]}`;
    if (requirePlaywright) {
      console.error(`${manifestPath}: FAIL — ${message}`);
      console.error(`Install with: npx playwright install ${browserName}`);
      process.exit(1);
    }
    console.log(`${manifestPath}: SKIP — ${message}`);
    console.log(`Install with: npx playwright install ${browserName}`);
    process.exit(42);
  }

  const observedById = {};
  const summaries = [];
  try {
    for (const testCase of manifest.cases || []) {
      const caseDir = path.join(outputDir, testCase.id);
      fs.mkdirSync(caseDir, { recursive: true });
      const hashesByRole = {};
      for (const [role, key] of [['reference', 'reference_html'], ['rendered', 'rendered_html']]) {
        const htmlPath = relativePath(testCase[key], fixtureDir, `${testCase.id} ${key}`);
        const hashes = [];
        for (let run = 1; run <= reruns; run += 1) {
          const page = await browser.newPage({ viewport, deviceScaleFactor, colorScheme: 'light', locale: 'en-US' });
          await page.goto(pathToFileURL(htmlPath).href, { waitUntil: 'load' });
          const screenshotPath = path.join(caseDir, `${testCase.id}-${role}-r${run}.png`);
          await page.screenshot({ path: screenshotPath, fullPage: false, animations: 'disabled', caret: 'hide', omitBackground: false });
          await page.close();
          const dimensions = pngDimensions(screenshotPath);
          if (dimensions.width !== viewport.width || dimensions.height !== viewport.height) {
            fail(`${screenshotPath} dimensions ${dimensions.width}x${dimensions.height} do not match viewport ${viewport.width}x${viewport.height}`);
          }
          hashes.push(sha256(screenshotPath));
        }
        if (new Set(hashes).size !== 1) {
          fail(`${testCase.id} ${role} viewport rerun disagreement: ${hashes.join(', ')}`);
        }
        hashesByRole[role] = hashes[0];
      }
      const observed = hashesByRole.reference === hashesByRole.rendered ? 'PASS' : 'FAIL';
      if (observed !== testCase.expected_outcome) {
        fail(`${testCase.id} expected ${testCase.expected_outcome} but observed ${observed}`);
      }
      observedById[testCase.id] = observed;
      summaries.push(`${testCase.id}:${observed}:${hashesByRole.reference.slice(0, 8)}->${hashesByRole.rendered.slice(0, 8)}`);
    }
  } finally {
    await browser.close();
  }

  const outcomesPath = path.join(outputDir, 'outcomes.toon');
  const outcomesSha = ensureOpaqueOutcomes(outcomesPath, observedById);
  console.log(`${manifestPath}: PASS (Playwright ${browserName} viewport ${viewport.width}x${viewport.height}; ${summaries.join(', ')}; outcomes ${outcomesSha.slice(0, 8)})`);
})().catch((error) => fail(error && error.stack ? error.stack : String(error)));
NODE
NODE_STATUS=$?
set -e

if [[ "$NODE_STATUS" == "42" ]]; then
  if [[ "$REQUIRE_PLAYWRIGHT" == "1" || "$REQUIRE_PLAYWRIGHT" == "true" || "$REQUIRE_PLAYWRIGHT" == "TRUE" ]]; then
    exit 1
  fi
  exit 0
fi
exit "$NODE_STATUS"
