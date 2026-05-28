---
title: Gherkin-driven arbiter eval harness
created: 2026-05-26
status: completed
completed: 2026-05-26
---

# Gherkin-Driven Arbiter Eval Harness Plan

## Goal

Make arbiter/process testing reusable as an eval suite: mocked red/green/arbiter scenarios are authored in a small Gherkin subset, then replayed deterministically without live model calls.

## Scope

- Add a `.feature` fixture that describes representative arbiter disputes and mocked arbiter outputs per scenario.
- Add `tests/arbiter-routing-evals.sh` to parse the feature table, generate PromptEnvelope artifacts in a temp run, call `tests/validate-barrier-envelopes.sh`, and verify expected routing/follow-up barriers.
- Cover at least `TEST_WRONG`, `IMPLEMENTATION_WRONG`, and `SPEC_INCOMPLETE`; include `INCONCLUSIVE` if cheap.
- Add validator/docs/handoff references.

## Acceptance

- [x] Gherkin feature fixture is human-readable and reusable.
- [x] Eval runner validates generated arbiter PromptEnvelopes.
- [x] Eval runner validates mocked arbiter JSON outcomes/routes.
- [x] Green follow-up for implementation-wrong cases is checked with PASS/FAIL-only barrier validation.
- [x] Fast validators pass.

## Validation

2026-05-26:

- `tests/arbiter-routing-evals.sh` — 4/4 eval cases passing (`TEST_WRONG`, `IMPLEMENTATION_WRONG`, `SPEC_INCOMPLETE`, `INCONCLUSIVE`).
- `tests/validate-adversarial-modules.sh` — 54/54 passing.
- `tests/validate-barrier-envelopes.sh` — self-tests passing.
- `tests/validate-pi-extension.sh` — 43/43 passing.
- `tests/validate-agents.sh` — 224/224 passing.
- `tests/validate-codex-plugin.sh` — 44/44 passing.
- `tests/validate-behavioral-smoke-contract.sh` — 7/7 passing.
