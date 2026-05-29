---
title: Generic Foundry workflow eval framework
created: 2026-05-26
status: completed
completed: 2026-05-29
---

# Generic Foundry Workflow Eval Framework Plan

## Goal

Promote the arbiter-specific Gherkin/mock eval pattern into a reusable deterministic workflow eval framework for Foundry process behavior.

## Scope

- Add a generic eval entrypoint and Python runner.
- Move arbiter eval feature/adaptation into the generic structure without losing wrapper compatibility.
- Add non-arbiter suites for divergence routing, green/red follow-up barriers, and spec update/restart provenance.
- Update validators and handoff to point at the generic command.

## Acceptance

- [x] `tests/foundry-evals.sh` runs all eval suites.
- [x] `tests/foundry-evals.sh --suite arbiter-routing` passes existing arbiter coverage.
- [x] `tests/foundry-evals.sh --suite green-followup-barrier` validates good and bad green follow-up barrier cases.
- [x] `tests/arbiter-routing-evals.sh` remains as a thin compatibility wrapper.
- [x] Module validators check generic eval anchors.
- [x] Fast validators pass.

## Validation

2026-05-29:

- `tests/foundry-evals.sh` — 5 suites / 18 cases passing.
- `tests/arbiter-routing-evals.sh tests/fixtures/arbiter-routing-evals.feature` — compatibility wrapper passing.
- `tests/foundry-evals.sh --suite divergence-routing` — 6/6 passing.
- `tests/foundry-evals.sh --suite red-followup-barrier` — 3/3 passing.
- `tests/foundry-evals.sh --suite spec-update-restart` — 3/3 passing.
- `tests/validate-adversarial-modules.sh` — 81/81 passing.
- `tests/validate-agents.sh` — 224/224 passing.
- `tests/validate-barrier-envelopes.sh` — self-tests passing.
- `tests/validate-pi-extension.sh` — 43/43 passing.
- `tests/validate-codex-plugin.sh` — 44/44 passing.
- `tests/validate-behavioral-smoke-contract.sh` — 7/7 passing.
