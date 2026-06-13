#!/usr/bin/env bash
# validate-adversarial-ui-visual-controls.sh — validate synthetic and file-backed visual-comparison controls.
set -euo pipefail

CAPTURE_SURFACES="${1:-examples/adversarial-ui-design-system/fixtures/capture-surfaces.json}"
VISUAL_CONTROLS="${2:-examples/adversarial-ui-design-system/fixtures/visual-comparison-controls.json}"

python3 - "$CAPTURE_SURFACES" "$VISUAL_CONTROLS" <<'PY'
import hashlib
import json
import re
import sys
from pathlib import Path

capture_path = Path(sys.argv[1])
controls_path = Path(sys.argv[2])
controls_dir = controls_path.parent

EXPECTED_CAPTURE_SCHEMA = "foundry.adversarial-ui.capture-surfaces.v1"
EXPECTED_CONTROLS_SCHEMA = "foundry.adversarial-ui.visual-comparison-controls.v1"
REQUIRED_SURFACE_CLASSES = {"web_browser", "simulator_emulator", "physical_device"}
OPAQUE_ID_RE = re.compile(r"^T-\d{3,}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
ALLOWED_CAPTURE_KINDS = {"public_reference", "hidden_reference", "rendered_capture", "visual_diff", "measurement_snapshot"}
ALLOWED_ARTIFACT_FORMATS = {"portable_pixmap_p3"}
RED_ONLY_VISIBILITIES = {
    "red_orchestrator_only",
    "red_orchestrator_comparator_only",
    "red_orchestrator_comparator_only_for_hidden_cases",
}


def fail(message):
    print(f"{controls_path}: FAIL — {message}")
    sys.exit(1)


def load_json(path):
    try:
        return json.loads(path.read_text())
    except Exception as exc:
        fail(f"invalid JSON in {path}: {exc}")


def parse_hex_color(value):
    if not isinstance(value, str) or not re.match(r"^#[0-9a-fA-F]{6}$", value):
        fail(f"invalid color {value!r}; expected #RRGGBB")
    return tuple(int(value[i:i + 2], 16) for i in (1, 3, 5)) + (255,)


capture_data = load_json(capture_path)
controls = load_json(controls_path)

if capture_data.get("schema_version") != EXPECTED_CAPTURE_SCHEMA:
    fail(f"capture surfaces schema_version must be {EXPECTED_CAPTURE_SCHEMA!r}")
if controls.get("schema_version") != EXPECTED_CONTROLS_SCHEMA:
    fail(f"visual controls schema_version must be {EXPECTED_CONTROLS_SCHEMA!r}")
if controls.get("visibility") not in RED_ONLY_VISIBILITIES:
    fail("visual controls manifest must be red/orchestrator/comparator-only")

surface_by_id = {}
for surface in capture_data.get("surfaces", []):
    surface_id = surface.get("id")
    surface_class = surface.get("surface_class")
    if not surface_id or not surface_class:
        fail("capture surfaces entries require id and surface_class")
    if surface_id in surface_by_id:
        fail(f"duplicate capture surface id {surface_id!r}")
    surface_by_id[surface_id] = surface

contract = controls.get("green_visible_results_contract")
if not isinstance(contract, dict):
    fail("green_visible_results_contract must be an object")
if contract.get("format") != "opaque_label_plus_PASS_or_FAIL_only":
    fail("green_visible_results_contract.format must be opaque_label_plus_PASS_or_FAIL_only")
allowed_examples = contract.get("allowed_examples", [])
if not isinstance(allowed_examples, list) or not allowed_examples:
    fail("green_visible_results_contract.allowed_examples must be non-empty")
for example in allowed_examples:
    if not re.match(r"^T-\d{3,}:\s*(PASS|FAIL)$", str(example)):
        fail(f"green-visible example must be opaque PASS/FAIL-only, got {example!r}")
for forbidden in [
    "reference image",
    "rendered image",
    "pixel diff",
    "failure reason",
    "comparator rationale",
    "screenshot artifact path",
    "screenshot hash",
    "vision prompt",
    "vision model rationale",
]:
    if forbidden not in "\n".join(str(item) for item in contract.get("forbidden", [])):
        fail(f"green_visible_results_contract.forbidden must mention {forbidden!r}")

comparison_contract = controls.get("comparison_contract")
if not isinstance(comparison_contract, dict):
    fail("comparison_contract must be an object")
size = comparison_contract.get("image_size_px")
if not isinstance(size, dict) or not isinstance(size.get("width"), int) or not isinstance(size.get("height"), int):
    fail("comparison_contract.image_size_px requires integer width and height")
width, height = size["width"], size["height"]
reruns = comparison_contract.get("reruns_per_case")
if not isinstance(reruns, int) or reruns < 2:
    fail("comparison_contract.reruns_per_case must be an integer >= 2")
threshold = (((comparison_contract.get("thresholds") or {}).get("max_changed_pixels_for_pass")))
if not isinstance(threshold, int) or threshold < 0:
    fail("comparison_contract.thresholds.max_changed_pixels_for_pass must be a non-negative integer")

cases = controls.get("cases")
if not isinstance(cases, list) or not cases:
    fail("cases must be a non-empty list")

seen_ids = set()
seen_surface_ids = set()
seen_surface_classes = set()
seen_outcomes = set()
results = []
file_results = []


def validate_common_case(case, index, group):
    if not isinstance(case, dict):
        fail(f"{group}[{index}] must be an object")
    case_id = case.get("id")
    if not isinstance(case_id, str) or not OPAQUE_ID_RE.match(case_id):
        fail(f"{group}[{index}] id must be opaque T-### label, got {case_id!r}")
    if case_id in seen_ids:
        fail(f"duplicate case id {case_id!r}")
    seen_ids.add(case_id)

    surface_id = case.get("surface_id")
    if surface_id not in surface_by_id:
        fail(f"case {case_id} references unknown surface_id {surface_id!r}")
    surface = surface_by_id[surface_id]
    surface_class = case.get("surface_class")
    if surface_class != surface.get("surface_class"):
        fail(f"case {case_id} surface_class {surface_class!r} does not match {surface_id} ({surface.get('surface_class')!r})")
    seen_surface_ids.add(surface_id)
    seen_surface_classes.add(surface_class)

    capture_kind = case.get("capture_kind")
    if capture_kind not in ALLOWED_CAPTURE_KINDS:
        fail(f"case {case_id} has invalid capture_kind {capture_kind!r}")
    visibility = ((capture_data.get("common_contract") or {}).get("artifact_visibility") or {}).get(capture_kind)
    if capture_kind != "public_reference" and visibility not in RED_ONLY_VISIBILITIES:
        fail(f"case {case_id} capture_kind {capture_kind!r} is not red/orchestrator-only in capture-surfaces contract")

    expected = case.get("expected_outcome")
    if expected not in {"PASS", "FAIL"}:
        fail(f"case {case_id} expected_outcome must be PASS or FAIL")
    seen_outcomes.add(expected)

    notes = case.get("barrier_notes")
    if not isinstance(notes, list) or not notes:
        fail(f"case {case_id} must include barrier_notes")

    return case_id, surface_id, surface_class, capture_kind, expected


def render_scene(scene):
    if not isinstance(scene, dict):
        fail("scene must be an object")
    background = parse_hex_color(scene.get("background", "#ffffff"))
    image = [[background for _ in range(width)] for _ in range(height)]
    rectangles = scene.get("rectangles", [])
    if not isinstance(rectangles, list):
        fail("scene.rectangles must be a list")
    for rect in rectangles:
        if not isinstance(rect, dict):
            fail("rectangle entries must be objects")
        xy = rect.get("xy")
        if not (isinstance(xy, list) and len(xy) == 4 and all(isinstance(v, int) for v in xy)):
            fail(f"rectangle xy must be four integers, got {xy!r}")
        x1, y1, x2, y2 = xy
        if x1 < 0 or y1 < 0 or x2 > width or y2 > height or x1 >= x2 or y1 >= y2:
            fail(f"rectangle xy out of bounds: {xy!r}")
        fill = parse_hex_color(rect.get("fill", "#ffffff"))
        outline = parse_hex_color(rect.get("outline", rect.get("fill", "#ffffff")))
        # Coordinates are half-open [x1, y1, x2, y2), matching common screenshot crop conventions.
        for y in range(y1, y2):
            for x in range(x1, x2):
                image[y][x] = outline if y in (y1, y2 - 1) or x in (x1, x2 - 1) else fill
    return image


def changed_pixels(reference, rendered):
    return sum(
        1
        for y in range(height)
        for x in range(width)
        if reference[y][x] != rendered[y][x]
    )


def strip_ppm_comments(text):
    lines = []
    for line in text.splitlines():
        lines.append(line.split("#", 1)[0])
    return "\n".join(lines)


def parse_ppm_p3(path, data):
    try:
        text = data.decode("ascii")
    except UnicodeDecodeError as exc:
        fail(f"{path} is not ASCII PPM P3: {exc}")
    tokens = strip_ppm_comments(text).split()
    if len(tokens) < 4 or tokens[0] != "P3":
        fail(f"{path} must be an ASCII PPM P3 file")
    try:
        image_width = int(tokens[1])
        image_height = int(tokens[2])
        max_value = int(tokens[3])
        values = [int(token) for token in tokens[4:]]
    except ValueError as exc:
        fail(f"{path} contains non-integer PPM fields: {exc}")
    if image_width != width or image_height != height:
        fail(f"{path} dimensions {image_width}x{image_height} do not match comparison_contract {width}x{height}")
    if max_value != 255:
        fail(f"{path} max channel value must be 255")
    expected_values = width * height * 3
    if len(values) != expected_values:
        fail(f"{path} has {len(values)} channel values, expected {expected_values}")
    if any(value < 0 or value > 255 for value in values):
        fail(f"{path} has channel values outside 0..255")
    pixels = []
    iterator = iter(values)
    for _ in range(height):
        row = []
        for _ in range(width):
            row.append((next(iterator), next(iterator), next(iterator), 255))
        pixels.append(row)
    return pixels


def load_artifact(artifact, case_id, role):
    if not isinstance(artifact, dict):
        fail(f"case {case_id} {role}_artifact must be an object")
    artifact_format = artifact.get("format")
    if artifact_format not in ALLOWED_ARTIFACT_FORMATS:
        fail(f"case {case_id} {role}_artifact format must be one of {sorted(ALLOWED_ARTIFACT_FORMATS)}, got {artifact_format!r}")
    visibility = artifact.get("visibility")
    if visibility not in RED_ONLY_VISIBILITIES:
        fail(f"case {case_id} {role}_artifact visibility must be red/orchestrator-only, got {visibility!r}")
    expected_sha = artifact.get("sha256")
    if not isinstance(expected_sha, str) or not SHA256_RE.match(expected_sha):
        fail(f"case {case_id} {role}_artifact sha256 must be 64 lowercase hex characters")
    raw_path = artifact.get("path")
    if not isinstance(raw_path, str) or not raw_path:
        fail(f"case {case_id} {role}_artifact path must be a non-empty relative path")
    rel_path = Path(raw_path)
    if rel_path.is_absolute() or ".." in rel_path.parts:
        fail(f"case {case_id} {role}_artifact path must stay under the visual-controls fixture directory")
    full_path = controls_dir / rel_path
    if not full_path.is_file():
        fail(f"case {case_id} {role}_artifact missing file {full_path}")
    data = full_path.read_bytes()
    actual_sha = hashlib.sha256(data).hexdigest()
    if actual_sha != expected_sha:
        fail(f"case {case_id} {role}_artifact SHA-256 mismatch for {raw_path}: expected {expected_sha}, got {actual_sha}")
    return parse_ppm_p3(full_path, data)


for index, case in enumerate(cases):
    case_id, surface_id, surface_class, _, expected = validate_common_case(case, index, "cases")
    observed = []
    deltas = []
    for _ in range(reruns):
        ref = render_scene(case.get("reference_scene"))
        got = render_scene(case.get("rendered_scene"))
        delta = changed_pixels(ref, got)
        outcome = "PASS" if delta <= threshold else "FAIL"
        observed.append(outcome)
        deltas.append(delta)
    if len(set(observed)) != 1:
        fail(f"case {case_id} rerun disagreement: {observed}")
    if observed[0] != expected:
        fail(f"case {case_id} expected {expected} but observed {observed[0]} with changed_pixels={deltas[0]}")
    results.append((case_id, surface_id, surface_class, expected, deltas[0]))

file_controls = controls.get("file_backed_controls")
if not isinstance(file_controls, dict):
    fail("file_backed_controls must be an object")
if file_controls.get("source_format") != "portable_pixmap_p3":
    fail("file_backed_controls.source_format must be portable_pixmap_p3")
if "not live" not in str(file_controls.get("purpose", "")).lower():
    fail("file_backed_controls.purpose must state these are not live captures")
if "stdlib" not in str(file_controls.get("format_note", "")).lower():
    fail("file_backed_controls.format_note must document stdlib-only parsing")
if file_controls.get("artifact_visibility") not in RED_ONLY_VISIBILITIES:
    fail("file_backed_controls.artifact_visibility must be red/orchestrator-only")
file_cases = file_controls.get("cases")
if not isinstance(file_cases, list) or not file_cases:
    fail("file_backed_controls.cases must be a non-empty list")
file_outcomes = set()
for index, case in enumerate(file_cases):
    case_id, surface_id, surface_class, _, expected = validate_common_case(case, index, "file_backed_controls.cases")
    if case.get("comparator") != "pixel_exact":
        fail(f"case {case_id} comparator must be pixel_exact")
    privacy = case.get("privacy_scrub")
    if not isinstance(privacy, list) or not privacy:
        fail(f"case {case_id} must include privacy_scrub checks")
    observed = []
    deltas = []
    for _ in range(reruns):
        ref = load_artifact(case.get("reference_artifact"), case_id, "reference")
        got = load_artifact(case.get("rendered_artifact"), case_id, "rendered")
        delta = changed_pixels(ref, got)
        outcome = "PASS" if delta <= threshold else "FAIL"
        observed.append(outcome)
        deltas.append(delta)
    if len(set(observed)) != 1:
        fail(f"case {case_id} file-backed rerun disagreement: {observed}")
    if observed[0] != expected:
        fail(f"case {case_id} expected {expected} but observed {observed[0]} with changed_pixels={deltas[0]}")
    file_outcomes.add(expected)
    file_results.append((case_id, surface_id, surface_class, expected, deltas[0]))

if file_outcomes != {"PASS", "FAIL"}:
    fail("file-backed controls must include both PASS and FAIL cases")

missing_classes = sorted(REQUIRED_SURFACE_CLASSES - seen_surface_classes)
if missing_classes:
    fail(f"visual controls missing required surface classes: {', '.join(missing_classes)}")
missing_surface_ids = sorted(set(surface_by_id) - seen_surface_ids)
if missing_surface_ids:
    fail(f"visual controls do not reference every capture surface id: {', '.join(missing_surface_ids)}")
if seen_outcomes != {"PASS", "FAIL"}:
    fail("visual controls must include both PASS and FAIL cases")

summary = ", ".join(f"{case_id}:{outcome}:{surface_class}:Δ{delta}" for case_id, _, surface_class, outcome, delta in results)
file_summary = ", ".join(f"{case_id}:{outcome}:{surface_class}:Δ{delta}" for case_id, _, surface_class, outcome, delta in file_results)
print(f"{controls_path}: PASS ({len(results)} synthetic controls; {summary}; {len(file_results)} file-backed controls; {file_summary})")
PY
