---
title: Add behavioral smoke tests — end-to-end adversarial runs, barrier invariants
origin: 2026-04-17 ilia-feedback-foundry-plugin (item 3)
priority: high
status: completed — replay harness, Pi dispatch primitive, slow/manual Pi live dispatch smoke, and smoke-scoped autonomous Pi adversarial run landed
updated: 2026-05-22
---

# Behavioral Smoke Tests

**Addendum:** 2026-05-21 — replay-level public-plugin harness landed. `tests/behavioral-smoke.sh` validates run directories containing PromptEnvelope JSON artifacts plus a compact `behavioral-smoke.toon` summary. It delegates barrier checks to `tests/validate-barrier-envelopes.sh`, asserts worked-example pass rates, model-lane matches, and Phase 1b/2b `VALUABLE` restart revision counts. It includes self-tests for good runs, model mismatch failures, divergence-count failures, and TOON row-count failures.

**Addendum:** 2026-05-21 — public Pi live-lane support started. Pi has no built-in subagent/team/swarm primitive, so the Foundry Pi package now includes `extensions/pi-foundry-team/index.ts`, inspired by Pi's bundled `examples/extensions/subagent/`. The `foundry_team` tool dispatches child `pi --mode json -p --no-session` processes from PromptEnvelope paths, validates withheld samples before dispatch, disables child sessions/extensions/skills/context-files by default, and reports actual model IDs for `behavioral-smoke.toon`.

**Addendum:** 2026-05-22 — slow/manual public Pi live dispatch smoke landed in `tests/pi-live-dispatch-smoke.sh`. It creates real PromptEnvelope artifacts, runs the Sudoku worked-example red tests (`30/30`), invokes `foundry_team` through `pi -e ./extensions/pi-foundry-team/index.ts`, captures provider-qualified child model lanes, emits `behavioral-smoke.toon`, and validates the generated run directory with `tests/behavioral-smoke.sh`.

**Addendum:** 2026-05-22 — smoke-scoped autonomous Pi adversarial run completed. Invoked `/skill:foundry-adversarial` through Pi with the `foundry_team` extension, copied the Sudoku example to `/tmp/foundry-pi-autonomous-sudoku-smoke/sudoku-solver`, ran `cargo test -- --nocapture` with `30/30` passing, dispatched red-team/green-team/barrier-integrity-auditor from PromptEnvelope artifacts, emitted `runs/pi-autonomous-sudoku-smoke/behavioral-smoke.toon`, and validated with `tests/behavioral-smoke.sh runs/pi-autonomous-sudoku-smoke` plus `tests/validate-barrier-envelopes.sh runs/pi-autonomous-sudoku-smoke/dispatch`.

`tests/validate-agents.sh` currently covers structural concerns: YAML frontmatter, required sections, `model: inherit`, tool lists, attribution, territory boundaries. That is valuable but it verifies file shape, not behavior.

The hard questions are behavioral:

- Does the barrier actually hold when skills are composed end-to-end?
- Do divergence restarts (Phase 1b/2b `VALUABLE`) behave correctly?
- Do different models stay within the intended lanes under parallel dispatch?
- Do the worked examples still produce their expected pass rates (sudoku 30/30, chess 44/44)?

## What to fix

- [x] Pi live dispatch smoke: Sudoku worked-example red tests run `30/30`, real `foundry_team` child dispatches execute from PromptEnvelope artifacts, and emitted `behavioral-smoke.toon` validates.
- [x] Smoke-scoped autonomous end-to-end Pi smoke test: Sudoku runs through `/skill:foundry-adversarial`, dispatches red/green/auditor through `foundry_team`, records `30/30`, and validates emitted artifacts.
- [x] Barrier invariant assertion: post-run, the captured dispatch envelopes (see `mechanical-barrier-enforcement.md`) are diffed against the barrier matrix — any leak fails the test.
- [x] Divergence restart assertion: a deliberately-ambiguous NLSpec triggers Phase 2b `VALUABLE`, the pipeline restarts, and the revision history is exactly one entry.
- [x] Model-lane assertion: if provider overrides are in play, each dispatch's actual model matches the planned provider.
- [x] Pi subagent/team primitive: because Pi has no built-in subagents, provide a public-extension dispatch mechanism that can produce real run artifacts from PromptEnvelope paths.

## Suggested approach

Add `tests/behavioral-smoke.sh` (or a Rust harness in the engine) that sits alongside `validate-agents.sh`. Tag it "slow" so CI gates it separately from fast structural checks.

The public-plugin replay harness exists as `tests/behavioral-smoke.sh`; the public Pi dispatch primitive exists as `foundry_team`; and the slow/manual public Pi live dispatch smoke exists as `tests/pi-live-dispatch-smoke.sh`. The remaining slow/live lane is to exercise a full autonomous public-plugin adversarial run and feed its emitted `runs/<run_id>/` artifacts back into the harness. This should be done in this public plugin/extension repo (not via private runtime assumptions), likely after Pi skill adapters are available.

See: `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md` (item 3).
