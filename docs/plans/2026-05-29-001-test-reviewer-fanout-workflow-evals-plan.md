---
title: Add reviewer fan-out workflow eval suite
created: 2026-05-29
status: completed
completed: 2026-05-29
---

# Add Reviewer Fan-Out Workflow Evals Plan

## Goal

Extend deterministic Foundry workflow eval coverage to Phase 3 reviewer fan-out so final review dispatches remain complete, conditional reviewers are selected correctly, and information-barrier envelopes stay valid.

## Scope

- Add a `reviewer-fanout` generic eval suite under `tests/evals/features/`.
- Add a matching adapter that generates Phase 3 PromptEnvelope artifacts and a deterministic dispatch plan.
- Validate red-facing, green-facing, implementation-facing, test-facing, and barrier-auditor prompts against the existing barrier validator and adapter-level assertions.
- Add structural anchors so future modularization cannot silently drop the suite.
- Update handoff/todo docs with the new suite count and next step.

## Non-goals

- Live model reviewer calls.
- Full Phase 0–4 choreography.
- Reworking reviewer prompt content outside deterministic eval artifacts.

## Acceptance

- [x] `tests/foundry-evals.sh --suite reviewer-fanout` passes.
- [x] `tests/foundry-evals.sh` passes all suites.
- [x] `tests/validate-adversarial-modules.sh` includes and passes reviewer fan-out anchors.
- [x] `docs/HANDOFF.md` documents current suite/case counts.

## Validation

2026-05-29:

- `tests/foundry-evals.sh --suite reviewer-fanout` — passed 3/3 cases.
- `tests/foundry-evals.sh` — passed 6 suites / 21 cases total.
- `tests/validate-adversarial-modules.sh` — passed 87/87 with reviewer fan-out anchors.
- `tests/validate-agents.sh` — passed 224/224.
