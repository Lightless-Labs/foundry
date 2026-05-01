---
title: "Adversarial process: spec divergence feedback loop"
origin: "2026-04-05 third-thoughts Batch 1 adversarial run; 2026-03-29 red-green-adversarial-workflow brainstorm"
priority: p2
status: merged
completed: 2026-04-08
tags:
  - adversarial
  - process
  - spec-review
---

# Spec Divergence Feedback Loop

**Completed:** 2026-04-08 — merged via PR #1. The public plugin now includes the `divergence-evaluator` agent plus Phase 1b/2b/restart extensions in `foundry-adversarial`.

Divergence from the NLSpec is always evaluated for value. If valuable, the nlspec agent is re-run with enriched input — never patched in place. The spec agent is the only thing that produces a NLSpec.

## The Loop

```
NLSpec
  └─► Red team writes tests
        └─► Review: tests vs spec
              └─► Divergence?
                    ├─► Valuable → re-run nlspec agent (original context + existing NLSpec + feedback)
                    │             → new NLSpec → continue pipeline from red team
                    └─► Not valuable → red team back to drawing board

  └─► Green team implements
        └─► Tests fail?
              └─► API divergence from spec?
                    ├─► Valuable → re-run nlspec agent (original context + existing NLSpec + feedback)
                    │             → new NLSpec → continue pipeline from red team
                    └─► Not valuable → green team back to drawing board
```

## Key Principle

Divergence is signal, not error. A team that deviates from the spec may be surfacing a real gap. The question is always: *is this divergence valuable?*

- If yes: the spec evolves. Re-run nlspec agent with the enriched input. The pipeline restarts from red team with the new NLSpec.
- If no: the team made an error. Send them back.

The spec is never amended directly. The nlspec agent always owns NLSpec authorship.

## Triggers

- **Phase 1b:** red team test references behavior not in the NLSpec
- **Phase 2:** green team exceeds retry threshold on a failing test, and the failure looks like an API contract issue rather than an implementation bug

## Design Considerations

- The divergence evaluator must be **ephemeral** — no persistent context across invocations
- Scoped to one divergence at a time — not a holistic view of the full suite or implementation
- **Prompt injection risk:** a team could embed instructions in code, comments, or variable names to influence the evaluator's judgment. Mitigation strategies TBD.

## Evidence

- Commit `dbf64c8` in third-thoughts: orchestrator fixed red's "State Emission Probabilities" → "State Characteristics" without evaluating whether the spec should change. The nlspec agent was not re-run. Signal was lost.
- NLSpec at `middens/docs/nlspecs/2026-04-05-python-techniques-batch1-nlspec.md` was not updated.
