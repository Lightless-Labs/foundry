---
title: Run a from-scratch Pi adversarial feature after the Sudoku replay
origin: 2026-05-22 follow-up after smoke-scoped Pi adversarial run
priority: medium
status: pending
updated: 2026-05-22
---

# From-Scratch Pi Adversarial Feature Run

The current Pi behavioral proof is a **smoke-scoped replay** over the Sudoku worked example:

- `/skill:foundry-adversarial` loads under Pi.
- `foundry_team` dispatches red-team, green-team, and barrier-integrity-auditor child Pi processes from PromptEnvelope artifacts.
- Existing worked-example red tests and green implementation are reused.
- `runs/pi-autonomous-sudoku-smoke/` validates with `tests/behavioral-smoke.sh` and `tests/validate-barrier-envelopes.sh`.

That proves the public Pi package/skill/dispatch/artifact lane. It does **not** yet prove that Pi can complete a fresh non-example feature where red and green generate new artifacts from scratch.

## What to do

Run a new small feature through the full Pi flow where:

1. Research/spec/NLSpec are either newly generated or intentionally minimal but reviewed.
2. Red team writes fresh tests from the NLSpec Done criteria.
3. Green team writes fresh implementation from NLSpec How plus PASS/FAIL labels only.
4. The orchestrator routes all subagent/reviewer work through `foundry_team` PromptEnvelope artifacts.
5. Final artifacts validate with:

```bash
tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch
tests/behavioral-smoke.sh runs/<run_id>
```

## Candidate feature

Pick something smaller than Sudoku but non-trivial enough to exercise red/green separation, for example:

- Rust CLI `roman-numeral` converter/parser with clear golden vectors.
- Rust CLI `semver-range-check` with explicit version comparison cases.
- Rust library `slugify` with Unicode/ASCII policy explicitly pinned.

Prefer golden vectors to avoid convention mismatch.

## What to learn

- Whether Pi follows the full canonical `foundry-adversarial` skill without the smoke-scoped shortcuts.
- Whether `foundry_team` tool outputs provide enough model-lane/detail data for real runs.
- Whether the heavy adversarial skill needs modularization before from-scratch use.
- Whether red/green workspace filesystem isolation needs additional hardening.

## Acceptance criteria

- [ ] New `runs/<run_id>/` artifact directory validates with both replay validators.
- [ ] Red and green artifacts are freshly generated, not copied from an existing worked example.
- [ ] Green PromptEnvelopes expose only NLSpec How + PASS/FAIL labels.
- [ ] Any obedience or workflow gaps are fed back into `todos/modularize-heaviest-skills.md` or a dedicated plan.
