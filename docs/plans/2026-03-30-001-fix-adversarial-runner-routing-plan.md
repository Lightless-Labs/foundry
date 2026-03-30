---
title: "fix: Connect adversarial engine to drive_concurrent via role-aware routing"
type: fix
status: completed
date: 2026-03-30
origin: foundry/todos/adversarial-runner-routing.md
---

# fix: Connect adversarial engine to drive_concurrent via role-aware routing

## Overview

`apply_outcome` in `runner.rs` routes all `ExecuteIteration` completions through `complete_execute_iteration`, which rejects `Adversarial` state. The adversarial state machine works at the unit level but can't be driven end-to-end through `drive_concurrent`. Fix the routing and add an e2e adversarial test.

## Problem Frame

The engine has well-tested adversarial state transitions (`complete_red_*`, `complete_green_*`). The concurrent driver (`drive_concurrent`) works for non-adversarial workflows. But the two aren't connected — `apply_outcome` always calls `complete_execute_iteration` for execute actions, which rejects adversarial state.

## Requirements Trace

- R1. `apply_outcome` dispatches adversarial execute actions to the correct engine functions based on `action.role`
- R2. Green test outcomes from `CliRunner::run_tests_for_green` flow back to the engine via `complete_green_test_run`
- R3. An e2e test drives an adversarial workflow to completion through `drive_concurrent`
- R4. FinalReview for adversarial tasks carries role information

## Scope Boundaries

- No changes to prompt engineering (CliRunner still hardcodes gate results — that's a separate concern)
- No changes to the Runner trait interface
- No changes to the adversarial engine state machine (it's correct)

## Key Technical Decisions

- **Multi-step adversarial protocol mapped to single Runner calls**: The Runner trait has 4 methods (research, plan, execute, review). The adversarial protocol has ~8 steps. Rather than adding Runner methods, `apply_outcome` orchestrates the multi-step protocol: for Red execute, it calls `start_red_authoring` → applies the runner's outcome → `submit_red_for_review`. For Green execute, it calls the runner → feeds outcomes to `complete_green_test_run`. Review routing uses `complete_red_review` or `complete_green_review` based on role.

- **New ActionOutcome variants not needed**: The existing `Execute(ExecuteOutcome)` and `FinalReview(GateResults)` carry enough data. The routing decision is on `action.role`, not the outcome type.

- **CliRunner returns test outcomes in ExecuteOutcome**: The `ExecuteOutcome` already has `gate_results` and `implementation_adjustments`. We add an optional `test_outcomes: Option<Vec<TestOutcome>>` field so the CliRunner can pass test results back through the existing interface without changing the Runner trait.

## Implementation Units

- [ ] **Unit 1: Add test_outcomes to ExecuteOutcome + role-aware apply_outcome**

  **Goal:** Make `apply_outcome` route adversarial actions correctly and carry test outcomes.

  **Files:**
  - Modify: `foundry-2/src/model.rs`
  - Modify: `foundry-2/src/runner.rs`
  - Modify: `foundry-2/src/engine.rs`
  - Test: `foundry-2/tests/e2e_workflows.rs`

  **Approach:**
  - Add `test_outcomes: Option<Vec<TestOutcome>>` to `ExecuteOutcome`
  - Update all `ExecuteOutcome` construction sites to include `test_outcomes: None`
  - In `apply_outcome`, for `ActionOutcome::Execute`: check `action.role`:
    - `Some(Red)` → `start_red_authoring` then `complete_red_iteration` (the runner's `on_execute_iteration` call represents the full red authoring cycle)
    - `Some(Green)` → if outcome has test_outcomes, call `complete_green_test_run`; otherwise fall through to standard execute
    - `None` → existing `complete_execute_iteration`
  - For `ActionOutcome::FinalReview`: check `action.role`:
    - `Some(Red)` → `complete_red_review`
    - `Some(Green)` → `complete_green_review`
    - `None` → existing `complete_final_review`
  - Add `next_actions` adversarial FinalReview: emit role-specific review actions

  **Test scenarios:**
  - Happy path: adversarial e2e through drive_concurrent — red authors tests, green implements, tests pass, both reviewers approve, done
  - Happy path: green fails tests, iterates, passes on retry
  - Error: drive_concurrent with adversarial workflow reaches Done state

- [ ] **Unit 2: CliRunner feeds test outcomes back**

  **Goal:** `CliRunner::on_execute_iteration` for green passes test outcomes into `ExecuteOutcome` instead of discarding them.

  **Files:**
  - Modify: `foundry-2/src/cli_runner.rs`
  - Test: `foundry-2/tests/cli_runner_tests.rs`

  **Approach:**
  - In `on_execute_iteration` for Green: capture `run_tests_for_green` result, put outcomes in `ExecuteOutcome::test_outcomes`
  - In `green_context`: pass previous test outcomes from engine state (requires reading them from ActionRequest context or storing in CliRunner)

## Sources & References

- Origin: `foundry/todos/adversarial-runner-routing.md`
- Review findings: P0-2, P0-3, P1-4, P2-10
