---
title: Exercise provider-diverse divergence restart
created: 2026-06-11
status: completed
completed: 2026-06-11
---

# Exercise Provider-Diverse Divergence Restart Plan

## Goal

Produce a replayable live Pi run that exercises the Foundry divergence-to-restart path with distinct provider/model lanes and proves `behavioral-smoke.toon` captures the restart contract.

## Scope

- Use a controlled slugify scenario under `runs/pi-live-kimi-minimax-divergence-restart-smoke/`.
- Preserve PromptEnvelope artifacts for:
  - red-team phase/restart work on MiniMax,
  - green-team implementation/follow-up work on Kimi,
  - `foundry:review:divergence-evaluator` over a Phase 2b stable failure,
  - `foundry:nlspec` rerun input for `spec_update_and_restart`.
- Dispatch live child Pi processes through `foundry_team` from envelope paths only.
- Record actual model lanes in `behavioral-smoke.toon` with `requires_distinct_model_lanes: true` and `requires_divergence_restart: true`.
- Validate replay artifacts with `tests/validate-barrier-envelopes.sh` and `tests/behavioral-smoke.sh`.

## Non-goals

- No private engine changes.
- No new public workflow semantics beyond documenting any provider-output quirks found in the live lane.
- No broad barrier breach: green sees only the How section and PASS/FAIL labels; red sees spec/NLSpec context but no green implementation workspace.

## Acceptance

- [x] PromptEnvelope artifacts validate before live dispatch.
- [x] Live `foundry_team` dispatch completes for red-team and green-team on distinct actual model lanes.
- [x] Live divergence evaluator returns a reviewer-schema `findings[0].outcome` route for a controlled Phase 2b stable failure.
- [x] The restart artifact records `revision_history_count: 1` and the behavioral smoke manifest validates with `requires_divergence_restart: true`.
- [x] Fast validation passes for changed public-plugin scripts/docs.
- [x] `docs/HANDOFF.md` is updated with results, learnings, and next steps.

## Validation Log

2026-06-11:

- `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-divergence-restart-smoke/dispatch` — passed for red-team, green-team, divergence-evaluator r1/r2, and `spec_update_and_restart` NLSpec rerun envelopes.
- `foundry_team` live dispatch completed for red-team on `minimax/MiniMax-M3`, green-team on `kimi-coding/kimi-for-coding`, and divergence-evaluator on `minimax/MiniMax-M3`.
- First divergence-evaluator packet returned `NOT_VALUABLE` because the prompt included an explicit v1 note excluding accented Latin transliteration; this is preserved as a prompt-authoring anomaly.
- Retry r2 removed that exclusion and returned reviewer-schema `findings[0].outcome = VALUABLE` with a non-null `gap_description` for accented Latin transliteration policy.
- `spec-update-and-restart.json` and `phase1-restart-package.json` record `revision_history_count: 1`, `gap_description_verbatim: true`, and `test_failure_tracker: reset_all_counters`.
- `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-divergence-restart-smoke` — passed with `requires_divergence_restart: true` and `requires_distinct_model_lanes: true`.
- `tests/validate-agents.sh` — passed 224/224.

## Notes

This is a controlled route/restart smoke, not a full post-restart adversarial implementation run. The pre-restart red/green phase artifacts intentionally remain tied to the original NLSpec; the restart evidence is the r2 evaluator route plus the `spec_update_and_restart` NLSpec rerun envelope and restart package. Post-restart red/green regeneration should be a follow-up if this smoke is expanded into a full resumed run.
