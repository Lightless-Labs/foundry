#!/usr/bin/env bash
# validate-public-plugin.sh — fast validation entrypoint for the Foundry public plugin repo.
#
# This intentionally excludes slow/live model lanes such as tests/pi-live-dispatch-smoke.sh.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

run_step() {
  local name="$1"
  shift
  printf '\n═══ %s ═══\n' "$name"
  "$@"
}

run_step "agent prompt structure" tests/validate-agents.sh
run_step "PromptEnvelope barrier self-tests" tests/validate-barrier-envelopes.sh
run_step "behavioral smoke contract" tests/validate-behavioral-smoke-contract.sh
run_step "Pi extension package" tests/validate-pi-extension.sh
run_step "Codex plugin package" tests/validate-codex-plugin.sh
run_step "adversarial skill modules" tests/validate-adversarial-modules.sh
run_step "workflow evals" tests/foundry-evals.sh
run_step "adversarial UI capture surfaces" tests/validate-adversarial-ui-capture-surfaces.sh
run_step "adversarial UI visual controls" tests/validate-adversarial-ui-visual-controls.sh

printf '\nAll fast public-plugin validation checks passed.\n'
