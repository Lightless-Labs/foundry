---
title: Persist parsed phase artifacts from Pi live dispatch smoke
created: 2026-06-08
status: completed
completed: 2026-06-08
research: docs/research/2026-06-08-phase-artifact-capture-research.md
todo: todos/multi-provider-delegation.md
---

# Persist Parsed Phase Artifacts from Pi Live Dispatch Smoke

## Goal

Make `tests/pi-live-dispatch-smoke.sh --phase-task artifact-sketch` more auditable by writing the parsed red and green phase artifacts to the run directory after validation.

## Scope

- In `artifact-sketch` mode, write:
  - `phase-artifacts/red-team-test-plan.json`
  - `phase-artifacts/green-team-implementation-plan.json`
- Keep `plumbing` mode unchanged and avoid creating `phase-artifacts/` there.
- Preserve all existing PromptEnvelope, model-lane, and behavioral-smoke validation behavior.
- Document the new artifact paths in plan/handoff/todo state.

## Non-goals

- No `behavioral-smoke.toon` schema change in this slice.
- No requirement that all run dirs include phase artifacts.
- No new preserved `runs/` directory unless a live run is intentionally kept.

## Acceptance

- [x] `artifact-sketch` mode writes both parsed JSON artifacts under `phase-artifacts/` when the run dir is kept.
- [x] `plumbing` mode remains exact `RED_OK` / `GREEN_OK` and does not require phase artifacts.
- [x] `bash -n tests/pi-live-dispatch-smoke.sh` passes.
- [x] Relevant fast validators pass.

## Validation Log

2026-06-08:

- `bash -n tests/pi-live-dispatch-smoke.sh` — passed.
- `tests/pi-live-dispatch-smoke.sh --phase-task artifact-sketch --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir <tmp>/phase-artifact-capture` — passed; wrote and parsed `phase-artifacts/red-team-test-plan.json` and `phase-artifacts/green-team-implementation-plan.json`.
- Post-run Python assertions over the two phase artifact files — passed.
- `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir <tmp>/plumbing-no-artifacts` — passed; explicit check confirmed plumbing mode did not create `phase-artifacts/`.
- `tests/validate-codex-plugin.sh` — passed 44/44.
- `tests/validate-behavioral-smoke-contract.sh` — passed 9/9.
- `tests/validate-pi-extension.sh` — passed 45/45.
- `tests/validate-agents.sh` — passed 224/224.
