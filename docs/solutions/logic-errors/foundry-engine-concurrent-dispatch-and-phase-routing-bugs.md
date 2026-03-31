---
title: "Concurrent dispatch stale-batch detection and adversarial phase routing"
module: foundry-engine
date: 2026-03-31
problem_type: logic_error
component: tooling
severity: critical
symptoms:
  - "drive_concurrent discards valid completions — same-batch sequence_number check was wrong"
  - "apply_outcome routes adversarial actions through non-adversarial engine functions"
  - "normalize() promotes task phase without checking test_outcomes"
  - "complete_red_iteration accepts calls in any RedPhase, bypassing write lock protocol"
  - "Red review rejection does not re-block green from testing"
root_cause: logic_error
resolution_type: code_fix
tags:
  - state-machine
  - concurrent-dispatch
  - adversarial-engine
  - stale-completion
  - batch-version
  - phase-validation
---

# Concurrent Dispatch Stale-Batch Detection and Adversarial Phase Routing

## Problem

Five related P0/P1 bugs in the adversarial engine's concurrent dispatch and state transition layer, found during code review after parallel implementation. All 148 unit and integration tests passed — these bugs existed in the composition layer between units.

## Symptoms

1. `drive_concurrent` appeared to process only one action per batch, silently discarding siblings
2. Adversarial workflows immediately failed with `SplitExecutionExpected` when driven through `drive_concurrent`
3. Green team could test against stale suite after red review rejection
4. `complete_red_iteration` could be called out of order (during Authoring phase)
5. `normalize()` could promote a task to FinalReview with failing test outcomes

## What Didn't Work

- Per-unit testing: each unit had comprehensive tests that passed, but the bugs lived at the seam between units
- The `sequence_number` approach for stale detection seemed correct in isolation but broke when batch size > 1

## Solution

**Bug 1 — Stale completion detection:** Changed from `completion.sequence_number < state.state_version` to `completion.sequence_number < batch_version`, where `batch_version` is captured once per batch before processing completions. This ensures siblings from the same dispatch batch are never discarded.

**Bug 2 — Adversarial routing:** `apply_outcome` now dispatches on `action.role`:
- `Some(Red)` → `start_red_authoring` → `submit_red_for_review` → `complete_red_iteration`
- `Some(Green)` → `complete_green_test_run` (with test outcomes from runner)
- `None` → existing `complete_execute_iteration` (non-adversarial path unchanged)

**Bug 3 — Green re-blocking:** `complete_red_review` rejection branch now sets `coordination.green_blocked = true`.

**Bug 4 — Phase validation:** `complete_red_iteration` validates `red_phase == InReview` before proceeding.

**Bug 5 — Normalize test check:** Added `all_tests_pass` to `normalize()`'s promotion condition, matching `complete_green_review`.

## Why This Works

The bugs shared a common pattern: each function was correct in isolation but assumed it would be called in a specific sequence that the composition layer didn't enforce. The fixes add explicit validation at each transition point rather than relying on caller discipline.

The batch_version fix distinguishes "which group of work is this from?" (batch identity) from "which state produced this?" (sequence identity). A single counter conflating both always breaks when parallelism > 1.

## Prevention

- **Always include a "drives to completion" e2e test** when building multi-phase state machines. This test starts from initial state, drives through every phase with a mock runner, and asserts terminal state. It catches integration gaps that unit tests structurally cannot find.
- **Validate phase preconditions** in every state transition function, even if the caller "should" always provide the right phase. Defense in depth.
- **Test with batch size > 1** when building concurrent dispatchers. Single-action tests cannot exercise the stale detection path.
