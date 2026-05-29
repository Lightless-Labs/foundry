from __future__ import annotations

from pathlib import Path

from adapters.common import envelope, fail, pass_case, require_columns, validate_with_barrier, write_json

REQUIRED_COLUMNS = [
    "case_id",
    "language",
    "has_build",
    "has_udl",
    "touches_io",
    "expected_reviewers",
]

LANGUAGE_REVIEWERS = {
    "rust": "rust-reviewer",
    "swift": "swift-reviewer",
    "typescript": "typescript-reviewer",
}

BASE_REVIEWERS = [
    "green-team-reviewer",
    "red-team-test-reviewer",
    "barrier-integrity-auditor",
    "correctness-reviewer",
    "testing-reviewer",
]

IMPLEMENTATION_FACING = {
    "green-team-reviewer",
    "rust-reviewer",
    "swift-reviewer",
    "typescript-reviewer",
    "bazel-reviewer",
    "uniffi-bridge-reviewer",
    "correctness-reviewer",
    "reliability-reviewer",
}

TEST_FACING = {"red-team-test-reviewer", "testing-reviewer"}

NLSPEC_HOW = "Parse canonical Roman numeral input, validate malformed subtractives, and return typed errors."
NLSPEC_DONE_SAMPLE = "Done: invalid subtractive notation such as IC must be rejected"
TEST_OUTCOMES = ["roman_valid_viii: PASS", "roman_invalid_ic: PASS", "roman_empty_input: PASS"]
RED_TEST_SAMPLE = "Scenario: reject IC as invalid subtractive notation"
RED_ASSERTION_SAMPLE = "assert_eq!(parse_roman(\"IC\"), Err(RomanError::InvalidSubtractive))"
RAW_FAILURE_SAMPLE = "expected Err(InvalidSubtractive), got Ok(99)"
IMPLEMENTATION_SNIPPET = "pub fn parse_roman(input: &str) -> Result<u32, RomanError> { validate_then_sum(input) }"
IMPLEMENTATION_PATH = "src/roman.rs"
TEST_FILE_PATH = "tests/features/roman.feature"
STEP_FILE_PATH = "tests/steps/roman_steps.rs"


def as_bool(case: dict[str, str], column: str) -> bool:
    raw = case[column].strip().lower()
    if raw == "true":
        return True
    if raw == "false":
        return False
    fail(case["case_id"], f"{column} must be true or false")
    raise AssertionError


def expected_reviewers(case: dict[str, str]) -> list[str]:
    return [part.strip() for part in case["expected_reviewers"].split(";") if part.strip()]


def selected_reviewers(case: dict[str, str]) -> list[str]:
    language = case["language"].strip().lower()
    if language not in LANGUAGE_REVIEWERS:
        fail(case["case_id"], f"unsupported language {language!r}")

    reviewers = BASE_REVIEWERS.copy()
    reviewers.insert(3, LANGUAGE_REVIEWERS[language])
    insertion_index = 4
    if as_bool(case, "has_build"):
        reviewers.insert(insertion_index, "bazel-reviewer")
        insertion_index += 1
    if as_bool(case, "has_udl"):
        reviewers.insert(insertion_index, "uniffi-bridge-reviewer")
        insertion_index += 1
    if as_bool(case, "touches_io"):
        reviewers.append("reliability-reviewer")
    return reviewers


def result_block() -> str:
    return "\n".join(f"  {line}" for line in TEST_OUTCOMES)


def implementation_prompt(reviewer: str, case: dict[str, str]) -> str:
    focus = {
        "green-team-reviewer": "code quality under information-barrier constraints",
        "rust-reviewer": "Rust ownership, error handling, Send/Sync, and unsafe usage",
        "swift-reviewer": "Swift API shape, concurrency, and platform integration",
        "typescript-reviewer": "TypeScript type safety, narrowing, and nullable handling",
        "bazel-reviewer": "Bazel target shape, dependency hygiene, and hermeticity",
        "uniffi-bridge-reviewer": "UniFFI bridge safety, exported types, and FFI boundaries",
        "correctness-reviewer": "logic errors, edge cases, and invariant preservation",
        "reliability-reviewer": "I/O error handling, retries, timeouts, and resilience",
    }[reviewer]

    return f"""You are foundry:review:{reviewer}. Review the final implementation for {focus}.

NLSpec How section: {NLSPEC_HOW}

Implementation files:
- {IMPLEMENTATION_PATH}

Implementation excerpt:
{IMPLEMENTATION_SNIPPET}

Test results:
{result_block()}

## Task
Return findings as JSON matching the findings schema. Use only NLSpec How, implementation artifacts, and PASS/FAIL labels. Do not request or infer test code, assertions, raw failure output, or NLSpec Done criteria.
"""


def test_prompt(reviewer: str) -> str:
    focus = {
        "red-team-test-reviewer": "coverage against the NLSpec Definition of Done",
        "testing-reviewer": "assertion strength, edge-case coverage, and weak or flaky tests",
    }[reviewer]
    return f"""You are foundry:review:{reviewer}. Review the final test suite for {focus}.

NLSpec Definition of Done excerpt:
- Reject malformed subtractive notation.
- Reject empty input with a typed error.

Test artifacts:
- {TEST_FILE_PATH}
- {STEP_FILE_PATH}

Representative test excerpt:
{RED_TEST_SAMPLE}

Representative assertion excerpt:
{RED_ASSERTION_SAMPLE}

## Task
Return findings as JSON matching the findings schema. You do not see implementation code or implementation paths. Judge only the NLSpec Definition of Done and test artifacts.
"""


def barrier_prompt(case: dict[str, str], reviewers: list[str]) -> str:
    envelope_paths = "\n".join(
        f"- runs/reviewer-fanout-eval-{case['case_id']}/dispatch/phase3/{reviewer}.json" for reviewer in reviewers
    )
    return f"""You are foundry:review:barrier-integrity-auditor. Final barrier audit.

Replay these PromptEnvelope artifacts:
{envelope_paths}

Verify that implementation-facing reviewers saw only NLSpec How, implementation artifacts, and PASS/FAIL labels; test-facing reviewers saw only NLSpec Done/test artifacts; and green-team-reviewer received no test code, assertions, raw failures, or NLSpec Done criteria.
Report any leak as P0.
"""


def reviewer_envelope(case: dict[str, str], reviewer: str, reviewers: list[str]) -> dict:
    run_id = f"reviewer-fanout-eval-{case['case_id']}"

    if reviewer == "barrier-integrity-auditor":
        return envelope(
            run_id=run_id,
            phase="phase3",
            recipient="foundry:review:barrier-integrity-auditor",
            prompt=barrier_prompt(case, reviewers),
            visible=[
                {"label": "dispatch_envelope_paths", "kind": "prompt_envelope_paths", "sha256": "eval", "content": ";".join(reviewers)},
            ],
            withheld=[],
        )

    if reviewer in IMPLEMENTATION_FACING:
        return envelope(
            run_id=run_id,
            phase="phase3",
            recipient=f"foundry:review:{reviewer}",
            prompt=implementation_prompt(reviewer, case),
            visible=[
                {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "eval", "content": NLSPEC_HOW},
                {"label": "implementation_excerpt", "kind": "implementation", "sha256": "eval", "content": IMPLEMENTATION_SNIPPET},
                {"label": "test_outcomes", "kind": "test_outcomes", "sha256": "eval", "content": "\n".join(TEST_OUTCOMES)},
            ],
            withheld=[
                {"label": "red_feature", "kind": "red_test_code", "sha256": "eval", "samples": [RED_TEST_SAMPLE, RED_ASSERTION_SAMPLE]},
                {"label": "raw_failure", "kind": "raw_test_output", "sha256": "eval", "samples": [RAW_FAILURE_SAMPLE]},
                {"label": "nlspec_done", "kind": "nlspec_done", "sha256": "eval", "samples": [NLSPEC_DONE_SAMPLE]},
            ],
            redactions=[
                {"source": "nlspec", "action": "how_only", "removed": ["why", "what", "done"]},
                {"source": "red_test_output", "action": "pass_fail_labels_only", "removed": ["feature_text", "assertions", "raw_failure_output"]},
            ],
        )

    if reviewer in TEST_FACING:
        return envelope(
            run_id=run_id,
            phase="phase3",
            recipient=f"foundry:review:{reviewer}",
            prompt=test_prompt(reviewer),
            visible=[
                {"label": "nlspec_done", "kind": "nlspec_done", "sha256": "eval", "content": "reject malformed subtractives; reject empty input"},
                {"label": "red_test_artifacts", "kind": "red_test_code", "sha256": "eval", "content": RED_TEST_SAMPLE},
            ],
            withheld=[
                {"label": "implementation_path", "kind": "implementation", "sha256": "eval", "samples": [IMPLEMENTATION_PATH]},
                {"label": "implementation_code", "kind": "implementation", "sha256": "eval", "samples": [IMPLEMENTATION_SNIPPET]},
            ],
            redactions=[
                {"source": "green_workspace", "action": "withhold_implementation", "removed": ["implementation_paths", "implementation_code", "green_reasoning"]},
            ],
        )

    fail(case["case_id"], f"no envelope template for reviewer {reviewer!r}")
    raise AssertionError


def validate_territory(case: dict[str, str], reviewer: str, prompt: str) -> None:
    if reviewer in IMPLEMENTATION_FACING:
        forbidden = [RED_TEST_SAMPLE, RED_ASSERTION_SAMPLE, RAW_FAILURE_SAMPLE, NLSPEC_DONE_SAMPLE, TEST_FILE_PATH, STEP_FILE_PATH]
        for sample in forbidden:
            if sample in prompt:
                fail(case["case_id"], f"{reviewer} prompt leaked test-facing sample: {sample}")
        if "NLSpec Definition of Done" in prompt:
            fail(case["case_id"], f"{reviewer} prompt referenced NLSpec Definition of Done")
    elif reviewer in TEST_FACING:
        forbidden = [IMPLEMENTATION_SNIPPET, IMPLEMENTATION_PATH]
        for sample in forbidden:
            if sample in prompt:
                fail(case["case_id"], f"{reviewer} prompt leaked implementation-facing sample: {sample}")
    elif reviewer != "barrier-integrity-auditor":
        fail(case["case_id"], f"unclassified reviewer {reviewer!r}")


def validate_selection(case: dict[str, str], actual: list[str], expected: list[str]) -> None:
    if actual != expected:
        fail(case["case_id"], f"selected reviewers {actual!r} did not match expected {expected!r}")
    if len(actual) != len(set(actual)):
        fail(case["case_id"], "selected reviewers must be unique")
    language_reviewers = [reviewer for reviewer in actual if reviewer in LANGUAGE_REVIEWERS.values()]
    if len(language_reviewers) != 1:
        fail(case["case_id"], f"expected exactly one language reviewer, got {language_reviewers!r}")
    if "barrier-integrity-auditor" not in actual:
        fail(case["case_id"], "barrier-integrity-auditor is mandatory")


def run_case(case: dict[str, str], base: Path, barrier_validator: Path) -> None:
    actual = selected_reviewers(case)
    expected = expected_reviewers(case)
    validate_selection(case, actual, expected)

    case_dir = base / case["case_id"]
    dispatch_dir = case_dir / "dispatch" / "phase3"
    plan = {
        "phase": "phase3",
        "parallel": True,
        "selected_reviewers": actual,
        "expected_reviewers": expected,
        "language": case["language"].strip().lower(),
        "conditional_inputs": {
            "has_build": as_bool(case, "has_build"),
            "has_udl": as_bool(case, "has_udl"),
            "touches_io": as_bool(case, "touches_io"),
        },
        "territory": {
            "implementation_facing": sorted(IMPLEMENTATION_FACING.intersection(actual)),
            "test_facing": sorted(TEST_FACING.intersection(actual)),
            "barrier_auditor": "barrier-integrity-auditor",
        },
    }
    write_json(case_dir / "reviewer-fanout-plan.json", plan)

    for reviewer in actual:
        data = reviewer_envelope(case, reviewer, actual)
        validate_territory(case, reviewer, data["prompt"])
        write_json(dispatch_dir / f"{reviewer}.json", data)

    validate_with_barrier(case_dir / "dispatch", barrier_validator, case["case_id"])
    pass_case(case["case_id"])


def run(*, root: Path, feature_path: Path, cases: list[dict[str, str]], work_dir: Path, barrier_validator: Path) -> None:
    require_columns("reviewer-fanout", cases, REQUIRED_COLUMNS)
    for case in cases:
        run_case(case, work_dir, barrier_validator)
