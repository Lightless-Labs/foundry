---
title: Broaden provider-diverse live smoke to phase artifacts
created: 2026-06-07
status: completed
completed: 2026-06-07
todo: todos/multi-provider-delegation.md
---

# Broaden Provider-Diverse Live Smoke to Phase Artifacts

## Goal

Extend the slow/manual Pi live dispatch smoke beyond `RED_OK`/`GREEN_OK` plumbing checks so provider-diverse red and green lanes can exercise lightweight adversarial phase work while preserving the PromptEnvelope information barrier.

## Scope

- Add an opt-in richer mode to `tests/pi-live-dispatch-smoke.sh`.
- Keep the existing default plumbing behavior unchanged for compatibility.
- In the richer mode:
  - red receives the full NLSpec/spec summary and produces a compact red test-plan artifact;
  - green receives only NLSpec How plus PASS/FAIL labels and produces a compact implementation-plan artifact;
  - generated artifacts are validated for expected shape and obvious withheld-sample leaks.
- Preserve existing per-lane model override and distinct-model-lane behavior.

## Non-goals

- No full autonomous adversarial run in this slice.
- No private engine changes.
- No committed generated run artifacts unless a slow/manual live run is explicitly kept.
- No weakening of the green barrier: green still receives only How-style guidance plus `test_name: PASS/FAIL` labels.

## Acceptance

- [x] `tests/pi-live-dispatch-smoke.sh --help` documents the new richer phase-task mode.
- [x] Default invocation remains backward-compatible with exact `RED_OK` / `GREEN_OK` checks.
- [x] Richer mode validates red/green child outputs as lightweight phase artifacts.
- [x] `bash -n tests/pi-live-dispatch-smoke.sh` passes.
- [x] Relevant fast validators still pass.

## Validation Log

2026-06-07:

- `bash -n tests/pi-live-dispatch-smoke.sh` — passed.
- `tests/pi-live-dispatch-smoke.sh --help` — documents `--phase-task` and `artifact-sketch`.
- `tests/pi-live-dispatch-smoke.sh --phase-task artifact-sketch --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes` — passed using temporary artifacts; red returned a JSON `red_test_plan`, green returned a JSON `green_implementation_plan`, PromptEnvelope and behavioral-smoke validation passed.
- `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes` — passed using temporary artifacts, proving the default exact `RED_OK` / `GREEN_OK` plumbing path remains compatible.
- `tests/validate-behavioral-smoke-contract.sh` — passed 9/9.
- `tests/validate-pi-extension.sh` — passed 45/45.
- `tests/validate-agents.sh` — passed 224/224.
