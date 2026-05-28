from __future__ import annotations

from pathlib import Path

from adapters.common import envelope, fail, pass_case, require_columns, validate_with_barrier, write_json

REQUIRED_COLUMNS = [
    "case_id",
    "nlspec_how",
    "test_results",
    "red_test_sample",
    "raw_failure_sample",
    "nlspec_done_sample",
]


def outcome_lines(raw: str) -> list[str]:
    lines = []
    for part in raw.split(";"):
        stripped = part.strip()
        if stripped:
            lines.append(stripped)
    if not lines:
        raise ValueError("test_results must contain at least one PASS/FAIL label")
    return lines


def green_envelope(case: dict[str, str]) -> dict:
    results = outcome_lines(case["test_results"])
    result_block = "\n".join(f"  {line}" for line in results)
    prompt = f"""You are the GREEN TEAM. Implement according to the NLSpec How section only.

NLSpec How section: {case['nlspec_how']}

Test results:
{result_block}

## Task
Use only the NLSpec How section and the PASS/FAIL labels above. Do not request or infer red test code, assertions, expected values, raw failure output, stack traces, or NLSpec Done criteria.
"""
    return envelope(
        run_id=f"green-followup-eval-{case['case_id']}",
        phase="phase2",
        recipient="green-team",
        prompt=prompt,
        visible=[
            {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "eval", "content": case["nlspec_how"]},
            {"label": "outcome_labels", "kind": "test_outcomes", "sha256": "eval", "content": "\n".join(results)},
        ],
        withheld=[
            {"label": "red_test_artifact", "kind": "red_test_code", "sha256": "eval", "samples": [case["red_test_sample"]]},
            {"label": "raw_failure", "kind": "raw_test_output", "sha256": "eval", "samples": [case["raw_failure_sample"]]},
            {"label": "nlspec_done", "kind": "nlspec_done", "sha256": "eval", "samples": [case["nlspec_done_sample"]]},
        ],
        redactions=[
            {"source": "nlspec", "action": "how_only", "removed": ["why", "what", "done"]},
            {"source": "red_test_output", "action": "pass_fail_labels_only", "removed": ["feature_text", "assertions", "expected_values", "raw_failure_output", "stack_trace"]},
        ],
    )


def run_case(case: dict[str, str], base: Path, barrier_validator: Path) -> None:
    try:
        outcome_lines(case["test_results"])
    except ValueError as exc:
        fail(case["case_id"], str(exc))

    path = base / case["case_id"] / "dispatch" / "phase2" / "green-team-followup.json"
    write_json(path, green_envelope(case))
    validate_with_barrier(path, barrier_validator, case["case_id"])
    pass_case(case["case_id"])


def run(*, root: Path, feature_path: Path, cases: list[dict[str, str]], work_dir: Path, barrier_validator: Path) -> None:
    require_columns("green-followup-barrier", cases, REQUIRED_COLUMNS)
    for case in cases:
        run_case(case, work_dir, barrier_validator)
