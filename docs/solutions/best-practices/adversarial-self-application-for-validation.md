---
title: "Adversarial self-application — use the red/green pattern to validate itself"
module: foundry-workflow
date: 2026-03-31
problem_type: best_practice
component: testing_framework
severity: medium
applies_when:
  - "Validating an adversarial development methodology"
  - "Building a test-first workflow engine"
  - "Implementing a new runner or agent integration"
tags:
  - adversarial-workflow
  - red-green
  - meta-testing
  - dogfooding
  - information-barrier
---

# Adversarial Self-Application — Use the Red/Green Pattern to Validate Itself

## Context

During Foundry's implementation, the red/green adversarial pattern was applied to implement the CliRunner itself — the component that orchestrates the adversarial workflow. This served as a manual dogfood of the methodology before the engine automation was complete.

## Guidance

The adversarial red/green pattern works even when orchestrated manually by a parent agent (no engine required):

1. **Red team subagent** writes tests based solely on the spec. It never sees implementation code.
2. **Green team subagent** implements based on the spec + test outcome labels only. It never sees test code.
3. **Orchestrating agent** mediates: runs tests, filters outcomes to `test_name: PASS/FAIL`, sends filtered results to green.

Result from the Foundry session: 19/19 tests passed on the green team's first implementation.

## Why This Matters

This validates two things simultaneously:
1. The adversarial methodology produces correct implementations from spec alone
2. The information barrier (outcome labels only) provides sufficient signal for the implementer

It also provides a template for testing new runner integrations — before wiring up the engine, validate the runner works with manual orchestration.

## When to Apply

- Before the engine is complete (manual orchestration)
- When adding a new CLI provider or runner implementation
- When validating changes to the information barrier model
- First dogfooding target for a new Foundry installation

## Examples

**Critical constraint:** The green team must receive **only** `test_name: PASS/FAIL`. Including assertion text, expected values, stack traces, or `.feature` file content collapses the information barrier and turns the adversarial protocol into conventional TDD, losing the independent verification property.

**What the green team prompt looks like:**
```
Test results:
  test_cli_runner_dispatches_red_to_red_workspace: FAIL
  test_cli_runner_dispatches_green_to_green_workspace: FAIL
  test_green_prompt_contains_no_test_code: FAIL

19 tests total, 0 passed, 19 failed.
```
