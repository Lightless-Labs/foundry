from __future__ import annotations

from pathlib import Path

from adapters.common import fail, pass_case, require_columns, write_json

REQUIRED_COLUMNS = [
    "case_id",
    "consecutive_fails",
    "test_hash_status",
    "implementation_hashes",
    "strategy",
    "threshold",
    "expected_decision",
    "expected_reason",
]

VALID_DECISIONS = {"trigger", "continue_green"}
VALID_REASONS = {
    "fixed_threshold",
    "adaptive_impl_changed",
    "waiting_for_more_signal",
    "test_changed_reset",
}


def parse_int(case: dict[str, str], field: str) -> int:
    try:
        value = int(case[field])
    except ValueError as exc:
        fail(case["case_id"], f"{field} must be an integer, got {case[field]!r}")
        raise AssertionError from exc
    if value < 0:
        fail(case["case_id"], f"{field} must be non-negative")
    return value


def parse_hashes(raw: str) -> list[str]:
    return [part.strip() for part in raw.split(",") if part.strip()]


def distinct_in_order(items: list[str]) -> list[str]:
    seen: set[str] = set()
    distinct: list[str] = []
    for item in items:
        if item not in seen:
            seen.add(item)
            distinct.append(item)
    return distinct


def decide(case: dict[str, str]) -> tuple[str, str]:
    consecutive_fails = parse_int(case, "consecutive_fails")
    threshold = parse_int(case, "threshold")
    hashes = parse_hashes(case["implementation_hashes"])
    distinct_hashes = distinct_in_order(hashes)
    strategy = case["strategy"]
    test_hash_status = case["test_hash_status"]

    if strategy != "adaptive_with_fixed_floor":
        fail(case["case_id"], f"unsupported strategy {strategy!r}")
    if threshold < 2:
        fail(case["case_id"], "threshold must be at least 2 so first failures do not trigger divergence")
    if test_hash_status not in {"unchanged", "changed"}:
        fail(case["case_id"], f"unsupported test_hash_status {test_hash_status!r}")

    if test_hash_status == "changed":
        # Red changed the test artifact, so the tracker has reset to the current
        # failing run and must not reuse stale failure evidence.
        return "continue_green", "test_changed_reset"

    adaptive_ready = consecutive_fails >= 2 and len(distinct_hashes) >= 2
    if adaptive_ready:
        return "trigger", "adaptive_impl_changed"

    if consecutive_fails >= threshold:
        return "trigger", "fixed_threshold"

    return "continue_green", "waiting_for_more_signal"


def validate_case(case: dict[str, str], actual_decision: str, actual_reason: str) -> None:
    expected_decision = case["expected_decision"]
    expected_reason = case["expected_reason"]
    if expected_decision not in VALID_DECISIONS:
        fail(case["case_id"], f"invalid expected_decision {expected_decision!r}")
    if expected_reason not in VALID_REASONS:
        fail(case["case_id"], f"invalid expected_reason {expected_reason!r}")
    if actual_decision != expected_decision:
        fail(case["case_id"], f"decision {actual_decision!r} != expected {expected_decision!r}")
    if actual_reason != expected_reason:
        fail(case["case_id"], f"reason {actual_reason!r} != expected {expected_reason!r}")


def run_case(case: dict[str, str], base: Path) -> None:
    actual_decision, actual_reason = decide(case)
    validate_case(case, actual_decision, actual_reason)

    hashes = parse_hashes(case["implementation_hashes"])
    record = {
        "case_id": case["case_id"],
        "strategy": case["strategy"],
        "threshold": parse_int(case, "threshold"),
        "consecutive_fails": parse_int(case, "consecutive_fails"),
        "test_hash_status": case["test_hash_status"],
        "implementation_attempt_hashes": hashes,
        "distinct_implementation_attempt_hashes": distinct_in_order(hashes),
        "decision": actual_decision,
        "reason": actual_reason,
        "routes_to_divergence_evaluator": actual_decision == "trigger",
        "green_feedback_remains_pass_fail_only": True,
    }
    write_json(base / case["case_id"] / "phase2-trigger-decision.json", record)
    pass_case(case["case_id"])


def run(*, root: Path, feature_path: Path, cases: list[dict[str, str]], work_dir: Path, barrier_validator: Path) -> None:
    require_columns("phase2-trigger-strategy", cases, REQUIRED_COLUMNS)
    expected_case_ids = {
        "fixed_threshold_third_fail",
        "adaptive_two_distinct_attempts",
        "no_early_when_impl_unchanged",
        "reset_when_test_hash_changes",
    }
    seen_case_ids = {case["case_id"] for case in cases}
    missing = sorted(expected_case_ids - seen_case_ids)
    if missing:
        fail("phase2-trigger-strategy", f"missing required trigger cases: {missing}")
    for case in cases:
        run_case(case, work_dir)
