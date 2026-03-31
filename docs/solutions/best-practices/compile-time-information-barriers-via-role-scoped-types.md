---
title: "Compile-time information barrier enforcement via role-scoped context types"
module: foundry-engine
date: 2026-03-31
problem_type: best_practice
component: tooling
severity: high
applies_when:
  - "Building adversarial or competitive multi-agent workflows"
  - "Enforcing information separation between roles or phases"
  - "Working with AI agents that suppress uncertainty and ignore soft constraints"
tags:
  - information-barrier
  - type-safety
  - adversarial-workflow
  - compile-time-enforcement
  - agent-reliability
  - role-scoped-context
---

# Compile-Time Information Barrier Enforcement via Role-Scoped Context Types

## Context

In the adversarial red/green workflow, the green team must never see test code, and the red team must never see implementation code. The information barrier is the core invariant — violating it defeats the purpose of adversarial development.

## Guidance

Use role-scoped context types that structurally cannot include wrong artifacts:

- `RedContext { spec, contract, workspace }` — red team sees spec, contract, test workspace
- `GreenContext { spec, contract, workspace, test_outcomes }` — green sees spec, contract, implementation workspace, outcome labels only
- `GreenReviewerContext { spec, contract, workspace, test_outcomes }` — reviewer sees implementation + outcomes, NOT test code
- `RedReviewerContext { spec, contract, test_workspace }` — reviewer sees test code, NOT implementation

The `CliRunner` receives a typed context and passes it to a prompt template. A prompt for green cannot reference red's test workspace because `GreenContext` has no field for it. Barrier violations become compile-time type errors, not runtime bugs.

## Why This Matters

Research on agent behavior (third-thoughts project, 85.5% risk suppression finding) shows agents suppress uncertainty 85.5% of the time and will not self-enforce barriers when instructed via prompts. A prompt saying "do not look at the test code" is unreliable — the agent may comply, may not, and you can't audit it.

Filesystem isolation provides runtime enforcement. Typed contexts provide compile-time enforcement. Together they make barrier violations structurally impossible rather than merely discouraged.

## When to Apply

- Any multi-agent workflow where roles have different information access
- Any system where "don't look at X" is a security or correctness requirement
- When building review systems where reviewer independence matters

## Examples

**Build output leakage:** Compilation errors in the runner workspace can leak test structure to the green agent if error messages reference test file names or assertion content. Solution: compilation failures produce a generic `"compilation: Error"` outcome for green-role contexts; full diagnostic output is only included in red-role contexts. This is enforced at the context-builder level, not via prompt instructions.

**Prompt construction:** The `CliRunner` calls `workspace.green_context(outcomes)` which returns a `GreenContext`. The prompt template receives this typed context — it can reference `context.spec`, `context.contract`, `context.workspace`, and `context.test_outcomes`, but there is no `context.test_workspace` field to accidentally include.
