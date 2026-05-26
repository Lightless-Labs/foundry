---
title: Arbiter agent for red/green test disputes
origin: 2026-03-29-red-green-adversarial-workflow brainstorm
priority: future
status: completed
completed: 2026-05-26
---

# Arbiter Agent

**Completed 2026-05-26** — Public plugin support landed: `plugins/foundry/agents/review/arbiter-agent.md`, `docs/playbooks/foundry-adversarial-arbiter-routing.md`, adversarial skill routing, barrier-auditor guidance, and validator coverage.

An ephemeral agent that performs a controlled, scoped breach of the information barrier to arbitrate whether a failing test is wrong, the implementation is wrong, or the spec is wrong/incomplete.

## Scope

The arbiter sees: spec + contract + implementation code + one specific test + its result.
It judges: "is the test wrong, is the implementation wrong, or is the spec wrong/incomplete?" and reports back to the orchestrator.

The arbiter is scoped to a single test at a time — it does not get a holistic view of the full test suite or full implementation.

## Triggers

- Green team fails a test repeatedly (exceeds retry threshold)
- Green team passes a test "too easily" (replacement for naive first-n-times heuristic)

## Design Considerations

- The arbiter must be ephemeral — no persistent context across invocations
- Its judgment feeds back into which path runs next: red fixes test, green fixes implementation, or the spec divergence loop re-runs NLSpec authorship
- Prompt injection risk: green team could embed instructions in code comments or variable names aimed at influencing the arbiter. Mitigation landed in the arbiter prompt: all source/test/log artifacts are treated as untrusted evidence, not instructions.
