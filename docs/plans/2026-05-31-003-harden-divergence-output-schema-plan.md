---
title: Harden divergence evaluator output schema
created: 2026-05-31
status: completed
completed: 2026-05-31
---

# Harden Divergence Evaluator Output Schema Plan

## Goal

Eliminate ambiguity surfaced by the live Pi dispute-route smoke: the divergence evaluator correctly returned `findings[0].outcome = VALUABLE`, but also emitted a noncanonical helper `route_to = NLSPEC_REDERIVATION`. The orchestrator already routes only on `findings[0].outcome`; tighten the prompt and deterministic evals so future runs avoid or mechanically flag extra routing fields.

## Scope

- Update `plugins/foundry/agents/review/divergence-evaluator.md` with a strict output contract: no `route_to`, no top-level `outcome`, no non-schema routing fields.
- Update divergence routing docs/playbooks to say extra route fields are ignored/schema drift and must not drive orchestration.
- Update deterministic workflow evals so mocked divergence output omits `route_to` and validation rejects it.
- Update module anchors and handoff.

## Non-goals

- No change to arbiter output; arbiter still uses `route_to` by design.
- No rewriting historical live run output; it remains useful as the captured example that motivated this hardening.
- No private engine changes.

## Acceptance

- [x] Divergence evaluator prompt forbids `route_to`/extra routing fields and keeps `findings[0].outcome` as the sole routing surface.
- [x] Divergence eval adapter rejects top-level `outcome` and `findings[0].route_to`.
- [x] Existing generic evals pass.
- [x] Agent/module validators pass.
- [x] Handoff documents the hardening.

## Validation

2026-05-31:

- `tests/foundry-evals.sh --suite divergence-routing` — passed 6/6 cases, including route-helper schema-drift self-checks.
- `tests/foundry-evals.sh --suite phase-choreography` — passed 3/3 cases.
- `tests/foundry-evals.sh` — passed 8 generic suites / 28 cases.
- `tests/validate-adversarial-modules.sh` — passed 106/106 checks.
- `tests/validate-agents.sh` — passed 224/224 checks.
