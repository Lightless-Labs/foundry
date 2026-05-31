---
title: Re-assess Phase 2 divergence trigger strategy
created: 2026-05-31
status: completed
completed: 2026-05-31
---

# Re-assess Phase 2 Divergence Trigger Strategy Plan

## Goal

Replace the purely fixed Phase 2b divergence trigger (`N=3` consecutive failures) with an auditable adaptive strategy that can escalate obvious green/spec divergence earlier without firing on ordinary first-pass implementation bugs.

## Scope

- Document the chosen trigger policy in `foundry-adversarial` and the divergence routing playbook.
- Preserve the fixed `N=3` threshold as the default floor/fallback.
- Add a pattern-based early trigger when a stable failing test has failed at least twice, the red test content has not changed, and green has made distinct implementation attempts that still do not move the outcome.
- Add deterministic workflow eval coverage for trigger decisions.
- Update the todo and handoff state after validation.

## Non-goals

- No private engine/state-machine implementation in this public plugin repo.
- No live model run; this is a process/routing contract update.
- No relaxation of the red/green information barrier or PASS/FAIL-only green feedback.

## Acceptance

- [x] Skill and playbook document the adaptive/fixed trigger policy and tracker metadata.
- [x] Generic workflow eval suite covers fixed-threshold, adaptive-early, no-early-unchanged-implementation, and test-hash-reset cases.
- [x] `tests/foundry-evals.sh --suite phase2-trigger-strategy` passes.
- [x] `tests/foundry-evals.sh` passes all suites.
- [x] `tests/validate-adversarial-modules.sh` passes.
- [x] Todo and handoff reflect the completed decision.

## Validation

2026-05-31:

- `tests/foundry-evals.sh --suite phase2-trigger-strategy` — passed 4/4 cases.
- `tests/foundry-evals.sh` — passed 8 suites total.
- `tests/validate-adversarial-modules.sh` — passed 103/103 checks.
- `tests/validate-agents.sh` — passed 224/224 checks.
