---
title: Arbiter agent for red/green test disputes
origin: 2026-03-29-red-green-adversarial-workflow brainstorm
priority: future
---

# Arbiter Agent

An ephemeral agent that performs a controlled, scoped breach of the information barrier to arbitrate whether a failing test is wrong or the implementation is wrong.

## Scope

The arbiter sees: spec + contract + implementation code + one specific test + its result.
It judges: "is the test wrong or the implementation wrong?" and reports back to the orchestrator.

The arbiter is scoped to a single test at a time — it does not get a holistic view of the full test suite or full implementation.

## Triggers

- Green team fails a test repeatedly (exceeds retry threshold)
- Green team passes a test "too easily" (replacement for naive first-n-times heuristic)

## Design Considerations

- The arbiter must be ephemeral — no persistent context across invocations
- Its judgment feeds back into which team iterates next (red fixes test vs green fixes implementation)
- Prompt injection risk: green team could embed instructions in code comments or variable names aimed at influencing the arbiter. Mitigation strategies TBD.
