---
title: Broaden provider-diverse smoke to selectable worked examples
created: 2026-06-05
status: completed
completed: 2026-06-05
research: docs/research/2026-06-05-provider-diverse-worked-example-smoke-research.md
---

# Broaden Provider-Diverse Smoke to Selectable Worked Examples

## Goal

Extend the slow/manual Pi live dispatch smoke so provider-diverse red/green lanes can validate against deeper worked examples, not only Sudoku.

## Scope

- Add a `--example` option to `tests/pi-live-dispatch-smoke.sh`.
- Support at least:
  - `sudoku-solver` (`30/30`)
  - `rubiks-solver` (`46/46`)
  - `chess-engine` (`44/44`)
- Keep PromptEnvelope isolation unchanged:
  - red prompt is self-contained and sees only full spec/NLSpec summary plus public product contract;
  - green prompt sees only How-style implementation guidance and PASS/FAIL labels;
  - withheld samples remain present for replay/barrier validation.
- Preserve existing default behavior (`sudoku-solver`).
- Run one provider-diverse live lane against `chess-engine` using MiniMax red and Kimi green.

## Non-goals

- No full autonomous adversarial run.
- No private engine changes.
- No richer red/green task than the current plumbing check responses (`RED_OK`/`GREEN_OK`).

## Acceptance

- [x] `tests/pi-live-dispatch-smoke.sh --help` documents `--example` and supported values.
- [x] Default invocation remains Sudoku-compatible.
- [x] A kept Chess provider-diverse run exists under `runs/pi-live-kimi-minimax-chess-smoke/`.
- [x] Chess run `behavioral-smoke.toon` records `chess-engine,44,44,44,44` and distinct MiniMax/Kimi lanes.
- [x] Focused validators pass:
  - [x] `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-chess-smoke/dispatch`
  - [x] `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-chess-smoke`
  - [x] `tests/validate-behavioral-smoke-contract.sh`
  - [x] `tests/validate-pi-extension.sh`
  - [x] `tests/validate-agents.sh`

## Validation Log

2026-06-05:

- `bash -n tests/pi-live-dispatch-smoke.sh` — passed.
- `tests/pi-live-dispatch-smoke.sh --help` — documented `--example` and supported values.
- `tests/pi-live-dispatch-smoke.sh --example chess-engine --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir runs/pi-live-kimi-minimax-chess-smoke` — passed; generated replay artifacts with `chess-engine,44,44,44,44` and distinct MiniMax/Kimi lanes.
- `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes` — passed using temporary artifacts, proving the default Sudoku path remains compatible.
- `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-chess-smoke/dispatch` — passed.
- `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-chess-smoke` — passed.
- `tests/behavioral-smoke.sh` — self-tests passed.
- `tests/validate-behavioral-smoke-contract.sh` — passed 9/9.
- `tests/validate-pi-extension.sh` — passed 45/45.
- `tests/validate-agents.sh` — passed 224/224 with exit status 0.
