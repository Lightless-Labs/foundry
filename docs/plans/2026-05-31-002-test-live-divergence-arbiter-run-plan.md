---
title: Exercise live divergence and scoped arbiter loops
created: 2026-05-31
status: completed
completed: 2026-05-31
---

# Exercise Live Divergence and Scoped Arbiter Loops Plan

## Goal

Produce a replayable run artifact that exercises the two highest-risk Foundry dispute routes with real Pi `foundry_team` child dispatches:

1. Phase 2b divergence evaluator over an ambiguous spec/test/implementation mismatch.
2. Scoped `arbiter-agent` over exactly one disputed test with a controlled information-barrier breach.

## Scope

- Create a small controlled `slugify` contract/run under `runs/pi-live-divergence-arbiter-smoke/`.
- Author PromptEnvelope artifacts for:
  - `foundry:review:divergence-evaluator`
  - `foundry:review:arbiter-agent`
  - barrier-safe red/green follow-up examples after those routes
- Dispatch the divergence evaluator and arbiter through Pi `foundry_team` rather than faking subagent output in the main session.
- Preserve Pi JSONL output, extracted child results, routing summary, and `behavioral-smoke.toon`.
- Validate replay artifacts with `validate-barrier-envelopes.sh` and `behavioral-smoke.sh`.

## Non-goals

- No private engine changes.
- No full product implementation; this is a live dispute-route smoke, not a new worked example.
- No broad context breach: arbiter receives one disputed test and one relevant implementation snippet only.

## Acceptance

- [x] Live Pi dispatch through `foundry_team` completes for both agents.
- [x] `runs/pi-live-divergence-arbiter-smoke/dispatch` validates with `tests/validate-barrier-envelopes.sh`.
- [x] `runs/pi-live-divergence-arbiter-smoke` validates with `tests/behavioral-smoke.sh`.
- [x] Handoff documents the new run and any learnings/gaps.

## Validation

2026-05-31:

- `pi -e extensions/pi-foundry-team/index.ts --tools foundry_team ...` — live child dispatch succeeded for `foundry:review:divergence-evaluator` and `foundry:review:arbiter-agent`.
- Divergence evaluator returned `findings[0].outcome = VALUABLE` for the slugify Unicode policy gap.
- Arbiter returned `findings[0].outcome = TEST_WRONG` for exactly one emoji-preservation test outside the ASCII slug contract.
- `tests/validate-barrier-envelopes.sh runs/pi-live-divergence-arbiter-smoke/dispatch` — passed.
- `tests/behavioral-smoke.sh runs/pi-live-divergence-arbiter-smoke` — passed.
