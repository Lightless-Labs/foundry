#!/usr/bin/env bash
# foundry-evals.sh — deterministic Gherkin-authored process evals for Foundry.
#
# Usage:
#   tests/foundry-evals.sh                         # run all generic eval suites
#   tests/foundry-evals.sh --suite arbiter-routing # run one suite
#   tests/foundry-evals.sh path/to/file.feature    # run explicit feature file(s)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "$ROOT_DIR/tests/evals/runner.py" "$@"
