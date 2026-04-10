---
title: "Grep anchors in skill docs: context labels double as test assertions"
module: foundry
date: 2026-04-08
problem_type: test_fragility
component: adversarial-skill
severity: medium
status: active
tags: [testing, skill-docs, grep, phase-labels, red-team-tests]
---

## Problem

Red team test scripts assert on patterns like `Phase 2b.*VALUABLE` using single-line grep. When editing skill docs, removing what looks like a redundant context label (e.g., "Phase 2b `VALUABLE`" → just "`VALUABLE`") silently breaks the assertion — because grep is single-line and can't infer the surrounding section heading.

### Concrete example

In `foundry-adversarial/SKILL.md`, the Phase 2b routing section originally read:

```
- Phase 2b: `VALUABLE` → invoke spec_update_and_restart, then restart Phase 1
- Phase 2b: `INCONCLUSIVE` → escalate to user (UserEscalation)
```

During PR #1 review cleanup, these were simplified to:

```
- `VALUABLE` → invoke spec_update_and_restart, then restart Phase 1
- `INCONCLUSIVE` → escalate to user (UserEscalation)
```

The section is already inside `### Phase 2b: Test-Fix Inner Loop`, so the label seems redundant. But `tests.sh` checks:

```bash
grep -q "Phase 2b.*VALUABLE"
grep -q "Phase 2b.*INCONCLUSIVE"
```

Both fail silently after the edit. The section heading is not on the same line as VALUABLE/INCONCLUSIVE, so grep never matches.

## Fix

Keep the phase label on the same line as the outcome keyword:

```
- Phase 2b `VALUABLE` → invoke spec_update_and_restart, then restart Phase 1
- Phase 2b `NOT_VALUABLE` → send green back with findings[0].rationale
- Phase 2b `INCONCLUSIVE` → escalate to user (UserEscalation)
```

This satisfies both the human reader (clear which phase) and the grep assertion (both terms on one line).

## Rule

**When editing routing/outcome sections in skill docs, never remove phase labels from outcome lines.** What reads as redundant context in prose is a load-bearing grep anchor in the test suite. Verify test passage before and after any such edit.

## Where this pattern appears

Any test script that asserts `PhaseX.*OUTCOME` or `OUTCOME.*PhaseX` on a single grep line. In Foundry, these are concentrated in `examples/*/red/tests.sh` and `tests/validate-agents.sh`.
