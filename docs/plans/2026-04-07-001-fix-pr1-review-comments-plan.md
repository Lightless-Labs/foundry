---
date: 2026-04-07
type: fix
status: active
pr: https://github.com/Lightless-Labs/foundry/pull/1
---

# fix: Address PR #1 Review Comments (Spec Divergence Feedback Loop)

**Plan for:** Lightless-Labs/foundry#1 review comment resolution
**Reviewers:** CodeRabbit (CHANGES_REQUESTED), Copilot (COMMENTED), Codex (COMMENTED)

## Problem Frame

PR #1 introduced the spec divergence feedback loop: a new `divergence-evaluator` agent plus modifications to `foundry-adversarial` SKILL.md. Three automated reviewers flagged issues across six files. All must be addressed, replied to with rationale, and CI must pass.

## Scope

**In scope:** All P1, P2, and actionable nitpick items from CodeRabbit, Copilot, and Codex.
**Out of scope:** Redesigning the divergence evaluation architecture; changing test counts.

## Requirements

| ID | Source | Priority | Description |
|----|--------|----------|-------------|
| R1 | Codex | P1 | Guard pre-overwrite `git commit` — NLSpec may have nothing staged; `git commit` without staged changes exits non-zero |
| R2 | Codex | P2 | `tests.sh` L485: red_test_paths test block only checks section existence, never validates the parameter string |
| R3 | Copilot | P1 | `divergence-evaluator.md` `name` field: should be `divergence-evaluator`, not `foundry:review:divergence-evaluator` (validate-agents.sh key) |
| R4 | Copilot | P1 | `divergence-evaluator.md` `tools` field: must be comma-separated (`Read, Grep, Glob, Bash`) |
| R5 | Copilot | P1 | SKILL.md routing uses `DivergenceJudgment.outcome` but evaluator output is `findings[0].outcome` — align |
| R6 | Copilot | P1 | SKILL.md Phase 2b: multiple tests may cross threshold simultaneously; spec must state sequential processing with deterministic order (e.g., alphabetical test_id) |
| R7 | Copilot | P1 | SKILL.md Phase 2b EvaluatorInput: missing `test_id` field in `diverging_artifact` |
| R8 | Copilot | P1 | SKILL.md spec_update_and_restart: pre-overwrite commit (step 2) should be deferred until after NLSpec agent succeeds — aligns with "no commit on failure" |
| R9 | Copilot | P2 | `divergence-evaluator.md` gap_description: ambiguous — "present only when VALUABLE" vs "null when not VALUABLE" — pick one form, document invariant explicitly |
| R10 | CodeRabbit | P2 | `docs/HANDOFF.md`: date says 2026-04-04 (should be 2026-04-07), validation count says 207/207 (should be 215/215), agent count stale |
| R11 | CodeRabbit | P2 | `foundry-nlspec/SKILL.md` doesn't document `NLSpecRerunInput` interface — green team/NLSpec agent callers have no contract |
| R12 | CodeRabbit | P2 | `docs/solutions/...md` L92: missing language specifier on code block |
| R13 | CodeRabbit | nitpick | Code blocks without language specifiers (divergence-evaluator.md, SKILL.md) |
| R14 | CodeRabbit | nitpick | `tests.sh` `search_either` is redundant — calls `search_both` with identical semantics |
| R15 | CodeRabbit | nitpick | `tests.sh` L483: `phase2b_section` assigned but never used (shellcheck SC2034) |
| R16 | CodeRabbit | nitpick | `tests.sh` L812-815: magic number `125` unexplained |
| R17 | CodeRabbit | nitpick | `docs/solutions/...md` Python salvage code uses bare `open()` without context manager |
| R18 | CodeRabbit | nitpick | `docs/solutions/...md` commit reference inconsistency (frontmatter `source_commit_range` vs body reference `7858009`) |

## Implementation Units

### Unit 1 — divergence-evaluator.md fixes (R3, R4, R9, R13)

**File:** `plugins/foundry/agents/review/divergence-evaluator.md`

- Fix `name` field to `divergence-evaluator` (strip `foundry:review:` prefix — validate-agents.sh strips it when matching)
- Fix `tools` to comma-separated: `Read, Grep, Glob, Bash`
- Clarify gap_description invariant: "gap_description MUST be present (non-null string) iff outcome == VALUABLE; MUST be absent (omitted or null) otherwise"
- Add `yaml` language specifier to frontmatter code block in output format section

**Test:** `bash tests/validate-agents.sh 2>&1 | grep divergence-evaluator` — must pass

### Unit 2 — SKILL.md routing and Phase 2b fixes (R5, R6, R7, R8, R13)

**File:** `plugins/foundry/skills/foundry-adversarial/SKILL.md`

- Phase 1b and 2b: Change `DivergenceJudgment.outcome` → `findings[0].outcome`
- Phase 2b: Add sentence: "When multiple tests cross threshold simultaneously, process one at a time in ascending test_id order. Do not advance to the next until the current divergence resolves."
- Phase 2b EvaluatorInput: Add `test_id` field alongside `diverging_artifact`
- spec_update_and_restart step 2 (pre-overwrite commit): Move to step 2b — only executed after NLSpec agent returns successfully (step 3). Sequence becomes: cap check → NLSpec agent → pre-overwrite commit → write new NLSpec → post-overwrite commit
- Add `yaml` language specifier to any bare code blocks in added sections
- Change "Never include" inline text to a bullet list

**Test:** `bash examples/spec-divergence-feedback-loop/red/tests.sh 2>&1 | tail -5` — must show 0 failures

### Unit 3 — foundry-nlspec/SKILL.md interface documentation (R11)

**File:** `plugins/foundry/skills/foundry-nlspec/SKILL.md`

- Add a `## Rerun Input (NLSpecRerunInput)` section documenting the three fields: `original_spec_path`, `existing_nlspec_path`, `evaluator_feedback` (verbatim gap_description string, not paraphrased)
- Note caller contract: evaluator_feedback must be the raw `gap_description` string from `DivergenceJudgment`

### Unit 4 — tests.sh cleanup (R2, R14, R15, R16)

**File:** `examples/spec-divergence-feedback-loop/red/tests.sh`

- R2: In the Phase 2b `red_test_paths` test block (L485), add a `grep` assertion that actually checks for the string `red_test_paths` in the skill file (not just section existence)
- R14: Remove `search_either` function entirely; replace any calls with `search_both`
- R15: Remove unused `phase2b_section` variable
- R16: Add comment above `exit 125`: `# 125 = "command not found" reserved exit code — chosen to distinguish test-runner failure from test assertion failures (0 = pass, 1–124 = assertion fails)`

**Test:** `bash examples/spec-divergence-feedback-loop/red/tests.sh 2>&1 | tail -5` — must show 0 failures

### Unit 5 — HANDOFF.md and solutions doc fixes (R10, R12, R17, R18)

**Files:**
- `docs/HANDOFF.md`
- `docs/solutions/workflow-issues/third-thoughts-batch4-feedback-20260406.md`

HANDOFF.md:
- Fix "Last updated: 2026-04-04" → "Last updated: 2026-04-07"
- Fix "Validation: 207/207" → "215/215"
- Update agent count to current value from validate-agents.sh output

solutions doc:
- Fix Python code block to use `with open(path, 'w') as f: f.write(body)`
- Add language specifier to bare code block at L92
- Fix commit reference inconsistency: align body reference with `source_commit_range` frontmatter or add explanatory note

## Sequencing

Units 1 and 2 are independent — run in parallel via subagents.
Unit 3, 4, 5 are independent of each other and of 1/2 — can run in parallel.
All units must complete before running final test validation.

## Verification

```bash
# After all fixes
bash tests/validate-agents.sh 2>&1 | tail -5
bash examples/spec-divergence-feedback-loop/red/tests.sh 2>&1 | tail -5
```

Both must exit 0. Then push and confirm CI is green.
