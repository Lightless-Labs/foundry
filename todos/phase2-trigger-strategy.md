---
title: "Re-assess Phase 2 divergence trigger strategy"
origin: "2026-04-06 forge brainstorm — spec divergence feedback loop"
priority: future
status: open
tags:
  - adversarial
  - process
  - spec-divergence
---

# Re-assess Phase 2 Divergence Trigger Strategy

## Current approach

Fixed N=3: after the same test fails on 3 consecutive green iterations, trigger the divergence evaluator.

## Why this was chosen

Start dumb. Fixed threshold is simple, predictable, and easy to reason about.

## Why to revisit

Pattern-based triggering may be more accurate: if green is actively changing code but a specific test stays red across iterations, that's a stronger signal of API divergence than a raw failure count. The fixed threshold may fire too early (green just needed more iterations) or too late (obvious divergence was missed for 3 cycles).

## Alternatives to evaluate

- **Pattern-based** — detect when implementation changes aren't moving a specific test's status
- **Immediate** — run evaluator on every failure (expensive but maximally sensitive)
- **Adaptive** — fixed N as a floor, but escalate earlier if pattern suggests divergence
