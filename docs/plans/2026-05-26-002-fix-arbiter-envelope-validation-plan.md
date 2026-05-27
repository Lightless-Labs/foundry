---
title: Arbiter PromptEnvelope validation hardening
created: 2026-05-26
status: completed
completed: 2026-05-26
---

# Arbiter PromptEnvelope Validation Hardening Plan

## Goal

Mechanically validate the new arbiter's controlled information-barrier breach so arbiter dispatches stay scoped to exactly one disputed test.

## Scope

- Extend `tests/validate-barrier-envelopes.sh` with arbiter-specific checks and self-tests.
- Mirror the checks in the Pi `foundry_team` extension so live Pi dispatch rejects unsafe arbiter envelopes before model invocation.
- Extend module/Pi validators with anchors for the new hardening.
- Update handoff with validation results.

## Acceptance

- [x] Good arbiter envelope self-test passes.
- [x] Bad arbiter envelopes for missing `single_test_scope` and over-broad visible context fail.
- [x] Runtime Pi extension contains matching arbiter checks.
- [x] Fast validators pass.

## Validation

2026-05-26:

- `tests/validate-barrier-envelopes.sh` — self-tests passing, including good arbiter, missing `single_test_scope`, and over-broad arbiter context cases.
- `tests/validate-pi-extension.sh` — 43/43 passing.
- `tests/validate-adversarial-modules.sh` — 46/46 passing.
- `tests/validate-agents.sh` — 224/224 passing.
- `tests/validate-codex-plugin.sh` — 44/44 passing.
- `tests/validate-behavioral-smoke-contract.sh` — 7/7 passing.
- `tests/validate-barrier-envelopes.sh runs/pi-autonomous-sudoku-smoke/dispatch` — passing.
- `tests/behavioral-smoke.sh runs/pi-autonomous-sudoku-smoke` — passing.
- `tests/validate-barrier-envelopes.sh runs/pi-from-scratch-roman-numeral/dispatch` — passing.
- `tests/behavioral-smoke.sh runs/pi-from-scratch-roman-numeral` — passing.
