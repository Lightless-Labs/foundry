#!/usr/bin/env bash
# pi-live-dispatch-smoke.sh — slow/manual live Pi smoke for Foundry's public dispatch lane.
#
# This script performs real Pi model calls. It is intentionally not part of the
# fast structural validators. It creates PromptEnvelope artifacts, invokes the
# public foundry_team extension through pi, emits behavioral-smoke.toon from the
# actual child-dispatch model lanes, and validates the resulting run directory.
#
# Usage:
#   tests/pi-live-dispatch-smoke.sh
#   tests/pi-live-dispatch-smoke.sh --keep
#   tests/pi-live-dispatch-smoke.sh --example chess-engine --run-dir runs/manual-pi-live-chess-smoke
#   tests/pi-live-dispatch-smoke.sh --example rubiks-solver --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --keep
#   tests/pi-live-dispatch-smoke.sh --phase-task artifact-sketch --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --keep
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXTENSION="$ROOT_DIR/extensions/pi-foundry-team/index.ts"
BEHAVIORAL_SMOKE="$ROOT_DIR/tests/behavioral-smoke.sh"

KEEP=0
RUN_DIR=""
EXPLICIT_RUN_DIR=0
RED_MODEL="${FOUNDRY_RED_MODEL:-}"
GREEN_MODEL="${FOUNDRY_GREEN_MODEL:-}"
REQUIRE_DISTINCT_MODEL_LANES=0
EXAMPLE="sudoku-solver"
PHASE_TASK="plumbing"

usage() {
  cat <<'USAGE'
Usage: tests/pi-live-dispatch-smoke.sh [--keep] [--run-dir DIR] [--example NAME] [--phase-task TASK] [--red-model MODEL] [--green-model MODEL] [--require-distinct-model-lanes]

Runs a slow/manual live Pi dispatch smoke using the Foundry foundry_team
extension. By default artifacts are written to a temporary directory and cleaned
up on success. Use --keep or --run-dir to inspect the generated run artifacts.

Use --example to choose which worked example's red tests are executed before
live dispatch. Supported examples: sudoku-solver (default), rubiks-solver,
chess-engine.

Use --phase-task to choose the child-agent task shape. Supported tasks:
plumbing (default, exact RED_OK/GREEN_OK) and artifact-sketch (red writes a
lightweight test-plan artifact; green writes a lightweight implementation-plan
artifact from How + PASS/FAIL labels only).

Use --red-model/--green-model to pass explicit per-lane Pi model overrides to
foundry_team. Use --require-distinct-model-lanes for multi-provider/model-lane
exercises; behavioral-smoke.toon will then fail validation if red and green run
on the same actual model lane.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --keep)
      KEEP=1
      shift
      ;;
    --run-dir)
      if [ "$#" -lt 2 ]; then
        echo "--run-dir requires a path" >&2
        exit 2
      fi
      RUN_DIR="$2"
      EXPLICIT_RUN_DIR=1
      KEEP=1
      shift 2
      ;;
    --example)
      if [ "$#" -lt 2 ]; then
        echo "--example requires a worked-example name" >&2
        exit 2
      fi
      EXAMPLE="$2"
      shift 2
      ;;
    --phase-task)
      if [ "$#" -lt 2 ]; then
        echo "--phase-task requires a task name" >&2
        exit 2
      fi
      PHASE_TASK="$2"
      shift 2
      ;;
    --red-model)
      if [ "$#" -lt 2 ]; then
        echo "--red-model requires a model id" >&2
        exit 2
      fi
      RED_MODEL="$2"
      shift 2
      ;;
    --green-model)
      if [ "$#" -lt 2 ]; then
        echo "--green-model requires a model id" >&2
        exit 2
      fi
      GREEN_MODEL="$2"
      shift 2
      ;;
    --require-distinct-model-lanes)
      REQUIRE_DISTINCT_MODEL_LANES=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$PHASE_TASK" in
  plumbing|artifact-sketch)
    ;;
  *)
    echo "Unsupported --phase-task '$PHASE_TASK'. Supported tasks: plumbing, artifact-sketch" >&2
    exit 2
    ;;
esac

case "$EXAMPLE" in
  sudoku-solver)
    EXPECTED_PASSED=30
    EXPECTED_TOTAL=30
    OUTCOME_LABEL="sudoku_red_tests"
    RED_NLSPEC_SUMMARY="Sudoku solver CLI: parse 81 cells, accept 0 or dot blanks, validate rows/columns/boxes, solve by constraint propagation plus backtracking, print a 9-line solved board, and report invalid/unsolvable inputs with non-zero exits."
    PUBLIC_CONTRACT="Worked example: examples/sudoku-solver. Red tests are black-box CLI tests derived from the NLSpec Definition of Done."
    GREEN_HOW="parse an 81-cell Sudoku puzzle; strip whitespace; accept 0 or dot blanks; validate duplicate givens; initialize candidate sets; propagate naked and hidden singles; backtrack with a minimum-remaining-values choice; print one 9-digit row per line"
    WITHHELD_IMPL_SAMPLE_1="CandidateSet { bits: 0x3FE }"
    WITHHELD_IMPL_SAMPLE_2="fn initialize_candidates(board: &Board)"
    WITHHELD_RED_SAMPLE_1="assert_eq!(stdout_str(&output), EASY_SOLUTION)"
    WITHHELD_RED_SAMPLE_2="fn test_rejects_duplicate_in_row()"
    WITHHELD_DONE_SAMPLE_1="6.8 Backtracking / Hard Puzzles"
    WITHHELD_DONE_SAMPLE_2="Definition of Done item: rejects invalid characters"
    WITHHELD_RAW_SAMPLE_1="expected solved grid, got empty stdout"
    WITHHELD_RAW_SAMPLE_2="thread 'test_accepts_dots_as_blanks' panicked"
    ;;
  rubiks-solver)
    EXPECTED_PASSED=46
    EXPECTED_TOTAL=46
    OUTCOME_LABEL="rubiks_red_tests"
    RED_NLSPEC_SUMMARY="Rubik's cube solver CLI: accept a 54-character URFDLB facelet string, validate color counts and cubie legality, recognize solved cubes, apply Kociemba-compatible move conventions anchored by golden vectors, find valid solutions for scrambles, and verify solutions."
    PUBLIC_CONTRACT="Worked example: examples/rubiks-solver. Red tests are black-box CLI tests derived from the repaired NLSpec and Kociemba reference golden vectors."
    GREEN_HOW="parse a 54-character URFDLB facelet string; validate facelet alphabet and color counts; map facelets to cubies; use Kociemba-compatible move tables anchored by golden vectors; search for or report a valid solution; verify that applying the solution returns the cube to solved state"
    WITHHELD_IMPL_SAMPLE_1="const MOVE_R_PERM: [usize; 54]"
    WITHHELD_IMPL_SAMPLE_2="fn solve_kociemba(cube: &Cube)"
    WITHHELD_RED_SAMPLE_1="assert_eq!(apply_moves(SOLVED, \"R U R' U'\"), GOLDEN_RURU)"
    WITHHELD_RED_SAMPLE_2="fn test_superflip_scramble_solves_and_verifies()"
    WITHHELD_DONE_SAMPLE_1="Golden vector: R U2 D' B D' R2"
    WITHHELD_DONE_SAMPLE_2="Definition of Done item: verifies returned solution"
    WITHHELD_RAW_SAMPLE_1="expected Kociemba golden facelet string"
    WITHHELD_RAW_SAMPLE_2="solution did not return cube to solved state"
    ;;
  chess-engine)
    EXPECTED_PASSED=44
    EXPECTED_TOTAL=44
    OUTCOME_LABEL="chess_red_tests"
    RED_NLSPEC_SUMMARY="Chess engine CLI: parse FEN and start positions, generate legal moves including castling/en passant/promotions, report perft counts using known golden vectors, support basic UCI commands, and produce deterministic search results for shallow depths."
    PUBLIC_CONTRACT="Worked example: examples/chess-engine. Red tests are black-box integration tests covering perft golden vectors, FEN, UCI, edge cases, and smoke-level search behavior."
    GREEN_HOW="represent the board with bitboards or an equivalent complete state; parse and emit FEN; generate only legal chess moves including check, castling, en passant, and promotion rules; implement perft as the authoritative move-generation oracle; support UCI uci/isready/position/go/quit commands; return deterministic bestmove output"
    WITHHELD_IMPL_SAMPLE_1="struct Zobrist { piece: [[u64; 64]; 12] }"
    WITHHELD_IMPL_SAMPLE_2="fn generate_legal_moves(board: &Board) -> Vec<Move>"
    WITHHELD_RED_SAMPLE_1="assert_eq!(run_perft(4, KIWIPETE_FEN), 4085603)"
    WITHHELD_RED_SAMPLE_2="fn test_uci_position_startpos_moves()"
    WITHHELD_DONE_SAMPLE_1="Perft Position 4 golden vector"
    WITHHELD_DONE_SAMPLE_2="Definition of Done item: UCI bestmove response"
    WITHHELD_RAW_SAMPLE_1="expected perft leaf-node count 4085603"
    WITHHELD_RAW_SAMPLE_2="uci session did not emit bestmove"
    ;;
  *)
    echo "Unsupported --example '$EXAMPLE'. Supported examples: sudoku-solver, rubiks-solver, chess-engine" >&2
    exit 2
    ;;
esac

EXAMPLE_DIR="$ROOT_DIR/examples/$EXAMPLE"
CARGO_OUTPUT="$RUN_DIR/$EXAMPLE-cargo-test.out"

if ! command -v pi >/dev/null 2>&1; then
  echo "pi-live-dispatch-smoke: FAIL — pi command not found" >&2
  exit 2
fi

if [ ! -f "$EXTENSION" ]; then
  echo "pi-live-dispatch-smoke: FAIL — missing extension $EXTENSION" >&2
  exit 1
fi

if [ ! -d "$EXAMPLE_DIR" ]; then
  echo "pi-live-dispatch-smoke: FAIL — missing example directory $EXAMPLE_DIR" >&2
  exit 1
fi

TMP_ROOT=""
if [ -z "$RUN_DIR" ]; then
  TMP_ROOT="$(mktemp -d)"
  RUN_DIR="$TMP_ROOT/runs/pi-live-dispatch-smoke"
  CARGO_OUTPUT="$RUN_DIR/$EXAMPLE-cargo-test.out"
fi

cleanup() {
  local status=$?
  if [ "$status" -eq 0 ] && [ "$KEEP" -eq 0 ] && [ -n "$TMP_ROOT" ]; then
    rm -rf "$TMP_ROOT"
  elif [ "$status" -ne 0 ] && [ -n "$TMP_ROOT" ]; then
    echo "pi-live-dispatch-smoke artifacts: $RUN_DIR" >&2
  fi
}
trap cleanup EXIT

mkdir -p "$RUN_DIR/dispatch/phase1" "$RUN_DIR/dispatch/phase2"

export EXAMPLE PHASE_TASK RED_NLSPEC_SUMMARY PUBLIC_CONTRACT GREEN_HOW OUTCOME_LABEL
export WITHHELD_IMPL_SAMPLE_1 WITHHELD_IMPL_SAMPLE_2
export WITHHELD_RED_SAMPLE_1 WITHHELD_RED_SAMPLE_2
export WITHHELD_DONE_SAMPLE_1 WITHHELD_DONE_SAMPLE_2
export WITHHELD_RAW_SAMPLE_1 WITHHELD_RAW_SAMPLE_2

python3 - "$RUN_DIR/dispatch/phase1/red-team.json" "$RUN_DIR/dispatch/phase2/green-team.json" <<'PY'
import json
import os
import sys
from pathlib import Path

red_path = Path(sys.argv[1])
green_path = Path(sys.argv[2])
example = os.environ["EXAMPLE"]
phase_task = os.environ["PHASE_TASK"]
red_summary = os.environ["RED_NLSPEC_SUMMARY"]
public_contract = os.environ["PUBLIC_CONTRACT"]
green_how = os.environ["GREEN_HOW"]
outcome_label = os.environ["OUTCOME_LABEL"]

if phase_task == "plumbing":
    red_prompt = (
        "Foundry Pi live dispatch smoke for the red test-writing lane. "
        "This is a benign plumbing check; do not write tests or attack anything. "
        f"Visible NLSpec summary: {red_summary} "
        f"Visible public product contract: {public_contract} "
        "Implementation details are intentionally withheld. Reply exactly: RED_OK"
    )

    green_prompt = (
        "You are the GREEN TEAM in a Foundry Pi live dispatch smoke.\n\n"
        f"NLSpec How section: {green_how}.\n\n"
        "Test results:\n"
        f"  {outcome_label}: PASS\n"
        "  pi_foundry_team_dispatch_smoke: PASS\n\n"
        "## Task\n"
        "Reply exactly: GREEN_OK"
    )
else:
    red_prompt = (
        "You are the RED TEAM in a Foundry provider-diverse live phase-artifact smoke.\n\n"
        "Allowed context:\n"
        f"- Example: {example}\n"
        f"- Full NLSpec summary: {red_summary}\n"
        f"- Public product contract: {public_contract}\n\n"
        "Implementation code, green-team reasoning, and implementation workspace details are intentionally withheld.\n\n"
        "## Task\n"
        "Return only one JSON object with this exact shape:\n"
        f"{{\"artifact_type\":\"red_test_plan\",\"example\":\"{example}\",\"implementation_visible\":false,\"test_categories\":[\"...\",\"...\",\"...\"],\"oracle_strategy\":\"...\"}}\n"
        "Do not include Markdown fences. Do not invent implementation details."
    )

    green_prompt = (
        "You are the GREEN TEAM in a Foundry provider-diverse live phase-artifact smoke.\n\n"
        "Allowed context:\n"
        f"- Example: {example}\n"
        f"- NLSpec How section: {green_how}.\n\n"
        "Test results:\n"
        f"  {outcome_label}: PASS\n"
        "  pi_foundry_team_dispatch_smoke: PASS\n\n"
        "You may not see red test code, assertions, expected values, raw failures, stack traces, or the NLSpec Done section.\n\n"
        "## Task\n"
        "Return only one JSON object with this exact shape:\n"
        f"{{\"artifact_type\":\"green_implementation_plan\",\"example\":\"{example}\",\"saw_red_tests\":false,\"permitted_feedback\":\"PASS_FAIL_ONLY\",\"implementation_steps\":[\"...\",\"...\",\"...\"]}}\n"
        "Do not include Markdown fences. Do not mention hidden tests, assertions, expected values, or raw failure details."
    )

red_envelope = {
    "schema_version": "foundry.prompt-envelope.v1",
    "run_id": "pi-live-dispatch-smoke",
    "phase": "phase1",
    "recipient": "red-team",
    "prompt": red_prompt,
    "visible_context": [
        {"label": "full_nlspec", "kind": "nlspec", "sha256": "live-smoke", "content": red_summary},
        {"label": "spec", "kind": "spec", "sha256": "live-smoke", "content": public_contract},
    ],
    "withheld_context": [
        {
            "label": "implementation_workspace",
            "kind": "implementation_code",
            "sha256": "live-smoke",
            "samples": [os.environ["WITHHELD_IMPL_SAMPLE_1"], os.environ["WITHHELD_IMPL_SAMPLE_2"]],
        }
    ],
    "redactions": [],
}

green_envelope = {
    "schema_version": "foundry.prompt-envelope.v1",
    "run_id": "pi-live-dispatch-smoke",
    "phase": "phase2",
    "recipient": "green-team",
    "prompt": green_prompt,
    "visible_context": [
        {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "live-smoke", "content": green_how},
        {
            "label": "outcome_labels",
            "kind": "test_outcomes",
            "sha256": "live-smoke",
            "content": f"{outcome_label}: PASS\npi_foundry_team_dispatch_smoke: PASS",
        },
    ],
    "withheld_context": [
        {
            "label": "red_test_code",
            "kind": "red_test_code",
            "sha256": "live-smoke",
            "samples": [os.environ["WITHHELD_RED_SAMPLE_1"], os.environ["WITHHELD_RED_SAMPLE_2"]],
        },
        {
            "label": "nlspec_done",
            "kind": "nlspec_done",
            "sha256": "live-smoke",
            "samples": [os.environ["WITHHELD_DONE_SAMPLE_1"], os.environ["WITHHELD_DONE_SAMPLE_2"]],
        },
        {
            "label": "raw_failure",
            "kind": "raw_test_output",
            "sha256": "live-smoke",
            "samples": [os.environ["WITHHELD_RAW_SAMPLE_1"], os.environ["WITHHELD_RAW_SAMPLE_2"]],
        },
    ],
    "redactions": [
        {
            "source": "raw_test_output",
            "action": "pass_fail_labels_only",
            "removed": ["assertion_text", "expected_values", "stack_traces", "line_numbers"],
        }
    ],
}

red_path.write_text(json.dumps(red_envelope, indent=2) + "\n", encoding="utf-8")
green_path.write_text(json.dumps(green_envelope, indent=2) + "\n", encoding="utf-8")
PY

echo "Running $EXAMPLE worked-example red tests (expected $EXPECTED_PASSED/$EXPECTED_TOTAL)..."
(
  cd "$EXAMPLE_DIR"
  cargo test --quiet
) >"$CARGO_OUTPUT" 2>&1

echo "Invoking Pi foundry_team live dispatch..."
DISPATCH_JSON=$(python3 - "$RUN_DIR/dispatch/phase1/red-team.json" "$RUN_DIR/dispatch/phase2/green-team.json" "$RED_MODEL" "$GREEN_MODEL" <<'PY'
import json
import sys
red_path, green_path, red_model, green_model = sys.argv[1:5]
dispatches = [
    {"envelopePath": red_path},
    {"envelopePath": green_path},
]
if red_model:
    dispatches[0]["model"] = red_model
if green_model:
    dispatches[1]["model"] = green_model
print(json.dumps({"dispatches": dispatches}, indent=2))
PY
)
PI_PROMPT=$(cat <<EOF
Use the foundry_team tool exactly once in parallel mode with exactly this JSON argument:

$DISPATCH_JSON

Do not answer directly before calling the tool. The only acceptable action is a foundry_team call with the dispatches above, preserving any model fields exactly.
EOF
)

pi \
  -e "$EXTENSION" \
  --no-extensions \
  --mode json \
  -p \
  --no-session \
  --no-context-files \
  --no-skills \
  --tools foundry_team \
  "$PI_PROMPT" >"$RUN_DIR/pi-foundry-team.jsonl"

python3 - "$RUN_DIR/pi-foundry-team.jsonl" "$RUN_DIR/behavioral-smoke.toon" "$RUN_DIR/phase-artifacts" "$RED_MODEL" "$GREEN_MODEL" "$REQUIRE_DISTINCT_MODEL_LANES" "$EXAMPLE" "$PHASE_TASK" "$EXPECTED_PASSED" "$EXPECTED_TOTAL" <<'PY'
import json
import sys
from pathlib import Path

jsonl_path = Path(sys.argv[1])
toon_path = Path(sys.argv[2])
phase_artifact_dir = Path(sys.argv[3])
expected_models = {
    "red-team": sys.argv[4] or None,
    "green-team": sys.argv[5] or None,
}
requires_distinct_model_lanes = sys.argv[6] == "1" or bool(expected_models["red-team"] and expected_models["green-team"] and expected_models["red-team"] != expected_models["green-team"])
example = sys.argv[7]
phase_task = sys.argv[8]
expected_passed = sys.argv[9]
expected_total = sys.argv[10]

orchestrator_model = None
tool_result = None

for line in jsonl_path.read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        continue

    message = event.get("message") or {}
    if message.get("role") == "assistant" and not orchestrator_model:
        provider = message.get("provider")
        model = message.get("model")
        if model:
            orchestrator_model = f"{provider}/{model}" if provider and "/" not in model else model

    if event.get("type") == "tool_execution_end" and event.get("toolName") == "foundry_team":
        tool_result = event.get("result")

if tool_result is None:
    raise SystemExit("foundry_team tool_execution_end was not observed")
if tool_result.get("isError"):
    raise SystemExit(f"foundry_team returned an error: {tool_result}")

results = (tool_result.get("details") or {}).get("results") or []
by_recipient = {result.get("recipient"): result for result in results}
for recipient in ["red-team", "green-team"]:
    result = by_recipient.get(recipient)
    if not result:
        raise SystemExit(f"missing foundry_team result for {recipient}")
    if result.get("exitCode") != 0:
        raise SystemExit(f"{recipient} exitCode={result.get('exitCode')}: {result}")
    if not result.get("actualModel"):
        raise SystemExit(f"{recipient} missing actualModel in tool result")
    expected_model = expected_models[recipient]
    if expected_model and result.get("plannedModel") != expected_model:
        raise SystemExit(f"{recipient} plannedModel mismatch: expected {expected_model!r}, got {result.get('plannedModel')!r}")
    if expected_model and result.get("actualModel") != expected_model:
        raise SystemExit(f"{recipient} actualModel mismatch: expected {expected_model!r}, got {result.get('actualModel')!r}")

if phase_task == "plumbing":
    for recipient, expected_output in [("red-team", "RED_OK"), ("green-team", "GREEN_OK")]:
        output = by_recipient[recipient].get("output", "").strip()
        if output != expected_output:
            raise SystemExit(f"{recipient} output mismatch: {output!r}")
else:
    def parse_json_object(recipient):
        output = by_recipient[recipient].get("output", "").strip()
        candidates = [output]
        if output.startswith("```"):
            lines = output.splitlines()
            if len(lines) >= 3 and lines[-1].strip() == "```":
                candidates.append("\n".join(lines[1:-1]).strip())
        first = output.find("{")
        last = output.rfind("}")
        if first != -1 and last > first:
            candidates.append(output[first:last + 1])
        last_error = None
        for candidate in candidates:
            try:
                parsed = json.loads(candidate)
            except json.JSONDecodeError as exc:
                last_error = exc
                continue
            if not isinstance(parsed, dict):
                raise SystemExit(f"{recipient} JSON artifact must be an object: {parsed!r}")
            return parsed
        raise SystemExit(f"{recipient} did not return a parseable JSON artifact: {last_error}: {output!r}")

    def require_list(value, field, recipient):
        if not isinstance(value, list) or len(value) < 3 or not all(isinstance(item, str) and item.strip() for item in value):
            raise SystemExit(f"{recipient} field {field!r} must be a list of at least three non-empty strings")

    red_artifact = parse_json_object("red-team")
    if red_artifact.get("artifact_type") != "red_test_plan":
        raise SystemExit(f"red-team artifact_type mismatch: {red_artifact!r}")
    if red_artifact.get("example") != example:
        raise SystemExit(f"red-team example mismatch: {red_artifact!r}")
    if red_artifact.get("implementation_visible") is not False:
        raise SystemExit(f"red-team must declare implementation_visible=false: {red_artifact!r}")
    require_list(red_artifact.get("test_categories"), "test_categories", "red-team")
    if not isinstance(red_artifact.get("oracle_strategy"), str) or not red_artifact["oracle_strategy"].strip():
        raise SystemExit(f"red-team oracle_strategy must be a non-empty string: {red_artifact!r}")

    green_artifact = parse_json_object("green-team")
    if green_artifact.get("artifact_type") != "green_implementation_plan":
        raise SystemExit(f"green-team artifact_type mismatch: {green_artifact!r}")
    if green_artifact.get("example") != example:
        raise SystemExit(f"green-team example mismatch: {green_artifact!r}")
    if green_artifact.get("saw_red_tests") is not False:
        raise SystemExit(f"green-team must declare saw_red_tests=false: {green_artifact!r}")
    if green_artifact.get("permitted_feedback") != "PASS_FAIL_ONLY":
        raise SystemExit(f"green-team permitted_feedback mismatch: {green_artifact!r}")
    require_list(green_artifact.get("implementation_steps"), "implementation_steps", "green-team")

    leaked_samples = [
        "CandidateSet { bits: 0x3FE }",
        "fn initialize_candidates(board: &Board)",
        "const MOVE_R_PERM: [usize; 54]",
        "fn solve_kociemba(cube: &Cube)",
        "struct Zobrist { piece: [[u64; 64]; 12] }",
        "fn generate_legal_moves(board: &Board) -> Vec<Move>",
        "assert_eq!(stdout_str(&output), EASY_SOLUTION)",
        "fn test_rejects_duplicate_in_row()",
        "assert_eq!(apply_moves(SOLVED, \"R U R' U'\"), GOLDEN_RURU)",
        "fn test_superflip_scramble_solves_and_verifies()",
        "assert_eq!(run_perft(4, KIWIPETE_FEN), 4085603)",
        "fn test_uci_position_startpos_moves()",
        "expected solved grid, got empty stdout",
        "thread 'test_accepts_dots_as_blanks' panicked",
        "expected Kociemba golden facelet string",
        "solution did not return cube to solved state",
        "expected perft leaf-node count 4085603",
        "uci session did not emit bestmove",
    ]
    for recipient in ["red-team", "green-team"]:
        output = by_recipient[recipient].get("output", "")
        for sample in leaked_samples:
            if sample in output:
                raise SystemExit(f"{recipient} output leaked withheld sample: {sample!r}")

    phase_artifact_dir.mkdir(parents=True, exist_ok=True)
    (phase_artifact_dir / "red-team-test-plan.json").write_text(
        json.dumps(red_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (phase_artifact_dir / "green-team-implementation-plan.json").write_text(
        json.dumps(green_artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

if not orchestrator_model:
    orchestrator_model = "unknown-inherited-model"

red_model = by_recipient["red-team"]["actualModel"]
green_model = by_recipient["green-team"]["actualModel"]
if requires_distinct_model_lanes and red_model == green_model:
    raise SystemExit(f"requires distinct red/green model lanes, but both used {red_model!r}")

toon_path.write_text(
    "schema_version: foundry.behavioral-smoke.v1\n"
    "run_id: pi-live-dispatch-smoke\n"
    "requires_divergence_restart: false\n"
    f"requires_distinct_model_lanes: {str(requires_distinct_model_lanes).lower()}\n"
    "\n"
    "test_results[1]{example,passed,total,expected_passed,expected_total}:\n"
    f"  {example},{expected_passed},{expected_total},{expected_passed},{expected_total}\n"
    "\n"
    "model_lanes[3]{recipient,planned_model,actual_model}:\n"
    f"  red-team,{red_model},{red_model}\n"
    f"  green-team,{green_model},{green_model}\n"
    f"  orchestrator,{orchestrator_model},{orchestrator_model}\n"
    "\n"
    "divergence_restarts[0]{phase,outcome,revision_history_count}:\n",
    encoding="utf-8",
)
PY

echo "Validating behavioral smoke artifacts..."
"$BEHAVIORAL_SMOKE" "$RUN_DIR"

if [ "$EXPLICIT_RUN_DIR" -eq 1 ] || [ "$KEEP" -eq 1 ]; then
  echo "pi-live-dispatch-smoke: PASS — artifacts kept at $RUN_DIR"
else
  echo "pi-live-dispatch-smoke: PASS"
fi
