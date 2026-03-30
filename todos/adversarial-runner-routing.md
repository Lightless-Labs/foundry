---
title: Route adversarial actions through correct engine functions in drive_concurrent
origin: 2026-03-30 ce:review finding P0-2
priority: p1
status: ready
---

# Adversarial Runner Routing

`apply_outcome` in `runner.rs` routes all `ExecuteIteration` completions through `complete_execute_iteration`, which rejects `ImplementationState::Adversarial` with `SplitExecutionExpected`. The adversarial workflow requires routing through `start_red_authoring`, `submit_red_for_review`, `complete_red_iteration`, `complete_green_test_run`, `complete_green_review`, and `complete_red_review` based on `action.role`.

## What needs to change

1. `apply_outcome` must dispatch on `action.role` for `ExecuteIteration` actions:
   - `Some(Red)` → red-team engine functions
   - `Some(Green)` → green-team engine functions (including test run)
   - `None` → existing `complete_execute_iteration` path

2. The `Runner` trait may need additional methods or the CliRunner needs to drive the multi-step adversarial protocol (authoring → review → iteration) within a single `on_execute_iteration` call.

3. Green test outcomes from `run_tests_for_green` must be fed back to the engine via `complete_green_test_run` rather than discarded.

## Related findings

- P0-3: CliRunner hardcodes all outcomes (deferred — prompt engineering)
- P1-4: Green test outcomes discarded
- P2-10: FinalReview role=None for adversarial tasks
