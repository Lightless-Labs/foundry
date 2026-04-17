---
title: "Orchestrator discipline: fixture verification, inline-fix carveout, correction verification, fallibility taxonomy"
module: foundry
date: 2026-04-10
problem_type: best_practice
component: adversarial-orchestration
severity: medium
status: accepted
source_repo: Lightless-Labs/third-thoughts
tags:
  - foundry
  - orchestrator
  - fixture-discipline
  - barrier-integrity
  - adversarial-workflow
---

# Orchestrator Discipline

Four operational rules for the orchestrator role in the foundry adversarial workflow, derived from observed failure modes during middens Batch 4 (third-thoughts, 2026-04-06).

---

## 1. Sanity-check the fixture before routing failures as green-team bugs

**Symptom:** Cucumber reports `0 X found` immediately after the orchestrator wrote or modified a fixture step. The failure looks like a green-team bug — wrong aggregation, wrong extraction logic.

**Root cause:** Fixture bugs and implementation bugs produce identical symptoms. The fixture may not be injecting the inputs the assertions imply (e.g. off-by-one on message index means injection silently never fires).

**Rule:** Before routing any test failure as a green-team bug, verify the fixture is actually producing the expected inputs:

1. Dump the first session (or a representative sample) of the fixture input
2. Confirm the expected signals are present in the raw data
3. Only after verifying the fixture, route as a green-team bug

**Heuristic:** If the failing assertion is "X should exist in output" and actual output shows "0 X found," the fixture is the first suspect — especially when the failure surfaces immediately after the orchestrator has written or modified a fixture step.

Cost: ~30 seconds per check. Savings: one full green re-dispatch cycle per mistaken routing.

---

## 2. Inline fixes are acceptable for green-team bugs with no algorithmic content

The playbook default is strict: never read both sides and fix code directly. This is correct in principle but leaves a gap — there is no middle ground between "route pass/fail to green" and "god-mode fix."

**The real distinction is algorithmic content.**

**Allowed inline:**
- Literal typos (variable name misspellings)
- Field-name drift matching a known NLSpec correction the orchestrator already made
- Off-by-one at a loop boundary when the intended bound is obvious from surrounding code
- Missing `if __name__ == '__main__':` guards
- Imports left out but clearly needed by otherwise-complete code

**Must go back to green:**
- Wrong aggregation (sum vs mean, product vs sum)
- Wrong iteration order
- Missing edge cases (empty input, single element, zero divisor)
- Wrong threshold comparisons (`>=` vs `>`)
- Missing normalization or sanitization
- Missing fallback branches

**The test:** could I explain this fix to green in a single pass/fail label? If no, and the fix is genuinely mechanical, inline is acceptable — but **log it in the retrospective** so the adversarial guarantee stays auditable.

---

## 3. Post-correction verification: use `grep -n`, not `grep -c`

After correcting a term in the NLSpec or shared contract (e.g. a field name rename), verify ALL reference files before dispatching.

**Wrong:**
```bash
grep -c "old_term" spec.md contract.md prompts/
```
`-c` reports a per-file count. One file with a subtly different encoding or quote character can show `0` while still containing the old term — you won't know which line.

**Right:**
```bash
grep -n "old_term" spec.md contract.md prompts/ && \
  echo "RESIDUAL MATCHES — fix before dispatch" || \
  echo "clean"
```

`-n` gives line-level visibility. Confirms zero actual hits across the full set.

When in doubt, regenerate the green prompt files from scratch rather than patching them.

---

## 4. Orchestrator fallibility is a first-class failure category

The standard failure taxonomy (contract gap / red bug / green bug / convention mismatch / ambiguous spec) implicitly assumes the orchestrator is infallible. When the orchestrator is an LLM, this assumption is soft.

**Add to triage:** before classifying any failure, ask — did the orchestrator get something wrong?

**Common orchestrator-side failure modes:**
- **Stale prompt files:** A correction in the canonical NLSpec didn't propagate to the green prompt attachment
- **Bad fixtures:** The fixture step doesn't produce the inputs the assertions imply (see §1)
- **Wrong dispatch command:** CLI argument order, stale model ID, missing `--format json`
- **Wrong routing:** Classified a contract gap as a green-team bug and re-dispatched, wasting a cycle
- **Inline fix without logging:** Fixed a green-team bug inline that had algorithmic content, breaking provenance (see §2)

**Self-reporting is the primary defense — the orchestrator has no external audit.** The retrospective for each adversarial run should enumerate orchestrator errors alongside red/green errors.

Budget: ~1 minute per failure for the orchestrator-side checklist. Savings: avoided re-dispatch cycles and preserved barrier integrity.
