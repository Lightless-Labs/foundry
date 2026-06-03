#!/usr/bin/env bash
# behavioral-smoke.sh — replay-level behavioral smoke tests for Foundry runs.
#
# Usage:
#   tests/behavioral-smoke.sh                 # run built-in self-tests
#   tests/behavioral-smoke.sh runs/<run_id>   # validate one or more run dirs
#
# Expected run-dir shape:
#   runs/<run_id>/dispatch/**/*.json          # PromptEnvelope v1 artifacts
#   runs/<run_id>/behavioral-smoke.toon       # foundry.behavioral-smoke.v1 summary
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BARRIER_VALIDATOR="$ROOT_DIR/tests/validate-barrier-envelopes.sh"

validate_targets() {
  if [ "$#" -eq 0 ]; then
    echo "No run directories supplied" >&2
    return 2
  fi

  local run_dir
  for run_dir in "$@"; do
    if [ ! -d "$run_dir" ]; then
      echo "$run_dir: FAIL — run directory not found" >&2
      return 1
    fi
    if [ ! -d "$run_dir/dispatch" ]; then
      echo "$run_dir: FAIL — missing dispatch/ PromptEnvelope directory" >&2
      return 1
    fi
    "$BARRIER_VALIDATOR" "$run_dir/dispatch"
  done

  python3 - "$@" <<'PY'
import csv
import re
import sys
from pathlib import Path

EXPECTED_SCHEMA = "foundry.behavioral-smoke.v1"
REQUIRED_TABLES = {"test_results", "model_lanes"}
OPTIONAL_TABLES = {"divergence_restarts"}
TABLE_FIELDS = {
    "test_results": ["example", "passed", "total", "expected_passed", "expected_total"],
    "model_lanes": ["recipient", "planned_model", "actual_model"],
    "divergence_restarts": ["phase", "outcome", "revision_history_count"],
}
TABLE_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_-]*)\[(\d+)\]\{([^}]*)\}:\s*$")
SCALAR_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_-]*):\s*(.*)$")


def fail(path, message):
    print(f"{path}: FAIL — {message}")
    return False


def pass_(path):
    print(f"{path}: PASS")
    return True


def parse_value(raw):
    value = raw.strip()
    if value == "":
        return ""
    if value.lower() == "true":
        return True
    if value.lower() == "false":
        return False
    if value.lower() == "null":
        return None
    if re.fullmatch(r"-?\d+", value):
        return int(value)
    return value


def parse_csv_row(raw):
    try:
        return next(csv.reader([raw], skipinitialspace=False))
    except Exception as exc:
        raise ValueError(f"invalid CSV-style row {raw!r}: {exc}") from exc


def parse_smoke_toon(path):
    lines = path.read_text(encoding="utf-8").splitlines()
    data = {"_tables": {}}
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            i += 1
            continue
        if line[:1].isspace():
            raise ValueError(f"unexpected indented row without table header on line {i + 1}")

        table_match = TABLE_RE.match(line)
        if table_match:
            name = table_match.group(1)
            expected_count = int(table_match.group(2))
            fields = [field.strip() for field in table_match.group(3).split(",") if field.strip()]
            rows = []
            i += 1
            while i < len(lines):
                row_line = lines[i]
                row_stripped = row_line.strip()
                if not row_stripped or row_stripped.startswith("#"):
                    i += 1
                    continue
                if not row_line[:1].isspace():
                    break
                values = [parse_value(value) for value in parse_csv_row(row_stripped)]
                if len(values) != len(fields):
                    raise ValueError(
                        f"table {name!r} row {len(rows) + 1} has {len(values)} values, expected {len(fields)}"
                    )
                rows.append(dict(zip(fields, values)))
                i += 1
            if len(rows) != expected_count:
                raise ValueError(
                    f"table {name!r} declares {expected_count} rows but contains {len(rows)}"
                )
            data["_tables"][name] = {"fields": fields, "rows": rows}
            continue

        scalar_match = SCALAR_RE.match(line)
        if scalar_match:
            data[scalar_match.group(1)] = parse_value(scalar_match.group(2))
            i += 1
            continue

        raise ValueError(f"unrecognized TOON subset syntax on line {i + 1}: {line!r}")
    return data


def require_fields(path, table_name, table):
    expected = TABLE_FIELDS[table_name]
    actual = table.get("fields", [])
    if actual != expected:
        return fail(path, f"table {table_name!r} fields must be {expected}, got {actual}")
    return True


def validate_manifest(path):
    try:
        data = parse_smoke_toon(path)
    except Exception as exc:
        return fail(path, f"invalid behavioral-smoke TOON: {exc}")

    if data.get("schema_version") != EXPECTED_SCHEMA:
        return fail(path, f"schema_version must be {EXPECTED_SCHEMA!r}")
    if not isinstance(data.get("run_id"), str) or not data["run_id"].strip():
        return fail(path, "run_id must be a non-empty string")

    tables = data.get("_tables", {})
    missing = sorted(REQUIRED_TABLES - set(tables))
    if missing:
        return fail(path, f"missing required tables: {', '.join(missing)}")

    ok = True
    for table_name in sorted((REQUIRED_TABLES | OPTIONAL_TABLES) & set(tables)):
        ok = require_fields(path, table_name, tables[table_name]) and ok
    if not ok:
        return False

    if not tables["test_results"]["rows"]:
        return fail(path, "test_results must contain at least one worked-example row")

    for row in tables["test_results"]["rows"]:
        if row["passed"] != row["expected_passed"] or row["total"] != row["expected_total"]:
            return fail(
                path,
                f"example {row['example']!r} expected {row['expected_passed']}/{row['expected_total']} "
                f"but saw {row['passed']}/{row['total']}",
            )

    model_rows = tables["model_lanes"]["rows"]
    for row in model_rows:
        if row["planned_model"] != row["actual_model"]:
            return fail(
                path,
                f"model lane mismatch for {row['recipient']!r}: planned {row['planned_model']!r}, "
                f"actual {row['actual_model']!r}",
            )

    if data.get("requires_distinct_model_lanes") is True:
        by_recipient = {str(row["recipient"]): row for row in model_rows}
        missing = [recipient for recipient in ("red-team", "green-team") if recipient not in by_recipient]
        if missing:
            return fail(path, f"requires_distinct_model_lanes=true but missing model lanes: {', '.join(missing)}")
        red = by_recipient["red-team"]
        green = by_recipient["green-team"]
        if red["planned_model"] == green["planned_model"] or red["actual_model"] == green["actual_model"]:
            return fail(
                path,
                "requires_distinct_model_lanes=true but red-team and green-team use the same model lane",
            )

    divergence_rows = tables.get("divergence_restarts", {}).get("rows", [])
    if data.get("requires_divergence_restart") is True:
        valuable = [row for row in divergence_rows if row["outcome"] == "VALUABLE"]
        if not valuable:
            return fail(path, "requires_divergence_restart=true but no VALUABLE divergence restart row exists")

    for row in divergence_rows:
        if row["outcome"] == "VALUABLE" and row["revision_history_count"] != 1:
            return fail(
                path,
                f"{row['phase']} VALUABLE restart must have exactly one revision-history entry, "
                f"got {row['revision_history_count']}",
            )

    return pass_(path)


def main(argv):
    ok = True
    for raw in argv:
        run_dir = Path(raw)
        manifest = run_dir / "behavioral-smoke.toon"
        if not manifest.exists():
            print(f"{manifest}: FAIL — missing behavioral-smoke.toon manifest")
            ok = False
            continue
        ok = validate_manifest(manifest) and ok
    return 0 if ok else 1

sys.exit(main(sys.argv[1:]))
PY
}

write_good_run() {
  local run_dir="$1"
  mkdir -p "$run_dir/dispatch/phase1" "$run_dir/dispatch/phase2"

  cat >"$run_dir/dispatch/phase1/red-team.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "behavioral-selftest",
  "phase": "phase1",
  "recipient": "red-team",
  "prompt": "You are the RED TEAM. Use the full NLSpec and product spec to write independent black-box tests. Do not inspect implementation workspaces.",
  "visible_context": [
    {"label": "full_nlspec", "kind": "nlspec", "sha256": "demo", "content": "Why/What/How/Done for a Sudoku solver"},
    {"label": "spec", "kind": "spec", "sha256": "demo", "content": "CLI accepts puzzles and prints solved grids"}
  ],
  "withheld_context": [
    {"label": "implementation_workspace", "kind": "implementation_code", "sha256": "demo", "samples": ["fn solve_with_hidden_singles", "struct CandidateGrid"]}
  ],
  "redactions": []
}
JSON

  cat >"$run_dir/dispatch/phase2/green-team.json" <<'JSON'
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "behavioral-selftest",
  "phase": "phase2",
  "recipient": "green-team",
  "prompt": "You are the GREEN TEAM.\n\nNLSpec How section: parse an 81-cell puzzle, propagate constraints, backtrack when necessary, print the solved grid.\n\nTest results:\n  test_accepts_argument: FAIL\n  test_rejects_duplicate_row: PASS\n\n## Task\nImplement against the How section using only these PASS/FAIL labels.",
  "visible_context": [
    {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "demo", "content": "parse puzzle; propagate constraints; backtrack; print solved grid"},
    {"label": "outcome_labels", "kind": "test_outcomes", "sha256": "demo", "content": "test_accepts_argument: FAIL\ntest_rejects_duplicate_row: PASS"}
  ],
  "withheld_context": [
    {"label": "red_test_code", "kind": "red_test_code", "sha256": "demo", "samples": ["Scenario: accepts dots as blanks", "assert_eq!(stdout.trim().len(), 81)"]},
    {"label": "nlspec_done", "kind": "nlspec_done", "sha256": "demo", "samples": ["Done item: reject duplicate rows with exit code 2"]},
    {"label": "raw_failure", "kind": "raw_test_output", "sha256": "demo", "samples": ["expected solved grid, got empty stdout"]}
  ],
  "redactions": [
    {"source": "raw_test_output", "action": "pass_fail_labels_only", "removed": ["assertion_text", "expected_values", "stack_traces"]}
  ]
}
JSON

  cat >"$run_dir/behavioral-smoke.toon" <<'TOON'
schema_version: foundry.behavioral-smoke.v1
run_id: behavioral-selftest
requires_divergence_restart: true
requires_distinct_model_lanes: true

test_results[2]{example,passed,total,expected_passed,expected_total}:
  sudoku-solver,30,30,30,30
  chess-engine,44,44,44,44

model_lanes[3]{recipient,planned_model,actual_model}:
  red-team,openai-codex/gpt-5.5:xhigh,openai-codex/gpt-5.5:xhigh
  green-team,kimi-coding/kimi-for-coding,kimi-coding/kimi-for-coding
  orchestrator,anthropic/claude-opus-4.7,anthropic/claude-opus-4.7

divergence_restarts[1]{phase,outcome,revision_history_count}:
  phase2b,VALUABLE,1
TOON
}

run_self_tests() {
  local tmp good bad_model bad_distinct bad_divergence bad_toon
  tmp="$(mktemp -d)"
  trap "rm -rf '$tmp'" EXIT

  good="$tmp/good-run"
  write_good_run "$good"
  echo "Self-test: good behavioral run should pass"
  validate_targets "$good"

  bad_model="$tmp/bad-model"
  cp -R "$good" "$bad_model"
  python3 - "$bad_model/behavioral-smoke.toon" <<'PY'
from pathlib import Path
path = Path(__import__('sys').argv[1])
text = path.read_text()
path.write_text(text.replace('kimi-coding/kimi-for-coding,kimi-coding/kimi-for-coding', 'kimi-coding/kimi-for-coding,anthropic/claude-opus-4.7'))
PY
  echo "Self-test: model-lane mismatch should fail"
  if validate_targets "$bad_model" >/tmp/foundry-behavioral-bad-model.out 2>&1; then
    cat /tmp/foundry-behavioral-bad-model.out
    echo "bad-model unexpectedly passed" >&2
    return 1
  fi
  cat /tmp/foundry-behavioral-bad-model.out

  bad_distinct="$tmp/bad-distinct"
  cp -R "$good" "$bad_distinct"
  python3 - "$bad_distinct/behavioral-smoke.toon" <<'PY'
from pathlib import Path
path = Path(__import__('sys').argv[1])
text = path.read_text()
path.write_text(text.replace('green-team,kimi-coding/kimi-for-coding,kimi-coding/kimi-for-coding', 'green-team,openai-codex/gpt-5.5:xhigh,openai-codex/gpt-5.5:xhigh'))
PY
  echo "Self-test: required distinct red/green model lanes should fail when lanes collapse"
  if validate_targets "$bad_distinct" >/tmp/foundry-behavioral-bad-distinct.out 2>&1; then
    cat /tmp/foundry-behavioral-bad-distinct.out
    echo "bad-distinct unexpectedly passed" >&2
    return 1
  fi
  cat /tmp/foundry-behavioral-bad-distinct.out

  bad_divergence="$tmp/bad-divergence"
  cp -R "$good" "$bad_divergence"
  python3 - "$bad_divergence/behavioral-smoke.toon" <<'PY'
from pathlib import Path
path = Path(__import__('sys').argv[1])
path.write_text(path.read_text().replace('phase2b,VALUABLE,1', 'phase2b,VALUABLE,2'))
PY
  echo "Self-test: bad divergence revision count should fail"
  if validate_targets "$bad_divergence" >/tmp/foundry-behavioral-bad-divergence.out 2>&1; then
    cat /tmp/foundry-behavioral-bad-divergence.out
    echo "bad-divergence unexpectedly passed" >&2
    return 1
  fi
  cat /tmp/foundry-behavioral-bad-divergence.out

  bad_toon="$tmp/bad-toon"
  cp -R "$good" "$bad_toon"
  python3 - "$bad_toon/behavioral-smoke.toon" <<'PY'
from pathlib import Path
path = Path(__import__('sys').argv[1])
path.write_text(path.read_text().replace('test_results[2]', 'test_results[3]'))
PY
  echo "Self-test: TOON declared row-count mismatch should fail"
  if validate_targets "$bad_toon" >/tmp/foundry-behavioral-bad-toon.out 2>&1; then
    cat /tmp/foundry-behavioral-bad-toon.out
    echo "bad-toon unexpectedly passed" >&2
    return 1
  fi
  cat /tmp/foundry-behavioral-bad-toon.out

  echo "Behavioral smoke self-tests: PASS"
}

if [ "$#" -eq 0 ]; then
  run_self_tests
else
  validate_targets "$@"
fi
