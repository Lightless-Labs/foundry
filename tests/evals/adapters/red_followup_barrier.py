from __future__ import annotations

from pathlib import Path

from adapters.common import envelope, pass_case, require_columns, validate_with_barrier, write_json

REQUIRED_COLUMNS = [
    "case_id",
    "route_source",
    "spec_context",
    "red_artifact",
    "redacted_feedback",
    "implementation_sample",
    "counterpart_path_sample",
    "counterpart_reasoning_sample",
]


def red_envelope(case: dict[str, str]) -> dict:
    prompt = f"""You are the RED TEAM. Revise red-side tests according to the spec and redacted process feedback.

Route source: {case['route_source']}
Spec/NLSpec context: {case['spec_context']}
Red-side artifact to revise: {case['red_artifact']}
Redacted feedback: {case['redacted_feedback']}

Do not inspect implementation files, counterpart workspace paths, counterpart reasoning, or runtime internals. Work only from the spec/NLSpec and red-side artifacts.
"""
    return envelope(
        run_id=f"red-followup-eval-{case['case_id']}",
        phase="phase1b",
        recipient="red-team",
        prompt=prompt,
        visible=[
            {"label": "route_source", "kind": "routing_metadata", "sha256": "eval", "content": case["route_source"]},
            {"label": "spec_context", "kind": "nlspec", "sha256": "eval", "content": case["spec_context"]},
            {"label": "red_artifact", "kind": "red_test_artifact", "sha256": "eval", "content": case["red_artifact"]},
            {"label": "redacted_feedback", "kind": "reviewer_feedback", "sha256": "eval", "content": case["redacted_feedback"]},
        ],
        withheld=[
            {"label": "implementation_code", "kind": "implementation", "sha256": "eval", "samples": [case["implementation_sample"]]},
            {"label": "counterpart_path", "kind": "green_path", "sha256": "eval", "samples": [case["counterpart_path_sample"]]},
            {"label": "counterpart_reasoning", "kind": "green_reasoning", "sha256": "eval", "samples": [case["counterpart_reasoning_sample"]]},
        ],
        redactions=[
            {"source": "implementation", "action": "remove_implementation_code", "removed": ["source_snippets", "file_paths", "runtime_internals"]},
            {"source": "counterpart_messages", "action": "remove_counterpart_reasoning", "removed": ["reasoning", "debug_trace", "implementation_strategy"]},
        ],
    )


def run_case(case: dict[str, str], base: Path, barrier_validator: Path) -> None:
    path = base / case["case_id"] / "dispatch" / "phase1b" / "red-team-followup.json"
    write_json(path, red_envelope(case))
    validate_with_barrier(path, barrier_validator, case["case_id"])
    pass_case(case["case_id"])


def run(*, root: Path, feature_path: Path, cases: list[dict[str, str]], work_dir: Path, barrier_validator: Path) -> None:
    require_columns("red-followup-barrier", cases, REQUIRED_COLUMNS)
    for case in cases:
        run_case(case, work_dir, barrier_validator)
