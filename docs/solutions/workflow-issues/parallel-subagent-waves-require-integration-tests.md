---
title: "Parallel subagent implementation waves require composition tests"
module: foundry-workflow
date: 2026-03-31
problem_type: workflow_issue
component: development_workflow
severity: high
applies_when:
  - "Implementing a large feature with parallel subagent waves"
  - "Building multi-phase state machines with independent units"
  - "Multiple agents implementing different modules concurrently"
tags:
  - parallel-subagents
  - integration-gaps
  - composition-testing
  - state-machine
  - implementation-waves
---

# Parallel Subagent Implementation Waves Require Composition Tests

## Context

Foundry's adversarial engine was implemented in 11 units across 4 parallel waves. Units 1, 5, 7 (no dependencies) ran first, then Units 2, 4, then Units 3, 6, 9, then Units 8, 10, 11. Each wave's units ran as parallel subagents. Every unit passed its own tests. The system had 148 tests, all green.

But it couldn't drive an adversarial workflow end-to-end.

## Guidance

**Per-unit correctness does not guarantee integration correctness.** When implementing in parallel waves:

1. **Write the "drives to completion" test first (red).** Before any implementation, write a single e2e test that starts from initial state and drives through every phase to terminal state. This test is the last thing that goes green.

2. **Test the composition seams, not just the units.** The bugs were all at the seam between units: `apply_outcome` routing actions to the wrong engine functions, stale completion detection discarding valid siblings, phase transitions not blocking dependent phases. No per-unit test can catch these.

3. **The adversarial red/green pattern applies to infrastructure too.** When we used the red/green approach for the CliRunner (red team wrote 19 tests before implementation existed, green team implemented from outcome labels only), the result was 19/19 first-pass. The pattern catches integration gaps that parallel implementation misses.

## Why This Matters

The failure mode is insidious: all tests pass, the code compiles, each component works in isolation, but the system doesn't function. The "driveshaft isn't connected" — the engine produces actions that the outcome handler can't consume. This only surfaces in a test that exercises the full loop.

The cost of adding one e2e composition test is minimal. The cost of discovering the gap in production (or during a review that catches 6 P0/P1 bugs) is high.

## Examples

**What happened:** `apply_outcome` routed all `ExecuteIteration` actions through `complete_execute_iteration`, which rejected `Adversarial` state with `SplitExecutionExpected`. The fix required `apply_outcome` to dispatch on `action.role` — a 30-line change that no unit test could have caught.

**What would have caught it:** A test that calls `drive_concurrent` with an adversarial workflow blueprint, scripts a MockRunner for each phase, and asserts the workflow reaches `Done`.
