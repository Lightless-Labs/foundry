---
title: Add full phase choreography workflow eval suite
created: 2026-05-30
status: completed
completed: 2026-05-30
---

# Add Full Phase Choreography Workflow Evals Plan

## Goal

Close the deterministic workflow-eval arc by adding full mocked adversarial-run choreography coverage across setup, red, review, green, test/fix, optional restart, final review, and finalization.

## Scope

- Add a `phase-choreography` generic eval suite under `tests/evals/features/`.
- Add an adapter that generates a replayable mocked run directory per case, including PromptEnvelopes and `behavioral-smoke.toon`.
- Cover at least:
  - happy path to finalization,
  - VALUABLE divergence restart and tracker reset,
  - Phase 3 reviewer rejection routed back to the proper team before finalization.
- Validate generated dispatch artifacts with `validate-barrier-envelopes.sh` and run-level artifacts with `behavioral-smoke.sh`.
- Add module-validator anchors and update handoff/todo docs.

## Non-goals

- Live model dispatch.
- Exhaustive branch coverage for every arbiter/divergence route already covered by focused suites.
- Replacing the focused eval suites; this suite validates end-to-end phase ordering and handoff contracts.

## Acceptance

- [x] `tests/foundry-evals.sh --suite phase-choreography` passes.
- [x] `tests/foundry-evals.sh` passes all suites.
- [x] `tests/validate-adversarial-modules.sh` includes and passes phase-choreography anchors.
- [x] `docs/HANDOFF.md` documents current suite/case counts and next steps.

## Validation

2026-05-30:

- `tests/foundry-evals.sh --suite phase-choreography` — passed 3/3 cases.
- `tests/foundry-evals.sh` — passed 7 suites / 24 cases total.
- `tests/validate-adversarial-modules.sh` — passed 94/94 with phase-choreography anchors.
- `tests/validate-agents.sh` — passed 224/224.
