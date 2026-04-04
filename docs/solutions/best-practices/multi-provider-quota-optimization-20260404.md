---
title: "Multi-provider quota optimization for adversarial workflows"
module: foundry-workflow
date: 2026-04-04
problem_type: best_practice
component: development_workflow
severity: medium
applies_when:
  - "Running adversarial workflows with multiple concurrent agent roles"
  - "Hitting quota limits on single provider"
  - "Wanting maximum independence between red and green contexts"
tags:
  - multi-provider
  - quota-optimization
  - adversarial-workflow
---

# Multi-Provider Quota Optimization for Adversarial Workflows

## Context

Natural mapping — orchestrator on Claude, red team on Gemini/Codex, green team on OpenCode/Codex, reviewers on spare quota.

## Benefits

- **Stronger isolation:** Different providers can't share memory, reinforcing the information barrier
- **Model diversity:** Different models catch different categories of bugs
- **Parallel execution:** No rate limit contention between concurrent agent roles

## Status

Infrastructure ready (gemini-cli, codex-cli, opencode-cli skills) but not fully exercised this session. The mapping is natural and the tooling exists — this is ready to be tested in a future adversarial workflow run.
