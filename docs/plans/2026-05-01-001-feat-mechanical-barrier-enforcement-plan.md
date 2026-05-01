---
date: 2026-05-01
type: feat
status: completed
completed: 2026-05-01
todo: todos/mechanical-barrier-enforcement.md
---

# feat: Mechanical Barrier Enforcement and Replayable Prompt Audits

**Completed:** 2026-05-01 — public plugin now defines PromptEnvelope v1, requires serialized dispatch envelopes in `foundry-adversarial`, includes a replayable envelope validator, and updates the barrier auditor to consume envelope artifacts.

**Plan for:** strengthening Foundry's red/green information barrier in the public plugin layer.

## Problem Frame

The adversarial workflow's central guarantee is the information barrier: red authors tests without seeing implementation, and green implements without seeing red tests, assertions, raw failures, or NLSpec Done criteria. Today the public plugin mostly enforces that guarantee through prose in `foundry-adversarial/SKILL.md` and manual barrier-auditor prompts.

The private engine ultimately needs a first-class `PromptEnvelope` type. In this public plugin repo, we can make the contract explicit, require replayable envelope artifacts from the skill, and add an executable validator that audits envelope artifacts for withheld-content leaks.

## Scope

**In scope:**

- Define the `PromptEnvelope` artifact contract for Foundry dispatches.
- Update `foundry-adversarial` so every subagent dispatch is assembled through a named envelope/redaction gate and serialized under `runs/<run_id>/dispatch/...`.
- Teach the barrier-integrity-auditor to audit envelopes, not only pasted prompt text.
- Add a validation script that can self-test and validate recorded envelope JSON artifacts.
- Update handoff/todo state.

**Out of scope:**

- Implementing the private Rust engine's actual `PromptEnvelope` type.
- Running full behavioral smoke tests; those belong to `todos/behavioral-smoke-tests.md` after envelope capture exists.
- Changing the red/green workflow semantics.

## Requirements

| ID | Description |
|----|-------------|
| R1 | Each adversarial dispatch has a replayable envelope artifact with recipient, phase, prompt, visible context, and withheld context. |
| R2 | Prompt construction passes through a named redaction/validation gate before `Agent(...)` dispatch. |
| R3 | Green envelopes fail validation if withheld red test, assertion, raw failure, or NLSpec Done samples appear in the prompt. |
| R4 | Red envelopes fail validation if withheld green implementation samples or green workspace paths appear in the prompt. |
| R5 | The barrier-integrity-auditor can replay envelope artifacts and report leaks with P0 severity. |
| R6 | Existing structural agent validation remains green. |

## Implementation Units

### Unit 1 — PromptEnvelope contract in adversarial skill

**File:** `plugins/foundry/skills/foundry-adversarial/SKILL.md`

- Add a mechanical barrier gate after workspace setup.
- Define `PromptEnvelope` v1 fields and serialization path.
- Require all subagent prompts to be built through `build_prompt_envelope` + `redact_and_validate_prompt`.
- Replace direct barrier-auditor prompt examples with envelope-based audits.

### Unit 2 — Replayable envelope validator

**File:** `tests/validate-barrier-envelopes.sh`

- Add a script that self-tests good/bad envelope fixtures.
- Add directory/file validation mode for `runs/**/dispatch/**/*.json` artifacts.
- Validate required schema fields and withheld sample non-leakage.

### Unit 3 — Auditor prompt update

**File:** `plugins/foundry/agents/review/barrier-integrity-auditor.md`

- Add PromptEnvelope as the primary audit input.
- Instruct the auditor to compare prompt text against withheld samples/hashes and barrier expectations.

### Unit 4 — State/docs updates

**Files:** `todos/mechanical-barrier-enforcement.md`, `docs/HANDOFF.md`, root instructions as needed.

- Record that public-plugin envelope contract/validator landed.
- Keep engine-side implementation as a follow-up if not completed here.

## Verification

```bash
tests/validate-agents.sh
tests/validate-barrier-envelopes.sh
```

Both must exit 0 before commit.
