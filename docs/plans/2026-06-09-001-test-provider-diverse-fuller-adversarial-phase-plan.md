---
title: Exercise a fuller provider-diverse adversarial phase
created: 2026-06-09
status: active
todo: todos/multi-provider-delegation.md
---

# Exercise a Fuller Provider-Diverse Adversarial Phase

## Goal

Move beyond provider-diverse plumbing and phase-artifact sketches by running a fuller PromptEnvelope-backed adversarial phase with distinct red and green provider/model lanes.

The target is a small, from-scratch Rust feature so the run is meaningful but bounded enough for live Pi runtime:

- Red lane writes executable tests from an NLSpec/Definition of Done without seeing implementation code.
- Green lane writes implementation from the NLSpec How section plus PASS/FAIL labels only, without seeing red test code/assertions/raw output/NLSpec Done.
- The orchestrator assembles and runs tests, preserving replayable run artifacts.
- Distinct provider/model lanes are recorded and mechanically validated.

## Proposed Live Lane

- Run directory: `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/`
- Red model: `minimax/MiniMax-M3`
- Green model: `kimi-coding/kimi-for-coding`
- Feature shape: small Rust library/CLI with deterministic golden examples and edge cases.
- Candidate feature: `slugify` or similar string transformer with explicit Unicode/ASCII/punctuation rules, because it is small, fast to test, and easy to specify with golden vectors.

## Scope

- Create a reviewed NLSpec fixture inside the run directory or a temporary project seed.
- Write PromptEnvelope artifacts for at least:
  - `phase1/red-team.json`
  - `phase2/green-team.json`
  - optional reviewer/auditor envelopes if runtime allows.
- Dispatch red and green via `foundry_team` using distinct provider/model lanes.
- Preserve generated red tests and green implementation under the run directory.
- Run the generated tests as orchestrator.
- Emit `behavioral-smoke.toon` with `requires_distinct_model_lanes: true`.
- Validate:
  - `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch`
  - `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke`

## Non-goals

- Do not modify the private Rust engine/state machine.
- Do not require a full Phase 3 reviewer fan-out if live runtime is tight; record skipped reviewers as a limitation and follow up separately.
- Do not let the orchestrator edit generated implementation code directly to force a pass. If green needs a fix, route it through a new green PromptEnvelope with PASS/FAIL labels only.
- Do not paste hidden red/green context into normal Pi messages; use serialized PromptEnvelope artifacts and `foundry_team` only.

## Acceptance

- [ ] A kept run directory exists with dispatch envelopes, generated red tests, generated green implementation, logs, and `behavioral-smoke.toon`.
- [ ] Red and green child dispatches report distinct actual model lanes.
- [ ] Red tests execute against green implementation with the accepted final pass count.
- [ ] Barrier validation passes for the run dispatch directory.
- [ ] Behavioral smoke validation passes and enforces distinct model lanes.
- [ ] `docs/HANDOFF.md` and the related todo are updated with the result and any learnings.

## Risk Controls

- Use a small feature and low test count to avoid another long-running Phase 3 timeout.
- Seed the NLSpec with concrete golden vectors to avoid red/green convention drift.
- Keep withheld samples high-entropy and never use allowed PASS/FAIL labels as poison samples.
- If a child model returns malformed artifacts, preserve the failed run notes and use the phase-artifact smoke as a fallback baseline.

## Validation Log

Pending.
