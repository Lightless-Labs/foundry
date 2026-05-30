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
ARBITER="docs/playbooks/foundry-adversarial-arbiter-routing.md"
BARRIER_VALIDATOR="tests/validate-barrier-envelopes.sh"
ARBITER_EVALS="tests/arbiter-routing-evals.sh"
ARBITER_EVAL_FEATURE="tests/fixtures/arbiter-routing-evals.feature"
GENERIC_EVALS="tests/foundry-evals.sh"
GENERIC_EVAL_RUNNER="tests/evals/runner.py"
GENERIC_ARBITER_FEATURE="tests/evals/features/arbiter-routing.feature"
GREEN_FOLLOWUP_FEATURE="tests/evals/features/green-followup-barrier.feature"
DIVERGENCE_EVAL_ADAPTER="tests/evals/adapters/divergence_routing.py"
DIVERGENCE_EVAL_FEATURE="tests/evals/features/divergence-routing.feature"
RED_FOLLOWUP_EVAL_ADAPTER="tests/evals/adapters/red_followup_barrier.py"
RED_FOLLOWUP_EVAL_FEATURE="tests/evals/features/red-followup-barrier.feature"
SPEC_RESTART_EVAL_ADAPTER="tests/evals/adapters/spec_update_restart.py"
SPEC_RESTART_EVAL_FEATURE="tests/evals/features/spec-update-restart.feature"
REVIEWER_FANOUT_EVAL_ADAPTER="tests/evals/adapters/reviewer_fanout.py"
REVIEWER_FANOUT_EVAL_FEATURE="tests/evals/features/reviewer-fanout.feature"
PHASE_CHOREOGRAPHY_EVAL_ADAPTER="tests/evals/adapters/phase_choreography.py"
PHASE_CHOREOGRAPHY_EVAL_FEATURE="tests/evals/features/phase-choreography.feature"

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
require_file "arbiter-routing-playbook" "$ARBITER"
require_file "barrier-envelope-validator" "$BARRIER_VALIDATOR"
require_file "arbiter-eval-runner" "$ARBITER_EVALS"
require_file "arbiter-eval-feature" "$ARBITER_EVAL_FEATURE"
require_file "generic-eval-entrypoint" "$GENERIC_EVALS"
require_file "generic-eval-runner" "$GENERIC_EVAL_RUNNER"
require_file "generic-arbiter-eval-feature" "$GENERIC_ARBITER_FEATURE"
require_file "green-followup-eval-feature" "$GREEN_FOLLOWUP_FEATURE"
require_file "divergence-eval-adapter" "$DIVERGENCE_EVAL_ADAPTER"
require_file "divergence-eval-feature" "$DIVERGENCE_EVAL_FEATURE"
require_file "red-followup-eval-adapter" "$RED_FOLLOWUP_EVAL_ADAPTER"
require_file "red-followup-eval-feature" "$RED_FOLLOWUP_EVAL_FEATURE"
require_file "spec-restart-eval-adapter" "$SPEC_RESTART_EVAL_ADAPTER"
require_file "spec-restart-eval-feature" "$SPEC_RESTART_EVAL_FEATURE"
require_file "reviewer-fanout-eval-adapter" "$REVIEWER_FANOUT_EVAL_ADAPTER"
require_file "reviewer-fanout-eval-feature" "$REVIEWER_FANOUT_EVAL_FEATURE"
require_file "phase-choreography-eval-adapter" "$PHASE_CHOREOGRAPHY_EVAL_ADAPTER"
require_file "phase-choreography-eval-feature" "$PHASE_CHOREOGRAPHY_EVAL_FEATURE"

require_literal "skill-references-divergence-playbook" "$SKILL" "$DIVERGENCE"
require_literal "skill-references-restart-playbook" "$SKILL" "$RESTART"
require_literal "skill-references-troubleshooting-playbook" "$SKILL" "$TROUBLE"
require_literal "skill-references-pi-continuation-playbook" "$SKILL" "$PI_CONTINUATION"
require_literal "skill-references-arbiter-playbook" "$SKILL" "$ARBITER"
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
require_literal "troubleshooting-arbiter-playbook" "$TROUBLE" "$ARBITER"

require_literal "arbiter-agent-route" "$ARBITER" 'foundry:review:arbiter-agent'
require_literal "arbiter-single-test-scope" "$ARBITER" "one disputed test"
require_literal "arbiter-test-wrong" "$ARBITER" "TEST_WRONG"
require_literal "arbiter-implementation-wrong" "$ARBITER" "IMPLEMENTATION_WRONG"
require_literal "arbiter-spec-incomplete" "$ARBITER" "SPEC_INCOMPLETE"
require_literal "arbiter-green-barrier" "$ARBITER" 'test_name: PASS/FAIL'
require_literal "arbiter-validator-recipient" "$BARRIER_VALIDATOR" "ARBITER_RECIPIENT_RE"
require_literal "arbiter-validator-scope" "$BARRIER_VALIDATOR" "single_test_scope"
require_literal "arbiter-validator-overbroad" "$BARRIER_VALIDATOR" "ARBITER_OVERBROAD_VISIBLE_CONTEXT"
require_literal "arbiter-eval-gherkin" "$ARBITER_EVAL_FEATURE" "Scenario Outline: Mocked arbiter output routes one disputed test"
require_literal "arbiter-eval-test-wrong" "$ARBITER_EVAL_FEATURE" "TEST_WRONG"
require_literal "arbiter-eval-implementation-wrong" "$ARBITER_EVAL_FEATURE" "IMPLEMENTATION_WRONG"
require_literal "arbiter-eval-spec-incomplete" "$ARBITER_EVAL_FEATURE" "SPEC_INCOMPLETE"
require_literal "arbiter-eval-route-map" "$ARBITER_EVALS" "EXPECTED_ROUTE"
require_literal "arbiter-eval-barrier-validator" "$ARBITER_EVALS" "validate-barrier-envelopes.sh"
require_literal "generic-eval-suite-flag" "$GENERIC_EVALS" "--suite arbiter-routing"
require_literal "generic-eval-adapter-dispatch" "$GENERIC_EVAL_RUNNER" "adapter_for"
require_literal "generic-arbiter-eval-gherkin" "$GENERIC_ARBITER_FEATURE" "Scenario Outline: Mocked arbiter output routes one disputed test"
require_literal "green-followup-barrier-gherkin" "$GREEN_FOLLOWUP_FEATURE" "Green follow-up preserves the information barrier"
require_literal "divergence-eval-gherkin" "$DIVERGENCE_EVAL_FEATURE" "Mocked divergence output routes one divergence"
require_literal "divergence-eval-phase1b" "$DIVERGENCE_EVAL_FEATURE" "PHASE_1B"
require_literal "divergence-eval-phase2b" "$DIVERGENCE_EVAL_FEATURE" "PHASE_2B"
require_literal "divergence-eval-findings-route" "$DIVERGENCE_EVAL_ADAPTER" "findings[0].outcome"
require_literal "divergence-eval-spec-restart" "$DIVERGENCE_EVAL_ADAPTER" "spec_update_and_restart"
require_literal "divergence-eval-tracker-reset" "$DIVERGENCE_EVAL_ADAPTER" "consecutive_fails"
require_literal "red-followup-eval-gherkin" "$RED_FOLLOWUP_EVAL_FEATURE" "Red follow-up preserves the information barrier"
require_literal "red-followup-eval-implementation-withheld" "$RED_FOLLOWUP_EVAL_ADAPTER" "implementation_code"
require_literal "spec-restart-eval-gherkin" "$SPEC_RESTART_EVAL_FEATURE" "Spec update and restart preserves provenance"
require_literal "spec-restart-eval-nlspec-rerun" "$SPEC_RESTART_EVAL_ADAPTER" "NLSpecRerunInput"
require_literal "spec-restart-eval-gap-verbatim" "$SPEC_RESTART_EVAL_ADAPTER" "gap_description_verbatim"
require_literal "spec-restart-eval-tracker-reset" "$SPEC_RESTART_EVAL_ADAPTER" "reset_all_counters"
require_literal "spec-restart-eval-revision-cap" "$SPEC_RESTART_EVAL_ADAPTER" "revision_cap_reached"
require_literal "reviewer-fanout-eval-gherkin" "$REVIEWER_FANOUT_EVAL_FEATURE" "Phase 3 reviewer fan-out is complete and barrier-safe"
require_literal "reviewer-fanout-eval-green-reviewer" "$REVIEWER_FANOUT_EVAL_ADAPTER" "green-team-reviewer"
require_literal "reviewer-fanout-eval-conditional-bazel" "$REVIEWER_FANOUT_EVAL_ADAPTER" "has_build"
require_literal "reviewer-fanout-eval-territory" "$REVIEWER_FANOUT_EVAL_ADAPTER" "IMPLEMENTATION_FACING"
require_literal "phase-choreography-eval-gherkin" "$PHASE_CHOREOGRAPHY_EVAL_FEATURE" "Mocked adversarial run follows the expected phase choreography"
require_literal "phase-choreography-eval-behavioral-smoke" "$PHASE_CHOREOGRAPHY_EVAL_ADAPTER" "behavioral-smoke.sh"
require_literal "phase-choreography-eval-restart" "$PHASE_CHOREOGRAPHY_EVAL_ADAPTER" "phase1b_valuable_restart"
require_literal "phase-choreography-eval-reviewer-reject" "$PHASE_CHOREOGRAPHY_EVAL_ADAPTER" "phase3_green_reject_then_fix"
require_literal "phase-choreography-eval-tracker-reset" "$PHASE_CHOREOGRAPHY_EVAL_ADAPTER" "reset_all_counters"

require_literal "pi-continuation-foundry-team" "$PI_CONTINUATION" "foundry_team"
require_literal "pi-continuation-pass-fail-labels" "$PI_CONTINUATION" "PASS/FAIL outcome labels"
require_literal "pi-continuation-no-outcome-label-samples" "$PI_CONTINUATION" "Do **not** use PASS/FAIL test outcome labels"
require_literal "pi-continuation-final-validators" "$PI_CONTINUATION" "tests/behavioral-smoke.sh runs/<run_id>"

TOTAL_COUNT=$((PASS_COUNT + FAIL_COUNT))
printf '\nTOTAL: %d passed, %d failed out of %d adversarial module checks\n' "$PASS_COUNT" "$FAIL_COUNT" "$TOTAL_COUNT"

if [ "$FAIL_COUNT" -ne 0 ]; then
  exit 1
fi
