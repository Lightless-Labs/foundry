#!/usr/bin/env bash
# validate-behavioral-smoke-contract.sh — ensure the adversarial skill emits replayable behavioral-smoke artifacts.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILL="$ROOT_DIR/plugins/foundry/skills/foundry-adversarial/SKILL.md"
PASS_COUNT=0
FAIL_COUNT=0

pass() {
  echo "$1: PASS"
  PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
  echo "$1: FAIL — $2"
  FAIL_COUNT=$((FAIL_COUNT + 1))
}

require_literal() {
  local name="$1"
  local literal="$2"
  if grep -Fq "$literal" "$SKILL"; then
    pass "$name"
  else
    fail "$name" "missing literal: $literal"
  fi
}

require_literal "behavioral-summary-path" 'runs/<run_id>/behavioral-smoke.toon'
require_literal "behavioral-schema" 'foundry.behavioral-smoke.v1'
require_literal "test-results-table" 'test_results[1]{example,passed,total,expected_passed,expected_total}'
require_literal "model-lanes-table" 'model_lanes[3]{recipient,planned_model,actual_model}'
require_literal "divergence-restarts-table" 'divergence_restarts[0]{phase,outcome,revision_history_count}'
require_literal "final-behavioral-validator" 'tests/behavioral-smoke.sh runs/<run_id>'
require_literal "final-barrier-validator" 'tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch'

TOTAL_COUNT=$((PASS_COUNT + FAIL_COUNT))
printf '\nTOTAL: %d passed, %d failed out of %d behavioral-smoke contract checks\n' "$PASS_COUNT" "$FAIL_COUNT" "$TOTAL_COUNT"

if [ "$FAIL_COUNT" -ne 0 ]; then
  exit 1
fi
