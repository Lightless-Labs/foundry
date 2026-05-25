#!/usr/bin/env bash
# validate-barrier-envelopes.sh — validate replayable Foundry PromptEnvelope artifacts.
#
# Usage:
#   tests/validate-barrier-envelopes.sh                    # run built-in self-tests
#   tests/validate-barrier-envelopes.sh runs/<id>/dispatch  # validate envelope JSON files
#   tests/validate-barrier-envelopes.sh path/to/envelope.json ...
set -euo pipefail

validate_targets() {
  python3 - "$@" <<'PY'
import json
import os
import re
import sys
from pathlib import Path

REQUIRED_FIELDS = {
    "schema_version",
    "run_id",
    "phase",
    "recipient",
    "prompt",
    "visible_context",
    "withheld_context",
}
EXPECTED_SCHEMA = "foundry.prompt-envelope.v1"

GREEN_RECIPIENT_RE = re.compile(r"(^|[-_])green([-_]|$)|green-team|green-reviewer")
RED_RECIPIENT_RE = re.compile(r"(^|[-_])red([-_]|$)|red-team|red-reviewer")

# These are coarse guardrails. Precise leak detection comes from withheld_context samples.
GREEN_FORBIDDEN_HINTS = [
    re.compile(r"\bassert(?:_eq)?!\s*\(", re.IGNORECASE),
    re.compile(r"\bexpect\s*\(", re.IGNORECASE),
    re.compile(r"expected\s+.+\s+got\s+.+", re.IGNORECASE),
    re.compile(r"stack\s+backtrace", re.IGNORECASE),
    re.compile(r"(?:^|/)step_definitions(?:/|$)", re.IGNORECASE),
    re.compile(r"(?:^|/)red(?:/|$)", re.IGNORECASE),
]

RED_FORBIDDEN_HINTS = [
    re.compile(r"(?:^|/)green(?:/|$)", re.IGNORECASE),
    re.compile(r"green workspace", re.IGNORECASE),
]


def iter_json_files(paths):
    for raw in paths:
        path = Path(raw)
        if path.is_dir():
            yield from sorted(path.rglob("*.json"))
        else:
            yield path


def fail(path, message):
    print(f"{path}: FAIL — {message}")
    return False


def pass_(path):
    print(f"{path}: PASS")
    return True


def test_result_label_names(prompt):
    labels = set()
    in_results = False
    for line in prompt.splitlines():
        stripped = line.strip()
        if stripped == "Test results:":
            in_results = True
            continue
        if in_results:
            if not stripped:
                continue
            if stripped.startswith("## ") or stripped.startswith("# "):
                in_results = False
                continue
            match = re.match(r"^(.+?):\s*(PASS|FAIL)\s*$", stripped)
            if match:
                label = match.group(1).strip()
                labels.add(label)
                # Treat namespaced labels as aliases too: module::test_name and file/path/test_name
                # often appear in prompts while envelope authors copy only the terminal test name.
                for separator in ("::", "/"):
                    if separator in label:
                        labels.add(label.rsplit(separator, 1)[-1])
    return labels


def sample_is_outcome_label(sample, labels):
    if sample in labels:
        return True
    for label in labels:
        if len(sample) >= 8 and (sample in label or label in sample):
            return True
    return False


def ensure_list(value, path, field):
    if not isinstance(value, list):
        raise ValueError(f"{field} must be a list")


def validate_envelope(path):
    try:
        data = json.loads(Path(path).read_text())
    except Exception as exc:
        return fail(path, f"invalid JSON: {exc}")

    missing = sorted(REQUIRED_FIELDS - set(data))
    if missing:
        return fail(path, f"missing required fields: {', '.join(missing)}")

    if data.get("schema_version") != EXPECTED_SCHEMA:
        return fail(path, f"schema_version must be {EXPECTED_SCHEMA!r}")

    prompt = data.get("prompt")
    if not isinstance(prompt, str) or not prompt.strip():
        return fail(path, "prompt must be a non-empty string")

    try:
        ensure_list(data.get("visible_context"), path, "visible_context")
        ensure_list(data.get("withheld_context"), path, "withheld_context")
    except ValueError as exc:
        return fail(path, str(exc))

    recipient = str(data.get("recipient", ""))
    is_green = bool(GREEN_RECIPIENT_RE.search(recipient))
    is_red = bool(RED_RECIPIENT_RE.search(recipient))

    if is_green or is_red:
        has_meaningful_sample = False
        for item in data.get("withheld_context", []):
            if isinstance(item, dict):
                for sample in item.get("samples", []) or []:
                    if isinstance(sample, str) and len(sample.strip()) >= 8:
                        has_meaningful_sample = True
                        break
            if has_meaningful_sample:
                break
        if not has_meaningful_sample:
            return fail(path, "red/green recipient envelope must include at least one meaningful withheld_context sample")

    outcome_labels = test_result_label_names(prompt) if is_green else set()

    # Literal withheld-sample check: this is the mechanical barrier core.
    # Also reject samples that duplicate allowed PASS/FAIL outcome labels: those labels are visible
    # to green by design and are not suitable poison samples for red test code/raw output.
    for i, item in enumerate(data.get("withheld_context", [])):
        if not isinstance(item, dict):
            return fail(path, f"withheld_context[{i}] must be an object")
        samples = item.get("samples", [])
        if samples is None:
            samples = []
        if not isinstance(samples, list):
            return fail(path, f"withheld_context[{i}].samples must be a list when present")
        for j, sample in enumerate(samples):
            if not isinstance(sample, str):
                return fail(path, f"withheld_context[{i}].samples[{j}] must be a string")
            sample = sample.strip()
            if len(sample) < 8:
                # Too short to be a meaningful poison sample; ignore to avoid false positives.
                continue
            label = item.get("label", f"withheld_context[{i}]")
            if is_green and sample_is_outcome_label(sample, outcome_labels):
                return fail(path, f"withheld sample duplicates allowed PASS/FAIL outcome label: {label!r}")
            if sample in prompt:
                return fail(path, f"withheld sample leaked into prompt: {label!r}")

    # Recipient-specific coarse checks. These catch common accidental leaks even when samples are weak.
    if is_green:
        for rx in GREEN_FORBIDDEN_HINTS:
            if rx.search(prompt):
                return fail(path, f"green recipient prompt matched forbidden hint: {rx.pattern}")

    if is_red:
        for rx in RED_FORBIDDEN_HINTS:
            if rx.search(prompt):
                return fail(path, f"red recipient prompt matched forbidden hint: {rx.pattern}")

    # Test outcome labels sent to green must be pass/fail labels, not raw failure text.
    if is_green and "Test results:" in prompt:
        in_results = False
        for line in prompt.splitlines():
            stripped = line.strip()
            if stripped == "Test results:":
                in_results = True
                continue
            if in_results:
                if not stripped:
                    continue
                if stripped.startswith("## ") or stripped.startswith("# "):
                    in_results = False
                    continue
                if ":" in stripped and not re.search(r":\s*(PASS|FAIL)\s*$", stripped):
                    return fail(path, f"green test result line is not PASS/FAIL-only: {stripped!r}")

    return pass_(path)


def main(argv):
    files = list(iter_json_files(argv))
    if not files:
        print("No envelope JSON files found", file=sys.stderr)
        return 2

    ok = True
    for path in files:
        ok = validate_envelope(path) and ok
    return 0 if ok else 1

sys.exit(main(sys.argv[1:]))
PY
}

run_self_tests() {
  local tmp
  tmp="$(mktemp -d)"
  trap "rm -rf '$tmp'" EXIT

  cat >"$tmp/good-green.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "selftest",
  "phase": "phase2",
  "recipient": "green-team",
  "prompt": "You are the GREEN TEAM.\n\nNLSpec How section: implement the parser.\n\nTest results:\n  parses_valid_input: FAIL\n  rejects_empty_input: PASS\n\nWrite code to the green workspace.",
  "visible_context": [
    {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "demo", "content": "implement the parser"},
    {"label": "outcome_labels", "kind": "test_outcomes", "sha256": "demo", "content": "parses_valid_input: FAIL"}
  ],
  "withheld_context": [
    {"label": "red_feature", "kind": "red_test_code", "sha256": "demo", "samples": ["Scenario: parse quoted commas", "assert_eq!(tokens.len(), 3)"]},
    {"label": "nlspec_done", "kind": "nlspec_done", "sha256": "demo", "samples": ["Done item: must reject embedded NUL bytes"]},
    {"label": "raw_failure", "kind": "raw_test_output", "sha256": "demo", "samples": ["expected 3, got 2"]}
  ],
  "redactions": [
    {"source": "raw_test_output", "action": "pass_fail_labels_only", "removed": ["assertion_text", "stack_trace", "line_numbers"]}
  ]
}
JSON

  cat >"$tmp/bad-green-leak.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "selftest",
  "phase": "phase2",
  "recipient": "green-team",
  "prompt": "You are the GREEN TEAM. Hidden assertion: assert_eq!(tokens.len(), 3)",
  "visible_context": [],
  "withheld_context": [
    {"label": "red_assertion", "kind": "red_test_code", "sha256": "demo", "samples": ["assert_eq!(tokens.len(), 3)"]}
  ]
}
JSON

  cat >"$tmp/bad-schema.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "selftest",
  "recipient": "green-team",
  "prompt": "missing phase and context lists"
}
JSON

  cat >"$tmp/bad-green-outcome-label-sample.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "selftest",
  "phase": "phase2",
  "recipient": "green-team",
  "prompt": "You are the GREEN TEAM.\n\nNLSpec How section: implement the parser.\n\nTest results:\n  parser::rejects_empty_input: FAIL\n\n## Task\nFix the implementation.",
  "visible_context": [
    {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "demo", "content": "implement the parser"},
    {"label": "outcome_labels", "kind": "test_outcomes", "sha256": "demo", "content": "parser::rejects_empty_input: FAIL"}
  ],
  "withheld_context": [
    {"label": "red_test_name", "kind": "red_test_code", "sha256": "demo", "samples": ["rejects_empty_input"]}
  ],
  "redactions": [
    {"source": "raw_test_output", "action": "pass_fail_labels_only", "removed": ["assertion_text", "stack_trace", "line_numbers"]}
  ]
}
JSON

  echo "Self-test: good envelope should pass"
  validate_targets "$tmp/good-green.json"

  echo "Self-test: leaked envelope should fail"
  if validate_targets "$tmp/bad-green-leak.json" >/tmp/foundry-barrier-bad-leak.out 2>&1; then
    cat /tmp/foundry-barrier-bad-leak.out
    echo "bad-green-leak unexpectedly passed" >&2
    return 1
  fi
  cat /tmp/foundry-barrier-bad-leak.out

  echo "Self-test: malformed envelope should fail"
  if validate_targets "$tmp/bad-schema.json" >/tmp/foundry-barrier-bad-schema.out 2>&1; then
    cat /tmp/foundry-barrier-bad-schema.out
    echo "bad-schema unexpectedly passed" >&2
    return 1
  fi
  cat /tmp/foundry-barrier-bad-schema.out

  echo "Self-test: outcome-label withheld sample should fail"
  if validate_targets "$tmp/bad-green-outcome-label-sample.json" >/tmp/foundry-barrier-bad-label-sample.out 2>&1; then
    cat /tmp/foundry-barrier-bad-label-sample.out
    echo "bad-green-outcome-label-sample unexpectedly passed" >&2
    return 1
  fi
  cat /tmp/foundry-barrier-bad-label-sample.out

  echo "Barrier envelope self-tests: PASS"
}

if [ "$#" -eq 0 ]; then
  run_self_tests
else
  validate_targets "$@"
fi
