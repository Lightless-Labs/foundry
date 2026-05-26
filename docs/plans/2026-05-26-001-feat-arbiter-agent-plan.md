---
title: Arbiter agent for scoped red/green disputes
created: 2026-05-26
status: completed
completed: 2026-05-26
---

# Arbiter Agent Plan

## Goal

Add a scoped, ephemeral arbiter process for single-test red/green disputes without weakening Foundry's default information barrier.

The arbiter is a controlled exception: it may see the full spec/NLSpec, one test artifact, the relevant implementation snippet, and the test result for exactly one disputed test. Its output goes only to the orchestrator, which routes follow-up work back through red, green, or the spec-divergence loop.

## Scope

- Add a `arbiter-agent` review prompt with strict single-test scope, prompt-injection cautions, and JSON routing output.
- Add an adversarial playbook that defines when to invoke arbitration, what a PromptEnvelope may contain, and how to route each outcome.
- Reference the playbook from `foundry-adversarial` Phase 2b / troubleshooting without sending arbiter details to red or green.
- Extend validation so the new agent count and arbiter coverage are mechanically checked.
- Update handoff/todo state after validation.

## Non-goals

- No private engine/runtime implementation in this public plugin repo.
- No broad context breach: arbiter sees one dispute, not whole red/green histories.
- No automatic forwarding of arbiter findings to red or green; the orchestrator must redact and route appropriate instructions.

## Acceptance

- [x] `tests/validate-agents.sh` passes with the arbiter included.
- [x] Arbiter prompt includes outcome routing for `TEST_WRONG`, `IMPLEMENTATION_WRONG`, `SPEC_INCOMPLETE`, and `INCONCLUSIVE`.
- [x] Adversarial skill references scoped arbitration while preserving PASS/FAIL-only green feedback.
- [x] Handoff and todo reflect completion or remaining follow-up.

## Validation

2026-05-26:

- `tests/validate-agents.sh` — 224/224 passing.
- `tests/validate-adversarial-modules.sh` — 42/42 passing.
- `tests/validate-pi-extension.sh` — 41/41 passing.
- `tests/validate-codex-plugin.sh` — 44/44 passing.
- `tests/validate-behavioral-smoke-contract.sh` — 7/7 passing.
- `tests/validate-barrier-envelopes.sh` — self-tests passing.
