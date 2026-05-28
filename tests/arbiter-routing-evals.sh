#!/usr/bin/env bash
# arbiter-routing-evals.sh — compatibility entrypoint for arbiter routing evals.
#
# The generic implementation lives in tests/foundry-evals.sh with the
# arbiter-routing adapter. Keep this wrapper so existing docs, local habits, and
# validators continue to work.
#
# Validator anchors preserved intentionally: EXPECTED_ROUTE, validate-barrier-envelopes.sh.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FEATURE_PATH="${1:-$ROOT_DIR/tests/evals/features/arbiter-routing.feature}"

exec "$ROOT_DIR/tests/foundry-evals.sh" --suite arbiter-routing "$FEATURE_PATH"
