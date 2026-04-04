---
title: "Golden test vectors as convention anchors for adversarial specifications"
module: foundry-workflow
date: 2026-04-04
problem_type: best_practice
component: development_workflow
severity: critical
applies_when:
  - "NLSpec describes state transformations, geometric conventions, or encoding schemes"
  - "Red and green teams must independently derive concrete data from abstract descriptions"
  - "The domain has reference implementations that produce authoritative output"
tags:
  - golden-vectors
  - convention-mismatch
  - adversarial-workflow
  - nlspec
  - spec-completeness
---

# Golden Test Vectors as Convention Anchors

## Context

Three example projects built with adversarial workflow. Sudoku had no convention issue (30/30). Rubik's cube had fatal mismatch without vectors (31/46, 15 unfixable). Chess had vectors that caught both convention errors AND spec derivation bugs (44/44 after fix).

## Guidance

Golden vectors are needed when both teams must independently derive concrete artifacts from abstract descriptions. Good vectors: sourced from reference implementation (not spec author), exercise simplest nontrivial operation, include 3+ vectors of increasing complexity, published in NLSpec DoD.

## Why This Matters

Convention mismatches look like implementation bugs but are spec defects. Both teams are correct within their convention — no iteration fixes it. Golden vectors anchor both teams to the same concrete reality. Cost to include: trivial. Cost to omit: deadlocked process.
