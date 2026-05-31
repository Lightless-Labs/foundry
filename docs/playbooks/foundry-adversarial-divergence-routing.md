# Foundry Adversarial Divergence Routing Playbook

This playbook is a mandatory module for `plugins/foundry/skills/foundry-adversarial/SKILL.md`. It owns Phase 1b/2b divergence evaluator dispatch and routing.

## Invariants

- Only trigger divergence routing for potential spec/NLSpec gaps, not ordinary weak tests, missing coverage, or implementation bugs.
- Only one divergence evaluator invocation may be in flight at a time.
- Route exclusively on `findings[0].outcome`; evaluator output follows the reviewer schema, not a top-level outcome field.
- `VALUABLE` means the NLSpec/spec should change; invoke `spec_update_and_restart` and restart Phase 1.
- `NOT_VALUABLE` means the current team should fix its artifact using `findings[0].rationale`.
- `INCONCLUSIVE` means pause and escalate to the user.

## Phase 1b: Red Test Divergence

Trigger when the red-team-test-reviewer flags a test scenario that references behavior not present in the NLSpec Definition of Done.

1. Assemble `EvaluatorInput` with:
   - `nlspec_content`: full current NLSpec text.
   - `diverging_artifact`: the raw flagged test scenario, not a summary.
   - `divergence_phase`: `PHASE_1B`.
   - `red_test_paths`: paths to current red team test files, captured for restart.
2. Create and validate a PromptEnvelope for `foundry:review:divergence-evaluator`.
3. Dispatch the evaluator once for this divergence.
4. Route on `findings[0].outcome`:
   - Phase 1b `VALUABLE` → invoke `spec_update_and_restart`, passing `red_test_paths`; then restart Phase 1.
   - Phase 1b `NOT_VALUABLE` → send red team back with `findings[0].rationale`.
   - Phase 1b `INCONCLUSIVE` → escalate to user (`UserEscalation`) and pause.

## Phase 2b: Test-Fix Inner-Loop Divergence

Trigger when a single failing test reaches the configured consecutive-failure threshold. The default strategy is `adaptive_with_fixed_floor`: keep the fixed N=3 threshold as the audited fallback, but escalate at N=2 when the red test content is unchanged and green has made at least two distinct implementation attempts (`implementation_attempt_hashes`) that still leave the same test failing.

Do not early-trigger on a first failure, on unchanged green implementation hashes, or after red test content changes. Those cases continue the normal green loop until the fixed threshold or another route applies.

1. Process threshold-crossing tests one at a time in ascending `test_id` order.
2. Resolve or escalate the current divergence before evaluating the next.
3. Assemble `EvaluatorInput` with:
   - `test_id`: identifier of the failing test.
   - `implementation_snippet`: raw implementation snippet most recently written by green.
   - `nlspec_content`: full current NLSpec text.
   - `divergence_phase`: `PHASE_2B`.
4. Create and validate a PromptEnvelope for `foundry:review:divergence-evaluator`.
5. Route on `findings[0].outcome`:
   - Phase 2b `VALUABLE` → invoke `spec_update_and_restart`; then restart Phase 1.
   - Phase 2b `NOT_VALUABLE` → send green back with `findings[0].rationale`; reset this test's tracker (`consecutive_fails=0`).
   - Phase 2b `INCONCLUSIVE` → escalate to user (`UserEscalation`) and pause.

## Tracker Reset Rules

The `TestFailureTracker` is pipeline-run-scoped:

- Reset all counters on every Phase 1 restart.
- Passing test → reset `consecutive_fails=0` and clear `implementation_attempt_hashes`.
- Failing test with changed test content hash → reset to `1` and start `implementation_attempt_hashes` from the current green implementation hash.
- Failing test with unchanged test content hash → increment by `1`; append the current implementation hash only if distinct.
- Adaptive early trigger → fire when `consecutive_fails >= 2`, the test content hash is unchanged, and at least two distinct `implementation_attempt_hashes` have failed.
- Fixed fallback trigger → fire when `consecutive_fails >= threshold` (default 3), even if the implementation hash did not change.
- Phase 2b `NOT_VALUABLE` route → reset the evaluated test to `0` after sending feedback to green.

## Behavioral-Smoke Contract

For every Phase 1b or Phase 2b `VALUABLE` restart, append a `divergence_restarts` row in `runs/<run_id>/behavioral-smoke.toon` with `revision_history_count` exactly `1` for that restart event.
