---
title: Exercise post-restart resumed red/green smoke
created: 2026-06-11
status: completed
completed: 2026-06-11
---

# Exercise Post-Restart Resumed Red/Green Smoke Plan

## Goal

Extend the provider-diverse divergence/restart evidence with a resumed post-restart red/green smoke that starts from the revised slugify NLSpec, uses opaque green test IDs, and proves the resumed path converges.

## Scope

- Use the existing `runs/pi-live-kimi-minimax-divergence-restart-smoke/` run as the anchor.
- Add post-restart PromptEnvelope artifacts under `dispatch/post_restart/` for:
  - red-team review/update of existing tests from the revised NLSpec plus change summary,
  - green-team implementation guidance from revised How plus opaque PASS/FAIL labels only.
- Dispatch red on `minimax/MiniMax-M3` and green on `kimi-coding/kimi-for-coding` through `foundry_team`.
- Preserve child outputs, an opaque test-id mapping for orchestrator audit only, and a resumed-run convergence record.
- Validate all PromptEnvelopes and the run-level `behavioral-smoke.toon`.

## Non-goals

- No private engine changes.
- No new public workflow semantics.
- No green exposure to human-readable test names, test code, assertions, raw failure output, or NLSpec Done criteria in post-restart envelopes.

## Acceptance

- [x] Post-restart red and green PromptEnvelope artifacts validate.
- [x] Live red/green post-restart dispatches complete on distinct actual model lanes.
- [x] Green post-restart envelope uses opaque test IDs only.
- [x] A convergence record shows the resumed slugify path passes the updated policy checks.
- [x] `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-divergence-restart-smoke/dispatch` passes.
- [x] `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-divergence-restart-smoke` passes.
- [x] Handoff is updated and changes are committed/pushed.

## Validation Log

2026-06-11:

- Authored post-restart red envelope from revised NLSpec + change summary and post-restart green envelopes from revised How + opaque `T-###` PASS/FAIL labels.
- `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-divergence-restart-smoke/dispatch/post_restart` — passed after removing a red-prompt forbidden hint and avoiding withheld samples that duplicated visible opaque labels.
- Live `foundry_team` post-restart dispatch completed for red-team on `minimax/MiniMax-M3` and green-team on `kimi-coding/kimi-for-coding`.
- Green plan r1 referenced the older fuller-smoke path, so a separate r2 green envelope requested a self-contained implementation artifact under this run only; r2 completed on `kimi-coding/kimi-for-coding`.
- `cd runs/pi-live-kimi-minimax-divergence-restart-smoke/resumed/green && cargo test --quiet` — passed `4/4` post-restart policy tests.
- `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-divergence-restart-smoke/dispatch` — passed for the full run dispatch tree.
- `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-divergence-restart-smoke` — passed with two `test_results` rows, seven `model_lanes`, `requires_divergence_restart: true`, and `requires_distinct_model_lanes: true`.

## Notes

The first post-restart red and green child calls were dispatched in the same `foundry_team` call, but the green envelope was authored before red child output existed and did not include red output. The r2 green implementation envelope explicitly redacts `post_restart_red_output` and still uses only opaque test IDs.
