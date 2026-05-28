---
title: Generalize Gherkin-authored workflow evals
created: 2026-05-28
status: completed
completed: 2026-05-28
---

# Generalize Gherkin-Authored Workflow Evals Plan

## Goal

Promote the arbiter-specific Gherkin/mock eval pattern into a reusable deterministic Foundry workflow eval harness.

The harness should let process scenarios be authored as small `.feature` files, execute adapter code that mocks agent outputs or workflow artifacts, validate generated PromptEnvelope artifacts with existing validators, and report clear route/barrier/provenance outcomes without live model calls.

## Scope

- Add a generic `tests/foundry-evals.sh` entrypoint.
- Add a reusable Python runner under `tests/evals/` that can run one or more feature files and dispatch to suite adapters.
- Port the existing arbiter routing evals into the generic framework while preserving the existing `tests/arbiter-routing-evals.sh` compatibility command.
- Add one non-arbiter suite; first slice: green follow-up barrier evals that prove green sees only NLSpec How plus `test_name: PASS/FAIL` labels.
- Update todo/handoff docs and run focused validators.

## Non-goals

- Live model/provider evals. Those can reuse feature files later but remain a separate slow/manual lane.
- Replacing `validate-barrier-envelopes.sh` or `behavioral-smoke.sh`; this framework should compose them.
- Full phase choreography in this slice.

## Design

```text
tests/
  foundry-evals.sh
  evals/
    runner.py
    adapters/
      arbiter_routing.py
      green_followup_barrier.py
    features/
      arbiter-routing.feature
      green-followup-barrier.feature
```

The runner parses a deliberately small Gherkin subset:

- `Feature:` metadata for human readability.
- Optional tags for future filtering.
- A single `Examples:` table per feature for deterministic cases.

Each adapter receives parsed case dictionaries plus root/temp paths and owns suite-specific artifact generation and assertions.

## Acceptance

- [x] `tests/foundry-evals.sh` runs all generic suites.
- [x] `tests/foundry-evals.sh --suite arbiter-routing` covers the existing four arbiter routes.
- [x] `tests/arbiter-routing-evals.sh` remains usable as a compatibility wrapper.
- [x] At least one non-arbiter suite passes through the generic runner.
- [x] Generated PromptEnvelope artifacts are validated by `tests/validate-barrier-envelopes.sh`.
- [x] `docs/HANDOFF.md` and `todos/generalize-workflow-evals.md` document the new command and status.

## Validation

2026-05-28:

- `tests/foundry-evals.sh` — passed 2 generic suites (`arbiter-routing`, `green-followup-barrier`) and 6 cases total.
- `tests/arbiter-routing-evals.sh tests/fixtures/arbiter-routing-evals.feature` — compatibility command passed against the old fixture path.
- `tests/foundry-evals.sh --suite green-followup-barrier` — passed 2/2 green barrier cases.
- `tests/validate-adversarial-modules.sh` — 62/62 passing with generic eval anchors.
- `tests/validate-agents.sh` — 224/224 passing.
