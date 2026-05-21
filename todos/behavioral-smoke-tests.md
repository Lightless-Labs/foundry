---
title: Add behavioral smoke tests — end-to-end adversarial runs, barrier invariants
origin: 2026-04-17 ilia-feedback-foundry-plugin (item 3)
priority: high
status: active
updated: 2026-05-21
---

# Behavioral Smoke Tests

**Addendum:** 2026-05-21 — replay-level public-plugin harness landed. `tests/behavioral-smoke.sh` validates run directories containing PromptEnvelope JSON artifacts plus a compact `behavioral-smoke.toon` summary. It delegates barrier checks to `tests/validate-barrier-envelopes.sh`, asserts worked-example pass rates, model-lane matches, and Phase 1b/2b `VALUABLE` restart revision counts. It includes self-tests for good runs, model mismatch failures, divergence-count failures, and TOON row-count failures.

**Addendum:** 2026-05-21 — public Pi live-lane support started. Pi has no built-in subagent/team/swarm primitive, so the Foundry Pi package now includes `extensions/pi-foundry-team/index.ts`, inspired by Pi's bundled `examples/extensions/subagent/`. The `foundry_team` tool dispatches child `pi --mode json -p --no-session` processes from PromptEnvelope paths, validates withheld samples before dispatch, disables child sessions/extensions/skills/context-files by default, and reports actual model IDs for `behavioral-smoke.toon`. Remaining gap: run a real public-plugin adversarial session under Pi and validate its emitted `runs/<run_id>/` artifacts.

`tests/validate-agents.sh` currently covers structural concerns: YAML frontmatter, required sections, `model: inherit`, tool lists, attribution, territory boundaries. That is valuable but it verifies file shape, not behavior.

The hard questions are behavioral:

- Does the barrier actually hold when skills are composed end-to-end?
- Do divergence restarts (Phase 1b/2b `VALUABLE`) behave correctly?
- Do different models stay within the intended lanes under parallel dispatch?
- Do the worked examples still produce their expected pass rates (sudoku 30/30, chess 44/44)?

## What to fix

- End-to-end smoke test: one of the examples (sudoku is cheapest) runs from NLSpec through adversarial skill to green implementation with test outcomes.
- Barrier invariant assertion: post-run, the captured dispatch envelopes (see `mechanical-barrier-enforcement.md`) are diffed against the barrier matrix — any leak fails the test.
- Divergence restart assertion: a deliberately-ambiguous NLSpec triggers Phase 2b `VALUABLE`, the pipeline restarts, and the revision history is exactly one entry.
- Model-lane assertion: if provider overrides are in play, each dispatch's actual model matches the planned provider.

## Suggested approach

Add `tests/behavioral-smoke.sh` (or a Rust harness in the engine) that sits alongside `validate-agents.sh`. Tag it "slow" so CI gates it separately from fast structural checks.

The public-plugin replay harness exists as `tests/behavioral-smoke.sh`; the public Pi dispatch primitive exists as `foundry_team`. The remaining slow/live lane is to exercise a real public-plugin adversarial run and feed its emitted `runs/<run_id>/` artifacts back into the harness.

See: `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md` (item 3).
