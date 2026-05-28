---
title: Generalize Gherkin-authored workflow evals
origin: 2026-05-26 arbiter routing eval discussion
priority: high
status: completed
completed: 2026-05-28
---

# Generalize Workflow Evals

The arbiter routing evals proved a useful pattern: author process scenarios in a small Gherkin subset, mock agent outputs deterministically, generate replayable PromptEnvelope artifacts, and validate routing/barrier/provenance invariants without live model calls.

Generalize this into a reusable Foundry workflow eval framework rather than keeping it arbiter-specific.

**Completed:** 2026-05-28 — generic deterministic eval harness landed at `tests/foundry-evals.sh` + `tests/evals/runner.py`. The old `tests/arbiter-routing-evals.sh` command is now a compatibility wrapper. Generic suites currently cover arbiter routing (4 mocked route cases) and green follow-up barrier enforcement (2 PromptEnvelope cases). Future suites can add divergence routing, red follow-up, spec restart, reviewer fan-out, and full phase choreography as adapters under `tests/evals/adapters/`.

## Goal

Create a deterministic eval harness for Foundry workflow behavior:

- Gherkin `.feature` files describe reusable process scenarios and expected routes.
- Adapters generate PromptEnvelopes, mocked agent outputs, route decisions, follow-up envelopes, and optional behavioral summaries.
- Existing validators (`validate-barrier-envelopes.sh`, module validators, behavioral smoke validators) remain the source of truth for mechanical invariants.
- The same feature cases can later be reused for live model evals by replacing mocked outputs with real dispatch.

## Candidate Layout

```text
tests/
  foundry-evals.sh
  evals/
    runner.py
    features/
      arbiter-routing.feature
      divergence-routing.feature
      barrier-envelopes.feature
      green-followup-barrier.feature
      red-followup-barrier.feature
      spec-update-restart.feature
      reviewer-fanout.feature
    adapters/
      arbiter_routing.py
      divergence_routing.py
      barrier_envelopes.py
      spec_update_restart.py
```

## Initial Suites

1. **Arbiter routing** — port existing `tests/arbiter-routing-evals.sh` + `tests/fixtures/arbiter-routing-evals.feature` into the generic framework.
2. **Divergence routing** — `VALUABLE`, `NOT_VALUABLE`, `INCONCLUSIVE` for Phase 1b and Phase 2b.
3. **Green follow-up barrier** — prove green receives only NLSpec How plus `test_name: PASS/FAIL`, never assertions, raw failures, `.feature` text, or NLSpec Done.
4. **Red follow-up barrier** — prove red receives no implementation code, green paths, or green reasoning after arbitration/reviewer routes.
5. **Spec update/restart** — prove `findings[0].gap_description` is passed verbatim, the NLSpec agent is sole author, `TestFailureTracker` resets, and Phase 1 restarts.
6. **Reviewer fan-out** — mock reviewer outputs and verify routing back to red, green, spec update, or user.
7. **Phase choreography** — full mocked adversarial runs with scripted outputs per phase.

## Acceptance

- [x] Generic eval runner exists and can run one or more feature files by suite.
- [x] Existing arbiter evals are ported without losing coverage.
- [x] At least one non-arbiter suite exists (green follow-up barrier).
- [x] Eval output reports case IDs, expected routes, actual routes, and validator failures clearly.
- [x] `docs/HANDOFF.md` and validator docs list the generic eval command.
- [x] Fast validation includes the generic eval suite as a standalone command plus structural anchors in `tests/validate-adversarial-modules.sh`.

## Notes

These are **deterministic process evals**, not model-quality evals. They test orchestration, barrier, provenance, and routing contracts. Live model evals can reuse the same `.feature` files later, but should be a separate slow/manual lane.
