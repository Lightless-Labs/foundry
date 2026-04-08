---
title: "Deferred commit pattern: commit after the operation, not before"
module: foundry
date: 2026-04-08
problem_type: implementation_pattern
component: adversarial-orchestration
severity: medium
status: active
tags: [git, spec-update, commit-sequencing, spec_update_and_restart]
---

## Problem

The `spec_update_and_restart` procedure originally committed the current NLSpec (pre-overwrite) **before** running the NLSpec agent. The failure branch then said "do NOT commit; NLSpec unchanged" — contradicting the commit that had already happened in step 2.

This pattern appears whenever a workflow wants to preserve state "before" an operation: the natural instinct is to snapshot first, operate second. But if the operation can fail and you want a clean abort, snapshotting before the operation ties your hands.

## Root cause

The design intent was correct — capture the before-state for audit purposes — but the implementation placed the commit too early. The commit was meant to be a safety net, but it fired unconditionally regardless of whether the agent succeeded.

## Fix

Defer the pre-overwrite commit until after the operation succeeds:

```
1. Run the operation (NLSpec agent)
2. If operation fails → bail out; no commits; state unchanged
3. Commit the before-state (pre-overwrite) — now safe, operation succeeded
4. Write new state (overwrite file)
5. Commit new state (post-overwrite)
```

Also guard the commit against the case where nothing is staged:

```bash
git add <path>
git diff --staged --quiet || git commit --author="..." -m "..."
```

`git commit` with nothing staged exits non-zero, which would abort the orchestrator. The guard makes the commit a no-op if the file hasn't actually changed.

## Where this pattern applies

Any two-commit sequence of the form "preserve before / write after" where the write can fail:
- `spec_update_and_restart` in `foundry:adversarial` (where this was fixed)
- Any workflow that commits a backup before calling an agent that modifies the same file
- Database migration patterns: snapshot table before transformation, only record the snapshot if transformation completes

## Invariant

**The pre-operation commit must be reachable only via the success path of the operation it precedes.**

If the operation has a failure branch, that branch must be unreachable from any point after the pre-operation commit.
