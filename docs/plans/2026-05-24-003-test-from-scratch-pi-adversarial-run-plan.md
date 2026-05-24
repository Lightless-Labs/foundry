---
title: From-scratch Pi adversarial feature run
created: 2026-05-24
completed: 2026-05-24
status: completed
---

# From-Scratch Pi Adversarial Feature Run Plan

## Goal

Prove the Foundry Pi package can run a fresh non-example adversarial feature where red and green generate new artifacts from an NLSpec, rather than replaying an existing worked example.

## Candidate Feature

Rust `roman-numeral` library/CLI:

- Convert integers `1..=3999` to canonical Roman numerals.
- Parse canonical Roman numerals back to integers.
- Reject invalid/non-canonical forms.
- Use explicit golden vectors to avoid convention drift.

## Constraints

- Use Pi `/skill:foundry-adversarial` and `foundry_team` where possible.
- Preserve PromptEnvelope artifacts for every red/green/reviewer dispatch.
- Green PromptEnvelopes expose only NLSpec How plus `test_name: PASS/FAIL` labels.
- Red and green artifacts must be freshly generated, not copied from existing examples.
- Final run directory must pass:
  - `tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch`
  - `tests/behavioral-smoke.sh runs/<run_id>`

## Steps

1. Create a small throwaway Rust project and reviewed minimal NLSpec with golden vectors.
2. Invoke Pi with the Foundry skill adapter and package extension enabled.
3. Capture run artifacts under `runs/<run_id>/`.
4. Run replay validators.
5. Record obedience/workflow gaps in the todo and handoff.

## Acceptance

- [x] New run artifacts exist under `runs/<run_id>/`.
- [x] Red/green artifacts are fresh.
- [x] Barrier and behavioral validators pass.
- [x] Any gap is documented with a follow-up.

## Outcome

Run artifacts: `runs/pi-from-scratch-roman-numeral/`.

Final result: 8/8 red tests pass. Both replay validators pass.

Documented follow-up observations in `todos/modularize-heaviest-skills.md`: use longer/resumable live orchestration for Phase 3 reviewer fan-out and add/helper-test withheld-sample derivation so allowed PASS/FAIL test outcome labels are not accidentally treated as forbidden samples.
