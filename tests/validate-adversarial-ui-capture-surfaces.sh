#!/usr/bin/env bash
# validate-adversarial-ui-capture-surfaces.sh — validate adversarial UI capture modality fixtures.
set -euo pipefail

TARGET="${1:-examples/adversarial-ui-design-system/fixtures/capture-surfaces.json}"

python3 - "$TARGET" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
try:
    data = json.loads(path.read_text())
except Exception as exc:
    print(f"{path}: FAIL — invalid JSON: {exc}")
    sys.exit(1)

EXPECTED_SCHEMA = "foundry.adversarial-ui.capture-surfaces.v1"
REQUIRED_SURFACES = {"web_browser", "simulator_emulator", "physical_device"}
COMMON_CAPTURE_FIELDS = {
    "coordinate_space",
    "scale_or_dpr",
    "orientation",
    "locale",
    "color_scheme",
    "font_policy",
}
MODE_FIELDS = {
    "web_browser": {"browser_engine", "browser_version_policy", "viewport_px", "zoom"},
    "simulator_emulator": {"platform", "os_version_policy", "device_profile", "screen_size_px", "safe_area_insets_policy", "status_nav_bar_policy"},
    "physical_device": {"device_class_or_model_policy", "os_version_policy", "capture_tool", "screen_size_px", "safe_area_insets_policy", "status_nav_bar_policy"},
}
FORBIDDEN_GREEN_TERMS = [
    "reference screenshot",
    "rendered screenshot",
    "visual diff",
    "OCR text",
    "comparator rationale",
    "device serial",
    "account name",
    "GPS/EXIF",
]
RED_ONLY_VISIBILITIES = {
    "red_orchestrator_only",
    "red_orchestrator_comparator_only",
    "red_orchestrator_comparator_only_for_hidden_cases",
}

def fail(message):
    print(f"{path}: FAIL — {message}")
    sys.exit(1)

if data.get("schema_version") != EXPECTED_SCHEMA:
    fail(f"schema_version must be {EXPECTED_SCHEMA!r}")

summary = data.get("green_visible_summary")
if not isinstance(summary, dict):
    fail("green_visible_summary must be an object")
allowed_text = "\n".join(str(item) for item in summary.get("allowed", []))
for term in FORBIDDEN_GREEN_TERMS:
    if term.lower() in allowed_text.lower():
        fail(f"green_visible_summary.allowed leaks forbidden term {term!r}")
for required_forbidden in ["hidden content", "reference screenshots", "visual diffs", "OCR text", "comparator rationales"]:
    if required_forbidden not in "\n".join(str(item) for item in summary.get("forbidden", [])):
        fail(f"green_visible_summary.forbidden must mention {required_forbidden!r}")

common = data.get("common_contract")
if not isinstance(common, dict):
    fail("common_contract must be an object")
artifact_visibility = common.get("artifact_visibility")
if not isinstance(artifact_visibility, dict):
    fail("common_contract.artifact_visibility must be an object")
for artifact_kind in ["hidden_reference", "rendered_capture", "visual_diff", "measurement_snapshot"]:
    visibility = artifact_visibility.get(artifact_kind)
    if visibility not in RED_ONLY_VISIBILITIES:
        fail(f"{artifact_kind} must be red/orchestrator-only, got {visibility!r}")
if artifact_visibility.get("green_outcome") != "opaque_label_plus_PASS_or_FAIL_only":
    fail("green_outcome must be opaque_label_plus_PASS_or_FAIL_only")

surfaces = data.get("surfaces")
if not isinstance(surfaces, list) or not surfaces:
    fail("surfaces must be a non-empty list")

seen = set()
for index, surface in enumerate(surfaces):
    if not isinstance(surface, dict):
        fail(f"surfaces[{index}] must be an object")
    surface_class = surface.get("surface_class")
    if surface_class not in REQUIRED_SURFACES:
        fail(f"surfaces[{index}].surface_class must be one of {sorted(REQUIRED_SURFACES)}, got {surface_class!r}")
    seen.add(surface_class)
    if surface.get("visibility") != "public_contract":
        fail(f"{surface_class} visibility must be public_contract")

    capture = surface.get("capture_contract")
    if not isinstance(capture, dict):
        fail(f"{surface_class}.capture_contract must be an object")
    missing_common = sorted(COMMON_CAPTURE_FIELDS - set(capture))
    if missing_common:
        fail(f"{surface_class}.capture_contract missing common fields: {', '.join(missing_common)}")
    missing_mode = sorted(MODE_FIELDS[surface_class] - set(capture))
    if missing_mode:
        fail(f"{surface_class}.capture_contract missing mode fields: {', '.join(missing_mode)}")

    artifacts = surface.get("artifact_refs")
    if not isinstance(artifacts, list) or not artifacts:
        fail(f"{surface_class}.artifact_refs must be a non-empty list")
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            fail(f"{surface_class}.artifact_refs entries must be objects")
        if not artifact.get("kind") or not artifact.get("sha256") or not artifact.get("visibility"):
            fail(f"{surface_class}.artifact_refs entries require kind, visibility, sha256")
        if artifact.get("kind") in {"hidden_reference", "rendered_capture", "visual_diff", "measurement_snapshot"}:
            visibility = artifact.get("visibility")
            if visibility not in RED_ONLY_VISIBILITIES:
                fail(f"{surface_class} {artifact.get('kind')} visibility must be red/orchestrator-only, got {visibility!r}")

    privacy = surface.get("privacy_scrub")
    if not isinstance(privacy, dict) or privacy.get("required") is not True:
        fail(f"{surface_class}.privacy_scrub.required must be true")
    checks_text = "\n".join(str(item) for item in privacy.get("checks", []))
    if surface_class == "physical_device":
        for required in ["serial", "notifications", "GPS/EXIF"]:
            if required.lower() not in checks_text.lower():
                fail(f"physical_device privacy checks must mention {required!r}")

    for notes_field in ["stability_notes", "barrier_notes"]:
        notes = surface.get(notes_field)
        if not isinstance(notes, list) or not notes:
            fail(f"{surface_class}.{notes_field} must be a non-empty list")

missing_surfaces = sorted(REQUIRED_SURFACES - seen)
if missing_surfaces:
    fail(f"missing required surface_class entries: {', '.join(missing_surfaces)}")

print(f"{path}: PASS ({len(surfaces)} capture surfaces)")
PY
