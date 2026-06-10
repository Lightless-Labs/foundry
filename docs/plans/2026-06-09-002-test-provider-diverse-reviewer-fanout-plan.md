---
title: Exercise provider-diverse Phase 3 reviewer fan-out
created: 2026-06-09
status: completed
completed: 2026-06-09
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

- [x] Phase 3 reviewer PromptEnvelopes exist under `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch/phase3/`.
- [x] `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch` passes before dispatch.
- [x] Live reviewer dispatches complete through `foundry_team` with recorded actual model lanes.
- [x] Reviewer outputs are preserved under the run directory.
- [x] Barrier and behavioral validators still pass for the run.
- [x] Handoff, todo, and run README/results are updated with the Phase 3 evidence.

## Validation Log

2026-06-09:

- Wrote Phase 3 PromptEnvelopes for `green-team-reviewer`, `red-team-test-reviewer`, `rust-reviewer`, and `barrier-integrity-auditor` under `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch/phase3/`.
- `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch` — passed before dispatch after replacing bad poison samples that duplicated allowed PASS/FAIL labels.
- `foundry_team` batch dispatch — succeeded `4/4` with explicit model overrides: green-team-reviewer and rust-reviewer on `kimi-coding/kimi-for-coding`, red-team-test-reviewer and barrier-integrity-auditor on `minimax/MiniMax-M3`.
- The initial MiniMax red-team-test-reviewer dispatch returned no output despite a successful child exit; retried via `dispatch/phase3/red-team-test-reviewer-r2.json`.
- `foundry_team` red-team-test-reviewer r2 dispatch — succeeded with actual model `minimax/MiniMax-M3`; returned parseable JSON wrapped in Markdown fences. This is preserved as a MiniMax structured-output obedience anomaly, not a clean JSON-compliance pass.
- Reviewer outputs were preserved under `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/reviews/`.
- `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch` — passed for all red/green and Phase 3 reviewer envelopes.
- `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke` — passed; final output included `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/behavioral-smoke.toon: PASS` with `model_lanes[8]`.
- Fast gates passed after artifact updates: `tests/validate-behavioral-smoke-contract.sh` 9/9, `tests/validate-pi-extension.sh` 45/45, `tests/validate-codex-plugin.sh` 44/44, and `tests/validate-agents.sh` 224/224.
