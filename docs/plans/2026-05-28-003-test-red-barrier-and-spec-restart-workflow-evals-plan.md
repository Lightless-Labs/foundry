---
title: Add red follow-up barrier and spec restart workflow eval suites
created: 2026-05-28
status: completed
completed: 2026-05-28
---

# Add Red Barrier and Spec Restart Workflow Evals Plan

## Goal

Extend process-scale eval coverage beyond divergence routing into two adjacent invariants:

1. Red follow-up prompts never receive implementation code, counterpart paths, or counterpart reasoning.
2. `spec_update_and_restart` preserves provenance: the NLSpec agent receives verbatim `findings[0].gap_description`, restart state resets trackers, and revision cap escalates rather than mutating specs.

## Scope

- Add `red-followup-barrier` generic eval suite.
- Add `spec-update-restart` generic eval suite.
- Update structural validator anchors and handoff/todo docs.

## Non-goals

- Live model runs.
- Actually rewriting NLSpec files; deterministic records and envelopes are enough for this slice.
- Full phase choreography.

## Acceptance

- [x] `tests/foundry-evals.sh --suite red-followup-barrier` passes.
- [x] `tests/foundry-evals.sh --suite spec-update-restart` passes.
- [x] `tests/foundry-evals.sh` passes all suites.
- [x] Generated PromptEnvelope artifacts validate through `tests/validate-barrier-envelopes.sh` where applicable.
- [x] `tests/validate-adversarial-modules.sh` has anchors for both new suites.
- [x] Handoff/todo docs document current suite count and validation.

## Validation

2026-05-28:

- `tests/foundry-evals.sh --suite red-followup-barrier` — passed 3/3 cases.
- `tests/foundry-evals.sh --suite spec-update-restart` — passed 3/3 cases.
- `tests/foundry-evals.sh` — passed 5 suites / 18 cases total.
- `tests/validate-adversarial-modules.sh` — 81/81 passing with red follow-up and spec restart eval anchors.
