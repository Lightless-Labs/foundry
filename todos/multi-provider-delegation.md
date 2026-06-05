---
title: Exercise multi-provider red/green delegation
origin: 2026-06-03 user request after Gemini became unavailable
priority: high
status: completed
updated: 2026-06-05
---

# Exercise Multi-Provider Red/Green Delegation

Foundry's information barrier should be strengthened by sending isolated red and green lanes to different model/provider choices. Gemini is no longer available, so use Codex 5.5 high-reasoning/xhigh for red and either Kimi or a lower-effort Codex 5.5 lane for green.

## Work items

- [x] Add explicit per-lane model override support to the public Pi live dispatch smoke.
- [x] Add replay validation for runs that intentionally require distinct red/green model lanes.
- [x] Document the recommended invocation patterns and fallback when Kimi is unavailable.
- [x] Run a slow/manual live smoke with distinct lanes and preserve/validate the run artifacts.
- [x] Update `docs/HANDOFF.md` with final model lanes, validation status, and learnings.

## Completion

Completed 2026-06-03. Live run: `runs/pi-live-multilane-smoke/` with red `openai-codex/gpt-5.5:xhigh`, green `openai-codex/gpt-5.5:medium`, orchestrator `openai-codex/gpt-5.5`; barrier and behavioral validators passed. Correction: Pi has `kimi-coding` auth configured, but Kimi spot-check calls hung without stdout/stderr, so Codex medium was used as the operational fallback.

**Addendum:** 2026-06-05 — Kimi and MiniMax are operational in Pi. Added selectable worked-example support to `tests/pi-live-dispatch-smoke.sh` (`sudoku-solver` 30/30, `rubiks-solver` 46/46, `chess-engine` 44/44). Preserved validated provider-diverse runs for Sudoku (`runs/pi-live-kimi-minimax-smoke/`) and Chess (`runs/pi-live-kimi-minimax-chess-smoke/`) with red `minimax/MiniMax-M3` and green `kimi-coding/kimi-for-coding`.

## Plan

See `docs/plans/2026-06-03-001-test-multi-provider-delegation-plan.md` and `docs/plans/2026-06-05-001-test-provider-diverse-worked-example-smoke-plan.md`.
