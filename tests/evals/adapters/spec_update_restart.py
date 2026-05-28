from __future__ import annotations

from pathlib import Path

from adapters.common import envelope, fail, pass_case, require_columns, validate_with_barrier, write_json

REQUIRED_COLUMNS = [
    "case_id",
    "original_spec_path",
    "existing_nlspec_path",
    "gap_description",
    "red_test_paths",
    "revision_count",
    "revision_cap",
    "expected_route",
    "expected_restart",
]


def as_int(case: dict[str, str], column: str) -> int:
    try:
        return int(case[column])
    except ValueError as exc:
        fail(case["case_id"], f"{column} must be an integer")
        raise AssertionError from exc


def as_bool(case: dict[str, str], column: str) -> bool:
    raw = case[column].strip().lower()
    if raw == "true":
        return True
    if raw == "false":
        return False
    fail(case["case_id"], f"{column} must be true or false")
    raise AssertionError


def expected_route(case: dict[str, str]) -> str:
    revision_count = as_int(case, "revision_count")
    revision_cap = as_int(case, "revision_cap")
    return "user" if revision_count >= revision_cap else "phase1_restart"


def red_test_paths(case: dict[str, str]) -> list[str]:
    return [part.strip() for part in case["red_test_paths"].split(";") if part.strip()]


def nlspec_rerun_envelope(case: dict[str, str]) -> dict:
    prompt = f"""You are the NLSpec agent. Revise the existing NLSpec from this enriched rerun input.

NLSpecRerunInput:
  original_spec_path: {case['original_spec_path']}
  existing_nlspec_path: {case['existing_nlspec_path']}
  evaluator_feedback: {case['gap_description']}

Use existing_nlspec_path as your starting point. Incorporate evaluator_feedback exactly as a gap to address. Do not paraphrase evaluator_feedback. The original spec remains authoritative for unrelated behavior.
"""
    return envelope(
        run_id=f"spec-update-eval-{case['case_id']}",
        phase="spec_update_and_restart",
        recipient="foundry:nlspec",
        prompt=prompt,
        visible=[
            {"label": "original_spec_path", "kind": "spec_path", "sha256": "eval", "content": case["original_spec_path"]},
            {"label": "existing_nlspec_path", "kind": "nlspec_path", "sha256": "eval", "content": case["existing_nlspec_path"]},
            {"label": "evaluator_feedback", "kind": "gap_description", "sha256": "eval", "content": case["gap_description"]},
        ],
        withheld=[
            {"label": "orchestrator_authored_nlspec", "kind": "forbidden_action", "sha256": "eval", "samples": ["orchestrator writes revised NLSpec directly"]},
        ],
        redactions=[{"source": "divergence_judgment", "action": "pass_gap_description_verbatim", "removed": ["paraphrase", "orchestrator_summary"]}],
    )


def spec_update_record(case: dict[str, str]) -> dict:
    before = f"commit-before-{case['case_id']}"
    after = f"commit-after-{case['case_id']}"
    return {
        "route": "spec_update_and_restart",
        "nlspec_author": "nlspec-agent",
        "orchestrator_wrote_nlspec": False,
        "original_spec_path": case["original_spec_path"],
        "existing_nlspec_path": case["existing_nlspec_path"],
        "evaluator_feedback": case["gap_description"],
        "gap_description_verbatim": True,
        "commit_before": before,
        "commit_after": after,
        "revision_record": {"commit_before": before, "commit_after": after},
    }


def phase1_restart_package(case: dict[str, str]) -> dict:
    return {
        "route": "phase1_restart",
        "existing_tests": red_test_paths(case),
        "new_nlspec_path": case["existing_nlspec_path"],
        "change_summary": {
            "sections_added": [],
            "sections_modified": ["3. How", "4. Definition of Done"],
            "requirements_delta": [case["gap_description"]],
        },
        "red_test_paths": red_test_paths(case),
        "test_failure_tracker": "reset_all_counters",
        "revision_history_count": 1,
    }


def user_escalation(case: dict[str, str]) -> dict:
    return {
        "route": "user",
        "pause": True,
        "reason": "revision_cap_reached",
        "revision_count": as_int(case, "revision_count"),
        "revision_cap": as_int(case, "revision_cap"),
        "revision_history": [
            {"commit_before": f"old-before-{i}", "commit_after": f"old-after-{i}"}
            for i in range(as_int(case, "revision_count"))
        ],
        "nlspec_agent_invoked": False,
        "orchestrator_wrote_nlspec": False,
        "evaluator_feedback": case["gap_description"],
    }


def validate_restart_records(case: dict[str, str], update: dict, restart: dict) -> None:
    if update["evaluator_feedback"] != case["gap_description"]:
        fail(case["case_id"], "evaluator_feedback must equal findings[0].gap_description verbatim")
    if update["orchestrator_wrote_nlspec"] is not False:
        fail(case["case_id"], "orchestrator must not author NLSpec content")
    if restart["test_failure_tracker"] != "reset_all_counters":
        fail(case["case_id"], "Phase 1 restart must reset TestFailureTracker")
    if restart["revision_history_count"] != 1:
        fail(case["case_id"], "restart event must record revision_history_count exactly 1")


def run_case(case: dict[str, str], base: Path, barrier_validator: Path) -> None:
    actual_route = expected_route(case)
    if case["expected_route"] != actual_route:
        fail(case["case_id"], f"expected_route {case['expected_route']!r} should be {actual_route!r}")
    should_restart = as_bool(case, "expected_restart")
    if should_restart != (actual_route == "phase1_restart"):
        fail(case["case_id"], "expected_restart does not match expected_route")

    case_dir = base / case["case_id"]
    if actual_route == "phase1_restart":
        envelope_path = case_dir / "dispatch" / "spec_update_and_restart" / "nlspec-rerun.json"
        write_json(envelope_path, nlspec_rerun_envelope(case))
        validate_with_barrier(envelope_path, barrier_validator, case["case_id"])

        update = spec_update_record(case)
        restart = phase1_restart_package(case)
        validate_restart_records(case, update, restart)
        write_json(case_dir / "spec-update-and-restart.json", update)
        write_json(case_dir / "phase1-restart-package.json", restart)
    elif actual_route == "user":
        escalation = user_escalation(case)
        if escalation["nlspec_agent_invoked"] is not False or escalation["orchestrator_wrote_nlspec"] is not False:
            fail(case["case_id"], "revision cap must pause without NLSpec mutation")
        write_json(case_dir / "user-escalation.json", escalation)
    else:
        fail(case["case_id"], f"unsupported route {actual_route!r}")

    pass_case(case["case_id"])


def run(*, root: Path, feature_path: Path, cases: list[dict[str, str]], work_dir: Path, barrier_validator: Path) -> None:
    require_columns("spec-update-restart", cases, REQUIRED_COLUMNS)
    for case in cases:
        run_case(case, work_dir, barrier_validator)
