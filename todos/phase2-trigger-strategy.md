---
title: "Re-assess Phase 2 divergence trigger strategy"
origin: "2026-04-06 forge brainstorm — spec divergence feedback loop"
priority: future
status: completed
completed: 2026-05-31
tags:
  - adversarial
  - process
  - spec-divergence
---

# Re-assess Phase 2 Divergence Trigger Strategy

**Completed 2026-05-31** — Chose `adaptive_with_fixed_floor`: keep fixed N=3 as the default audited fallback, and add a pattern-based early trigger at N=2 only when the red test content is unchanged and green has made at least two distinct implementation attempts (`implementation_attempt_hashes`) that still leave the same test failing. Documented in `foundry-adversarial` and `docs/playbooks/foundry-adversarial-divergence-routing.md`; validated by the new `phase2-trigger-strategy` eval suite.

## Previous approach

Fixed N=3: after the same test fails on 3 consecutive green iterations, trigger the divergence evaluator.

## Why this was chosen

Start dumb. Fixed threshold is simple, predictable, and easy to reason about.

## Why to revisit

Pattern-based triggering may be more accurate: if green is actively changing code but a specific test stays red across iterations, that's a stronger signal of API divergence than a raw failure count. The fixed threshold may fire too early (green just needed more iterations) or too late (obvious divergence was missed for 3 cycles).

## Decision

Adopt **Adaptive**.

- Preserve N=3 to keep the default trigger simple, predictable, and easy to audit.
- Escalate one iteration earlier when there is stronger evidence: unchanged red test, two consecutive failures, and at least two distinct green implementation hashes.
- Do not early-trigger on first failures, unchanged green implementations, or after red test content changes.
- Keep green feedback PASS/FAIL-only; the implementation-hash evidence is orchestrator-side trigger state and is not sent as raw failure detail.

## Alternatives evaluated

- **Pattern-based only** — rejected as too implicit without a fixed fallback.
- **Immediate** — rejected as too expensive and likely to over-trigger on ordinary first-pass implementation bugs.
- **Adaptive** — accepted: fixed N as a floor, but escalate earlier if pattern suggests divergence.
