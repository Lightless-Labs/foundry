---
title: Exercise multi-provider red/green delegation
created: 2026-06-03
status: completed
completed: 2026-06-03
---

# Exercise Multi-Provider Red/Green Delegation Plan

## Goal

Systematically exercise Foundry's ability to delegate isolated red and green lanes to distinct model/provider choices through the public Pi `foundry_team` PromptEnvelope boundary.

Gemini is no longer available in the current environment, so the first live lane should use one of:

- red: Codex 5.5 high-reasoning/xhigh lane; green: Kimi lane (`kimi-coding/kimi-for-coding`), preferred when available for true provider diversity; or
- red: Codex 5.5 high-reasoning/xhigh lane; green: Codex 5.5 medium lane, acceptable as a distinct model-lane fallback when Kimi is unavailable.

## Scope

- Keep all red/green dispatches behind serialized `foundry.prompt-envelope.v1` artifacts.
- Extend the slow/manual Pi live dispatch smoke so red and green can be invoked with explicit per-lane model overrides.
- Extend replay validation so a run can declare that it intentionally requires distinct red/green model lanes.
- Document the invocation pattern for Codex/Kimi or Codex/Codex-effort fallback lanes.
- Preserve compatibility with existing run artifacts that do not require distinct lanes.

## Non-goals

- No private engine/provider implementation changes.
- No Codex-native subagent assumptions; Codex remains packaging-only until a PromptEnvelope-safe dispatch primitive exists.
- No weakening of the green barrier: green still receives only NLSpec How plus `test_name: PASS/FAIL` labels.

## Acceptance

- [x] `tests/pi-live-dispatch-smoke.sh` accepts red/green model override arguments and passes them to `foundry_team` per dispatch.
- [x] `behavioral-smoke.toon` can mark `requires_distinct_model_lanes: true`, and `tests/behavioral-smoke.sh` fails if red/green lanes collapse to the same actual model.
- [x] Docs show a live invocation for Codex-xhigh red versus Kimi or Codex-medium green.
- [x] Fast validators pass.
- [x] A slow/manual live run is executed when the requested models are available, and the kept run directory is recorded in `docs/HANDOFF.md`.

## Validation Log

2026-06-03:

- `tests/pi-live-dispatch-smoke.sh --red-model openai-codex/gpt-5.5:xhigh --green-model openai-codex/gpt-5.5:medium --require-distinct-model-lanes --run-dir runs/pi-live-multilane-smoke` — passed; generated replay artifacts and `behavioral-smoke.toon` with distinct red/green Codex thinking lanes.
- `tests/validate-barrier-envelopes.sh runs/pi-live-multilane-smoke/dispatch` — passed.
- `tests/behavioral-smoke.sh runs/pi-live-multilane-smoke` — passed.
- `tests/behavioral-smoke.sh` — self-tests passed, including the new collapsed-lane failure case.
- `tests/validate-behavioral-smoke-contract.sh` — 9/9 passed.
- `tests/validate-pi-extension.sh` — 45/45 passed.
- `tests/validate-adversarial-modules.sh` — 106/106 passed.
- `tests/validate-agents.sh` — 224/224 passed.
