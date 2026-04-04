---
title: "Red team test data must be verified against authoritative sources"
module: foundry-workflow
date: 2026-04-04
problem_type: best_practice
component: testing_framework
severity: high
applies_when:
  - "Red team writes tests for domains with precise correct answers"
  - "Test positions or inputs are hand-constructed rather than sourced"
  - "Red team test suite has high count but data not independently verified"
tags:
  - red-team
  - test-quality
  - adversarial-workflow
  - test-data-verification
---

# Red Team Test Data Must Be Verified Against Authoritative Sources

## Context

Chess example — 44 tests, 8 had data bugs (impossible mate-in-one, non-stalemate, wrong FEN, wrong move count). Bad test data blames correct implementation. Under information barrier, green team can't question tests — amplifies impact.

## Guidance

Quality of test data outweighs quantity. Red-team-test-reviewer should verify positions against authoritative sources. Every test input should be traceable to a known-good reference, not hand-constructed by the red team agent.

## Why This Matters

When the adversarial workflow enforces an information barrier, the green team has no recourse to challenge test data. A single bad test vector can consume entire iteration cycles as the green team tries to "fix" correct code to match incorrect expectations. Verification at the source is the only reliable defense.
