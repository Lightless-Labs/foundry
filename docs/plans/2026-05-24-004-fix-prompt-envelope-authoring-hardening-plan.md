---
title: PromptEnvelope authoring hardening after Pi Roman numeral run
created: 2026-05-24
completed: 2026-05-24
status: completed
---

# PromptEnvelope Authoring Hardening Plan

## Goal

Convert the concrete gaps from `runs/pi-from-scratch-roman-numeral/` into small, executable hardening:

1. Catch withheld samples that are actually allowed PASS/FAIL outcome labels.
2. Document a replay-safe Pi run continuation pattern for timeouts or interrupted reviewer fan-out.

## Evidence

The from-scratch Pi Roman numeral run proved the workflow, but surfaced two operational issues:

- The outer shell timeout interrupted Phase 3 reviewer fan-out, requiring manual continuation from PromptEnvelope artifacts.
- A continuation envelope accidentally used a test name/outcome-label fragment as a withheld sample; `foundry_team` rejected it, but the error can be clearer and validators can catch it earlier.

## Implementation

- Add outcome-label sample-quality checks to `tests/validate-barrier-envelopes.sh` self-tests and runtime validation.
- Add equivalent clearer rejection to `extensions/pi-foundry-team/index.ts` before dispatch.
- Add a Pi continuation/resume playbook and reference it from the adversarial skill / handoff.
- Run fast validators and the Roman numeral replay validators.

## Acceptance

- [x] Validator self-tests include bad outcome-label samples.
- [x] Roman numeral run still validates.
- [x] Pi extension validates.
- [x] Continuation playbook documents the exact safe pattern.
