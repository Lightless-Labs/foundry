#!/usr/bin/env bash
# validate-adversarial-modules.sh — structural checks for extracted foundry-adversarial playbooks.
set -euo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SKILL="plugins/foundry/skills/foundry-adversarial/SKILL.md"
DIVERGENCE="docs/playbooks/foundry-adversarial-divergence-routing.md"
RESTART="docs/playbooks/foundry-adversarial-spec-update-and-restart.md"
TROUBLE="docs/playbooks/foundry-adversarial-provider-troubleshooting.md"
PI_CONTINUATION="docs/playbooks/foundry-adversarial-pi-continuation.md"

pass() {
  echo "$1: PASS"
  PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
  echo "$1: FAIL — $2"
  FAIL_COUNT=$((FAIL_COUNT + 1))
}

require_file() {
  local name="$1" path="$2"
  if [ -f "$path" ]; then pass "$name"; else fail "$name" "missing file $path"; fi
}

require_literal() {
  local name="$1" path="$2" literal="$3"
  if grep -Fq -- "$literal" "$path"; then
    pass "$name"
  else
    fail "$name" "missing literal in $path: $literal"
  fi
}

require_file "adversarial-skill" "$SKILL"
require_file "divergence-playbook" "$DIVERGENCE"
require_file "restart-playbook" "$RESTART"
require_file "provider-troubleshooting-playbook" "$TROUBLE"
require_file "pi-continuation-playbook" "$PI_CONTINUATION"

require_literal "skill-references-divergence-playbook" "$SKILL" "$DIVERGENCE"
require_literal "skill-references-restart-playbook" "$SKILL" "$RESTART"
require_literal "skill-references-troubleshooting-playbook" "$SKILL" "$TROUBLE"
require_literal "skill-references-pi-continuation-playbook" "$SKILL" "$PI_CONTINUATION"
require_literal "skill-keeps-phase2b-valuable-anchor" "$SKILL" 'Phase 2b `VALUABLE`'
require_literal "skill-keeps-findings-outcome" "$SKILL" "findings[0].outcome"
require_literal "skill-keeps-spec-update-name" "$SKILL" "spec_update_and_restart"
require_literal "skill-keeps-behavioral-smoke" "$SKILL" "runs/<run_id>/behavioral-smoke.toon"
require_literal "skill-keeps-barrier-green-labels" "$SKILL" "test_name: PASS/FAIL"

require_literal "divergence-findings-outcome" "$DIVERGENCE" "findings[0].outcome"
require_literal "divergence-phase1b-valuable" "$DIVERGENCE" 'Phase 1b `VALUABLE`'
require_literal "divergence-phase2b-valuable" "$DIVERGENCE" 'Phase 2b `VALUABLE`'
require_literal "divergence-red-test-paths" "$DIVERGENCE" "red_test_paths"
require_literal "divergence-one-at-a-time" "$DIVERGENCE" "one at a time"
require_literal "divergence-behavioral-contract" "$DIVERGENCE" 'revision_history_count` exactly `1`'

require_literal "restart-hard-rule" "$RESTART" "MUST NOT write NLSpec content directly"
require_literal "restart-gap-description" "$RESTART" "findings[0].gap_description"
require_literal "restart-deferred-commit-guard" "$RESTART" "git diff --staged --quiet"
require_literal "restart-phase1-package" "$RESTART" "Phase1RestartPackage"
require_literal "restart-revision-history" "$RESTART" "revision_history"
require_literal "restart-red-test-paths" "$RESTART" "red_test_paths"

require_literal "troubleshooting-opencode" "$TROUBLE" "OpenCode"
require_literal "troubleshooting-kimi" "$TROUBLE" "Kimi K2.5"
require_literal "troubleshooting-no-raw-failures" "$TROUBLE" "Do not reveal assertions, stack traces, raw outputs"
require_literal "pi-continuation-foundry-team" "$PI_CONTINUATION" "foundry_team"
require_literal "pi-continuation-pass-fail-labels" "$PI_CONTINUATION" "PASS/FAIL outcome labels"
require_literal "pi-continuation-no-outcome-label-samples" "$PI_CONTINUATION" "Do **not** use PASS/FAIL test outcome labels"
require_literal "pi-continuation-final-validators" "$PI_CONTINUATION" "tests/behavioral-smoke.sh runs/<run_id>"

TOTAL_COUNT=$((PASS_COUNT + FAIL_COUNT))
printf '\nTOTAL: %d passed, %d failed out of %d adversarial module checks\n' "$PASS_COUNT" "$FAIL_COUNT" "$TOTAL_COUNT"

if [ "$FAIL_COUNT" -ne 0 ]; then
  exit 1
fi
