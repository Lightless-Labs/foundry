---
title: Exercise multi-provider red/green delegation
origin: 2026-06-03 user request after Gemini became unavailable
priority: high
status: completed
updated: 2026-06-03
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

Completed 2026-06-03. Live run: `runs/pi-live-multilane-smoke/` with red `openai-codex/gpt-5.5:xhigh`, green `openai-codex/gpt-5.5:medium`, orchestrator `openai-codex/gpt-5.5`; barrier and behavioral validators passed.

## Plan

See `docs/plans/2026-06-03-001-test-multi-provider-delegation-plan.md`.
