---
title: Exercise provider-diverse Phase 3 reviewer fan-out
created: 2026-06-09
status: active
todo: todos/multi-provider-delegation.md
---

# Exercise Provider-Diverse Phase 3 Reviewer Fan-Out

## Goal

Extend the preserved `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/` run with a live Phase 3 reviewer fan-out using provider/model-diverse Pi child dispatches while preserving reviewer territory boundaries.

## Scope

Dispatch at least these reviewers from serialized PromptEnvelope artifacts:

- `green-team-reviewer` — sees NLSpec How, green implementation, and PASS/FAIL outcome labels only; must not see red test code or NLSpec Done.
- `red-team-test-reviewer` — sees NLSpec/DoD and red tests only; must not see green implementation.
- `rust-reviewer` — sees implementation-facing Rust/Cargo context; must not see red tests or NLSpec Done.
- `barrier-integrity-auditor` — sees dispatch envelope paths and audits the whole run for barrier violations.

Use explicit, distinct provider/model lanes where useful, and record planned/actual lanes in the run artifact.

## Non-goals

- Do not change the Pi dispatch extension, validators, or behavioral-smoke schema.
- Do not require a green fix unless a reviewer reports a material finding.
- Do not paste hidden red/green context into normal Pi messages; all reviewer dispatches must use PromptEnvelope artifacts and `foundry_team`.

## Acceptance

- [ ] Phase 3 reviewer PromptEnvelopes exist under `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch/phase3/`.
- [ ] `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch` passes before dispatch.
- [ ] Live reviewer dispatches complete through `foundry_team` with recorded actual model lanes.
- [ ] Reviewer outputs are preserved under the run directory.
- [ ] Barrier and behavioral validators still pass for the run.
- [ ] Handoff, todo, and run README/results are updated with the Phase 3 evidence.

## Validation Log

Pending.
