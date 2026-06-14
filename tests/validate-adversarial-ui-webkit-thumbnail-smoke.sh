#!/usr/bin/env bash
# validate-adversarial-ui-webkit-thumbnail-smoke.sh — validate/rerun macOS WebKit QuickLook UI thumbnail smoke.
set -euo pipefail

MANIFEST="${1:-examples/adversarial-ui-design-system/fixtures/webkit-thumbnail-smoke/manifest.json}"
RERUN="${RERUN_WEBKIT_THUMBNAIL_SMOKE:-1}"

python3 - "$MANIFEST" "$RERUN" <<'PY'
import hashlib
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

manifest_path = Path(sys.argv[1])
rerun_enabled = sys.argv[2] not in {"0", "false", "False", "no"}
fixture_dir = manifest_path.parent
repo_root = Path.cwd()
example_root = fixture_dir.parents[1]
artifact_dir = example_root / "artifacts" / "webkit-thumbnail-smoke"

EXPECTED_SCHEMA = "foundry.adversarial-ui.webkit-thumbnail-smoke.v1"
RED_ONLY_VISIBILITIES = {"red_orchestrator_only", "red_orchestrator_comparator_only", "red_orchestrator_comparator_only_for_hidden_cases"}
OPAQUE_IDS = {"T-501", "T-502"}
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def fail(message):
    print(f"{manifest_path}: FAIL — {message}")
    sys.exit(1)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_json(path):
    try:
        return json.loads(path.read_text())
    except Exception as exc:
        fail(f"invalid JSON in {path}: {exc}")


def png_dimensions(path):
    data = path.read_bytes()
    if len(data) < 24 or data[:8] != PNG_SIGNATURE or data[12:16] != b"IHDR":
        fail(f"{path} is not a PNG with an IHDR header")
    return struct.unpack(">II", data[16:24])


def require_relative_path(raw_path, base, label):
    if not isinstance(raw_path, str) or not raw_path:
        fail(f"{label} path must be a non-empty relative path")
    rel = Path(raw_path)
    if rel.is_absolute() or ".." in rel.parts:
        fail(f"{label} path must stay under its fixture/artifact directory")
    return base / rel


def check_html_pair(case):
    ref_path = require_relative_path(case.get("reference_html"), fixture_dir, f"{case.get('id')} reference_html")
    rendered_path = require_relative_path(case.get("rendered_html"), fixture_dir, f"{case.get('id')} rendered_html")
    if not ref_path.is_file() or not rendered_path.is_file():
        fail(f"case {case.get('id')} HTML fixtures must exist")
    ref = ref_path.read_text()
    rendered = rendered_path.read_text()
    case_id = case.get("id")
    if case_id == "T-501" and ref != rendered:
        fail("T-501 must be an unchanged HTML PASS control")
    if case_id == "T-502":
        expected = ref.replace("background: #2563eb;", "background: #d0d7de;", 1)
        if rendered != expected:
            fail("T-502 rendered HTML must differ only by changing the button background token #2563eb -> #d0d7de")
    return ref_path, rendered_path


def check_artifacts(case, artifact_key, expected_width, expected_height):
    artifacts = case.get(artifact_key)
    if not isinstance(artifacts, list) or len(artifacts) < 2:
        fail(f"case {case.get('id')} {artifact_key} must include at least two rerun artifacts")
    hashes = []
    for artifact in artifacts:
        path = require_relative_path(artifact.get("path"), artifact_dir, f"{case.get('id')} {artifact_key}")
        if not path.is_file():
            fail(f"artifact missing: {path}")
        expected_sha = artifact.get("sha256")
        if not isinstance(expected_sha, str) or len(expected_sha) != 64:
            fail(f"artifact {path} must include a 64-character sha256")
        actual_sha = sha256(path)
        if actual_sha != expected_sha:
            fail(f"artifact SHA-256 mismatch for {path}: expected {expected_sha}, got {actual_sha}")
        width, height = png_dimensions(path)
        if (width, height) != (expected_width, expected_height):
            fail(f"artifact {path} dimensions {width}x{height} do not match expected {expected_width}x{expected_height}")
        hashes.append(actual_sha)
    if len(set(hashes)) != 1:
        fail(f"case {case.get('id')} {artifact_key} rerun disagreement: {hashes}")
    return hashes[0]


def rerun_quicklook(case, html_role, html_path, expected_sha):
    qlmanage = shutil.which("qlmanage")
    if not qlmanage:
        return "SKIP_NO_QLMANAGE"
    expected_size = int(((data.get("renderer") or {}).get("thumbnail_size_px") or {}).get("width", 800))
    with tempfile.TemporaryDirectory(prefix="foundry-webkit-thumbnail-") as tmp:
        tmpdir = Path(tmp)
        src = tmpdir / f"{case.get('id')}-{html_role}.html"
        src.write_text(html_path.read_text())
        result = subprocess.run([qlmanage, "-t", "-s", str(expected_size), "-o", str(tmpdir), str(src)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if result.returncode != 0:
            fail(f"qlmanage rerun failed for {case.get('id')} {html_role}")
        png = tmpdir / f"{src.name}.png"
        if not png.is_file():
            fail(f"qlmanage rerun did not produce {png.name} for {case.get('id')} {html_role}")
        actual = sha256(png)
        if actual != expected_sha:
            fail(f"qlmanage rerun hash mismatch for {case.get('id')} {html_role}: expected {expected_sha}, got {actual}")
    return "PASS"


data = load_json(manifest_path)
if data.get("schema_version") != EXPECTED_SCHEMA:
    fail(f"schema_version must be {EXPECTED_SCHEMA!r}")
if data.get("visibility") not in RED_ONLY_VISIBILITIES:
    fail("manifest visibility must be red/orchestrator/comparator-only")
if data.get("surface_id") != "surface-web-001" or data.get("surface_class") != "web_browser":
    fail("this smoke must target the surface-web-001 web_browser capture surface")
if data.get("capture_kind") != "rendered_capture":
    fail("capture_kind must be rendered_capture")

renderer = data.get("renderer")
if not isinstance(renderer, dict):
    fail("renderer must be an object")
for required in ["qlmanage", "QuickLook", "thumbnail"]:
    if required.lower() not in json.dumps(renderer).lower():
        fail(f"renderer metadata must mention {required}")
size = renderer.get("thumbnail_size_px")
if not isinstance(size, dict) or not isinstance(size.get("width"), int) or not isinstance(size.get("height"), int):
    fail("renderer.thumbnail_size_px must include integer width and height")
expected_width, expected_height = size["width"], size["height"]
if expected_width != expected_height:
    fail("QuickLook smoke expects square thumbnail dimensions; do not treat this as a viewport screenshot")
if "not" not in str(renderer.get("framing_note", "")).lower() or "viewport" not in str(renderer.get("framing_note", "")).lower():
    fail("renderer.framing_note must state thumbnails are not DOM viewport pixels")

contract = data.get("green_visible_results_contract")
if not isinstance(contract, dict) or contract.get("format") != "opaque_label_plus_PASS_or_FAIL_only":
    fail("green_visible_results_contract.format must be opaque_label_plus_PASS_or_FAIL_only")
for example in contract.get("allowed_examples", []):
    if not isinstance(example, str) or not (example.startswith("T-") and (example.endswith(": PASS") or example.endswith(": FAIL"))):
        fail(f"green-visible example must be opaque PASS/FAIL-only, got {example!r}")
for forbidden in ["HTML hidden-case content", "thumbnail path", "thumbnail hash", "renderer metadata", "pixel diff", "failure reason", "comparator rationale"]:
    if forbidden not in "\n".join(str(item) for item in contract.get("forbidden", [])):
        fail(f"green_visible_results_contract.forbidden must mention {forbidden!r}")

thresholds = data.get("thresholds")
if not isinstance(thresholds, dict):
    fail("thresholds must be an object")
if thresholds.get("rerun_hash_agreement_required") is not True:
    fail("thresholds.rerun_hash_agreement_required must be true")
if "not_calibrated" not in str(thresholds.get("pixel_delta_threshold", "")):
    fail("pixel_delta_threshold must remain explicitly not calibrated for thumbnail framing")

seen_cases = set()
observed_by_id = {}
case_summaries = []
for case in data.get("cases", []):
    if not isinstance(case, dict):
        fail("cases entries must be objects")
    case_id = case.get("id")
    if case_id not in OPAQUE_IDS:
        fail(f"unexpected case id {case_id!r}")
    if case_id in seen_cases:
        fail(f"duplicate case id {case_id}")
    seen_cases.add(case_id)
    if case.get("expected_outcome") not in {"PASS", "FAIL"}:
        fail(f"case {case_id} expected_outcome must be PASS or FAIL")
    notes = case.get("barrier_notes")
    if not isinstance(notes, list) or not notes:
        fail(f"case {case_id} must include barrier_notes")

    ref_html, rendered_html = check_html_pair(case)
    reference_hash = check_artifacts(case, "reference_artifacts", expected_width, expected_height)
    rendered_hash = check_artifacts(case, "rendered_artifacts", expected_width, expected_height)
    observed = "PASS" if reference_hash == rendered_hash else "FAIL"
    if observed != case.get("expected_outcome"):
        fail(f"case {case_id} expected {case.get('expected_outcome')} but observed {observed}")
    observed_by_id[case_id] = observed
    if rerun_enabled:
        rerun_quicklook(case, "reference", ref_html, reference_hash)
        rerun_quicklook(case, "rendered", rendered_html, rendered_hash)
    case_summaries.append(f"{case_id}:{observed}:{reference_hash[:8]}->{rendered_hash[:8]}")

if seen_cases != OPAQUE_IDS:
    fail(f"expected cases {sorted(OPAQUE_IDS)}, got {sorted(seen_cases)}")

outcomes = data.get("green_visible_outcomes")
if not isinstance(outcomes, dict):
    fail("green_visible_outcomes must be an object")
if outcomes.get("visibility") != "green_visible_opaque_pass_fail_only":
    fail("green_visible_outcomes.visibility must be green_visible_opaque_pass_fail_only")
outcomes_path = require_relative_path(outcomes.get("path"), example_root, "green_visible_outcomes")
if not outcomes_path.is_file():
    fail(f"green-visible outcomes file missing: {outcomes_path}")
expected_outcomes_sha = outcomes.get("sha256")
actual_outcomes_sha = sha256(outcomes_path)
if actual_outcomes_sha != expected_outcomes_sha:
    fail(f"green-visible outcomes SHA-256 mismatch: expected {expected_outcomes_sha}, got {actual_outcomes_sha}")
outcomes_text = outcomes_path.read_text()
for forbidden in ["thumbnail", "sha256", "renderer", "html", "pixel", "diff", "path", "QuickLook", "qlmanage"]:
    if forbidden.lower() in outcomes_text.lower():
        fail(f"green-visible outcomes must not include forbidden detail {forbidden!r}")
for case_id, observed in observed_by_id.items():
    if f"{case_id},{observed}" not in outcomes_text:
        fail(f"green-visible outcomes missing {case_id},{observed}")

rerun_note = "with qlmanage rerun" if rerun_enabled and shutil.which("qlmanage") else "committed artifact validation only"
print(f"{manifest_path}: PASS ({rerun_note}; {', '.join(case_summaries)})")
PY
