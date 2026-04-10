---
title: "Ephemeral evaluator output shape: findings[0], not top-level"
module: foundry
date: 2026-04-08
problem_type: contract_gap
component: divergence-evaluator
severity: medium
status: active
tags: [evaluator, findings, routing, reviewer-schema, divergence-evaluator]
---

## Problem

Foundry ephemeral evaluators (e.g., `divergence-evaluator`) follow the standard reviewer output schema:

```json
{
  "reviewer": "...",
  "findings": [{ "outcome": "...", "rationale": "...", "gap_description": "..." }],
  "residual_risks": [],
  "testing_gaps": []
}
```

The judgment fields live inside `findings[0]`, not at the top level. But the original skill doc referred to `DivergenceJudgment.outcome` and `judgment.rationale` — names that suggest a flat top-level structure. These names diverge from the actual access path as soon as the evaluator is implemented against the reviewer schema.

## Root cause

The evaluator was designed using a named type (`DivergenceJudgment`) before the output format was settled. When the format adopted the reviewer schema for consistency, the prose references were not updated to match.

## Fix

In all routing logic and prose references, use:
- `findings[0].outcome` (not `DivergenceJudgment.outcome`)
- `findings[0].rationale` (not `judgment.rationale`)
- `findings[0].gap_description` (not `judgment.gap_description`)

Explicitly document in the evaluator that `findings` contains exactly one element per invocation — this is the contract that makes `findings[0]` safe to use without bounds-checking.

## Rule

**When adding a new ephemeral evaluator, document `findings` as a single-element array at the point of definition.** All routing logic referencing the evaluator output must use `findings[0].*`. Never use a hypothetical top-level field name that isn't in the actual schema.

## Where this pattern applies

Any skill that spawns an ephemeral evaluator and routes on its output:
- `foundry:adversarial` Phase 1b and Phase 2b divergence checks
- Any future evaluator added to the adversarial pipeline
