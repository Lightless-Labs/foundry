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

**Completed:** 2026-05-28 — generic deterministic eval harness landed at `tests/foundry-evals.sh` + `tests/evals/runner.py`. The old `tests/arbiter-routing-evals.sh` command is now a compatibility wrapper. Generic suites initially covered arbiter routing (4 mocked route cases) and green follow-up barrier enforcement (2 PromptEnvelope cases).

**Addendum:** 2026-05-28 — added the first process-routing expansion suite, `divergence-routing`, with Phase 1b and Phase 2b coverage for `VALUABLE`, `NOT_VALUABLE`, and `INCONCLUSIVE`. The adapter validates divergence evaluator PromptEnvelopes, reviewer-schema `findings[0].outcome` routing, spec restart provenance records, red/green barrier-preserving follow-up, and Phase 2b tracker reset metadata.

**Addendum:** 2026-05-28 — added `red-followup-barrier` and `spec-update-restart` suites. Red follow-up cases validate that arbiter/divergence/reviewer routes withhold implementation code, counterpart paths, and counterpart reasoning. Spec restart cases validate `NLSpecRerunInput`, verbatim `gap_description`, NLSpec-agent-only authorship metadata, Phase 1 restart packages, tracker reset, and revision-cap escalation. Future suites can add reviewer fan-out and full phase choreography as adapters under `tests/evals/adapters/`.

**Addendum:** 2026-05-29 — added `reviewer-fanout` suite. It generates Phase 3 PromptEnvelopes for mandatory reviewers plus language/Bazel/UniFFI/reliability conditionals, validates all envelopes through `validate-barrier-envelopes.sh`, and asserts implementation-facing versus test-facing reviewer territory boundaries. Generic eval coverage is now 6 suites / 21 deterministic cases; remaining high-value expansion is full phase choreography.

**Addendum:** 2026-05-30 — added `phase-choreography` suite. It builds replayable mocked run directories across setup, red/review, green, test/fix, optional `VALUABLE` spec restart, reviewer rejection/fix, final review, and Phase 4 finalization; validates PromptEnvelopes and `behavioral-smoke.toon`; and raises coverage to 7 suites / 24 deterministic cases. The original generic eval roadmap is now complete enough to use as a fast regression lane; future work should either add edge-case suites from real bugs or reuse the scenarios in a slow/live lane.

**Addendum:** 2026-05-31 — added `phase2-trigger-strategy` suite. It covers the adopted `adaptive_with_fixed_floor` trigger policy: fixed N=3 fallback, N=2 early trigger when unchanged tests survive distinct green implementation attempts, no early trigger for unchanged implementation hashes, and tracker reset after red test content changes. Generic eval coverage is now 8 suites / 28 deterministic cases.

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
6. **Reviewer fan-out** — deterministic Phase 3 reviewer selection plus PromptEnvelope territory checks for implementation-facing and test-facing reviewers.
7. **Phase choreography** — full mocked adversarial runs with scripted outputs per phase, including `VALUABLE` spec restart, reviewer rejection routed back to green, final validator gates, and behavioral-smoke artifacts.
8. **Phase 2 trigger strategy** — adaptive/fixed Phase 2b trigger decisions: N=3 fallback, N=2 early trigger with distinct green implementation hashes, no early trigger for unchanged implementations, and reset on red test content changes.

## Acceptance

- [x] Generic eval runner exists and can run one or more feature files by suite.
- [x] Existing arbiter evals are ported without losing coverage.
- [x] At least one non-arbiter suite exists (green follow-up barrier).
- [x] Eval output reports case IDs, expected routes, actual routes, and validator failures clearly.
- [x] `docs/HANDOFF.md` and validator docs list the generic eval command.
- [x] Fast validation includes the generic eval suite as a standalone command plus structural anchors in `tests/validate-adversarial-modules.sh`.

## Notes

These are **deterministic process evals**, not model-quality evals. They test orchestration, barrier, provenance, and routing contracts. Live model evals can reuse the same `.feature` files later, but should be a separate slow/manual lane.
