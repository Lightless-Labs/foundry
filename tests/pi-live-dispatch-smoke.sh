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
#   tests/pi-live-dispatch-smoke.sh --run-dir runs/manual-pi-live-smoke
#   tests/pi-live-dispatch-smoke.sh --red-model openai-codex/gpt-5.5:xhigh --green-model kimi/k2.5 --require-distinct-model-lanes --keep
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXTENSION="$ROOT_DIR/extensions/pi-foundry-team/index.ts"
BEHAVIORAL_SMOKE="$ROOT_DIR/tests/behavioral-smoke.sh"
SUDOKU_DIR="$ROOT_DIR/examples/sudoku-solver"

KEEP=0
RUN_DIR=""
EXPLICIT_RUN_DIR=0
RED_MODEL="${FOUNDRY_RED_MODEL:-}"
GREEN_MODEL="${FOUNDRY_GREEN_MODEL:-}"
REQUIRE_DISTINCT_MODEL_LANES=0

usage() {
  cat <<'USAGE'
Usage: tests/pi-live-dispatch-smoke.sh [--keep] [--run-dir DIR] [--red-model MODEL] [--green-model MODEL] [--require-distinct-model-lanes]

Runs a slow/manual live Pi dispatch smoke using the Foundry foundry_team
extension. By default artifacts are written to a temporary directory and cleaned
up on success. Use --keep or --run-dir to inspect the generated run artifacts.

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

if ! command -v pi >/dev/null 2>&1; then
  echo "pi-live-dispatch-smoke: FAIL — pi command not found" >&2
  exit 2
fi

if [ ! -f "$EXTENSION" ]; then
  echo "pi-live-dispatch-smoke: FAIL — missing extension $EXTENSION" >&2
  exit 1
fi

TMP_ROOT=""
if [ -z "$RUN_DIR" ]; then
  TMP_ROOT="$(mktemp -d)"
  RUN_DIR="$TMP_ROOT/runs/pi-live-dispatch-smoke"
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

cat >"$RUN_DIR/dispatch/phase1/red-team.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "pi-live-dispatch-smoke",
  "phase": "phase1",
  "recipient": "red-team",
  "prompt": "You are the RED TEAM in a Foundry Pi live dispatch smoke. You can see the full Sudoku solver NLSpec summary and public product contract. You cannot see implementation details. Reply exactly: RED_OK",
  "visible_context": [
    {"label": "full_nlspec", "kind": "nlspec", "sha256": "live-smoke", "content": "Sudoku solver CLI: parse 81 cells, accept 0 or dot blanks, validate rows/columns/boxes, solve by constraint propagation plus backtracking, print a 9-line solved board, and report invalid/unsolvable inputs with non-zero exits."},
    {"label": "spec", "kind": "spec", "sha256": "live-smoke", "content": "Worked example: examples/sudoku-solver. Red tests are black-box CLI tests derived from the NLSpec Definition of Done."}
  ],
  "withheld_context": [
    {"label": "implementation_workspace", "kind": "implementation_code", "sha256": "live-smoke", "samples": ["CandidateSet { bits: 0x3FE }", "fn initialize_candidates(board: &Board)"]}
  ],
  "redactions": []
}
JSON

cat >"$RUN_DIR/dispatch/phase2/green-team.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "pi-live-dispatch-smoke",
  "phase": "phase2",
  "recipient": "green-team",
  "prompt": "You are the GREEN TEAM in a Foundry Pi live dispatch smoke.\n\nNLSpec How section: parse an 81-cell Sudoku puzzle; strip whitespace; accept 0 or dot blanks; validate duplicate givens; initialize candidate sets; propagate naked and hidden singles; backtrack with a minimum-remaining-values choice; print one 9-digit row per line.\n\nTest results:\n  sudoku_red_tests: PASS\n  pi_foundry_team_dispatch_smoke: PASS\n\n## Task\nReply exactly: GREEN_OK",
  "visible_context": [
    {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "live-smoke", "content": "parse an 81-cell Sudoku puzzle; strip whitespace; accept 0 or dot blanks; validate duplicate givens; initialize candidate sets; propagate naked and hidden singles; backtrack with a minimum-remaining-values choice; print one 9-digit row per line"},
    {"label": "outcome_labels", "kind": "test_outcomes", "sha256": "live-smoke", "content": "sudoku_red_tests: PASS\npi_foundry_team_dispatch_smoke: PASS"}
  ],
  "withheld_context": [
    {"label": "red_test_code", "kind": "red_test_code", "sha256": "live-smoke", "samples": ["assert_eq!(stdout_str(&output), EASY_SOLUTION)", "fn test_rejects_duplicate_in_row()"]},
    {"label": "nlspec_done", "kind": "nlspec_done", "sha256": "live-smoke", "samples": ["6.8 Backtracking / Hard Puzzles", "Definition of Done item: rejects invalid characters"]},
    {"label": "raw_failure", "kind": "raw_test_output", "sha256": "live-smoke", "samples": ["expected solved grid, got empty stdout", "thread 'test_accepts_dots_as_blanks' panicked"]}
  ],
  "redactions": [
    {"source": "raw_test_output", "action": "pass_fail_labels_only", "removed": ["assertion_text", "expected_values", "stack_traces", "line_numbers"]}
  ]
}
JSON

echo "Running Sudoku worked-example red tests (expected 30/30)..."
(
  cd "$SUDOKU_DIR"
  cargo test --quiet
) >"$RUN_DIR/sudoku-cargo-test.out" 2>&1

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
  --mode json \
  -p \
  --no-session \
  --no-context-files \
  --no-skills \
  --tools foundry_team \
  "$PI_PROMPT" >"$RUN_DIR/pi-foundry-team.jsonl"

python3 - "$RUN_DIR/pi-foundry-team.jsonl" "$RUN_DIR/behavioral-smoke.toon" "$RED_MODEL" "$GREEN_MODEL" "$REQUIRE_DISTINCT_MODEL_LANES" <<'PY'
import json
import sys
from pathlib import Path

jsonl_path = Path(sys.argv[1])
toon_path = Path(sys.argv[2])
expected_models = {
    "red-team": sys.argv[3] or None,
    "green-team": sys.argv[4] or None,
}
requires_distinct_model_lanes = sys.argv[5] == "1" or bool(expected_models["red-team"] and expected_models["green-team"] and expected_models["red-team"] != expected_models["green-team"])

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
for recipient, expected_output in [("red-team", "RED_OK"), ("green-team", "GREEN_OK")]:
    result = by_recipient.get(recipient)
    if not result:
        raise SystemExit(f"missing foundry_team result for {recipient}")
    if result.get("exitCode") != 0:
        raise SystemExit(f"{recipient} exitCode={result.get('exitCode')}: {result}")
    if result.get("output", "").strip() != expected_output:
        raise SystemExit(f"{recipient} output mismatch: {result.get('output')!r}")
    if not result.get("actualModel"):
        raise SystemExit(f"{recipient} missing actualModel in tool result")
    expected_model = expected_models[recipient]
    if expected_model and result.get("plannedModel") != expected_model:
        raise SystemExit(f"{recipient} plannedModel mismatch: expected {expected_model!r}, got {result.get('plannedModel')!r}")
    if expected_model and result.get("actualModel") != expected_model:
        raise SystemExit(f"{recipient} actualModel mismatch: expected {expected_model!r}, got {result.get('actualModel')!r}")

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
    "  sudoku-solver,30,30,30,30\n"
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
