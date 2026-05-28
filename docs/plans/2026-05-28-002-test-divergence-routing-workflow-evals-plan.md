---
title: Add divergence routing workflow eval suite
created: 2026-05-28
status: completed
completed: 2026-05-28
---

# Add Divergence Routing Workflow Evals Plan

## Goal

Extend the generic Gherkin/mock workflow eval harness with deterministic coverage for Phase 1b and Phase 2b divergence routing.

## Scope

- Add `tests/evals/features/divergence-routing.feature` with `VALUABLE`, `NOT_VALUABLE`, and `INCONCLUSIVE` cases for both Phase 1b and Phase 2b.
- Add `tests/evals/adapters/divergence_routing.py` that:
  - emits a scoped `foundry:review:divergence-evaluator` PromptEnvelope,
  - mocks reviewer-schema outputs under `findings[0].outcome`,
  - routes `VALUABLE` to a provenance-preserving `spec_update_and_restart` record,
  - routes Phase 1b `NOT_VALUABLE` back to red without implementation context,
  - routes Phase 2b `NOT_VALUABLE` back to green with only NLSpec How plus PASS/FAIL labels and tracker reset metadata,
  - routes `INCONCLUSIVE` to user escalation.
- Add structural validator anchors and update handoff/todo status.

## Non-goals

- Live model evals.
- Full NLSpec rewrite simulation; the spec restart record is enough for this deterministic process slice.
- Full phase choreography across red/green iterations.

## Acceptance

- [x] `tests/foundry-evals.sh --suite divergence-routing` passes all cases.
- [x] `tests/foundry-evals.sh` includes the new suite and still passes all suites.
- [x] Generated PromptEnvelope artifacts validate through `tests/validate-barrier-envelopes.sh` where applicable.
- [x] `tests/validate-adversarial-modules.sh` includes stable anchors for the new suite.
- [x] `docs/HANDOFF.md` documents the additional suite and validation.

## Validation

2026-05-28:

- `tests/foundry-evals.sh --suite divergence-routing` — passed 6/6 cases across Phase 1b and Phase 2b, including `VALUABLE`, `NOT_VALUABLE`, and `INCONCLUSIVE` routes.
- `tests/foundry-evals.sh` — passed 3 suites / 12 cases total (`arbiter-routing`, `divergence-routing`, `green-followup-barrier`).
- `tests/validate-adversarial-modules.sh` — 70/70 passing with divergence eval anchors.
