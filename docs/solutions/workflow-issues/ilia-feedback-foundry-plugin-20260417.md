---
title: "Ilia feedback on the foundry plugin — repo identity, barrier enforcement, behavioral validation, skill modularization, hygiene"
module: foundry
date: 2026-04-17
problem_type: upstream_feedback
component: plugin-identity-and-validation
severity: medium
status: proposed
source_repo: Lightless-Labs/ilia
provenance: |
  Plain-English review of the foundry plugin / skills / agents repo contributed
  from the ilia project on 2026-04-17. Not tied to any specific dispatch, run,
  or commit range — a qualitative assessment of the repo's theory of operation,
  structural strengths, and rough edges, with five recommended next moves for
  hardening the plugin layer.
tags:
  - foundry
  - feedback
  - plugin-identity
  - barrier-integrity
  - validation
  - skill-modularization
  - repo-hygiene
---

# Ilia Feedback on the Foundry Plugin

## Overall Assessment

Foundry is one of the more serious agent-workflow repos in the Lightless Labs orbit. It has an actual theory of operation, not just a loose bundle of prompts. The most distinctive strength is the explicit information barrier between red-team test authoring and green-team implementation. That makes the repo feel structurally motivated rather than cosmetically agentic.

## What Feels Strong

- **Clear invariant.** The red/green separation is the sharpest idea in the repo. It gives the system a real epistemic goal: reduce correlated outputs by separating what each side is allowed to see.
- **Coherent skill pipeline.** `foundry:research`, `foundry:brainstorm`, `foundry:nlspec`, `foundry:adversarial`, and `foundry:forge` form a comprehensible progression rather than an arbitrary menu of prompts.
- **Artifacts are first-class.** Research docs, specs, NLSpecs, worked examples, and learnings are treated as durable outputs. That is healthier than relying on chat context alone.
- **Validation discipline.** `tests/validate-agents.sh` gives the prompt/plugin layer a testable contract and raises confidence that the repo is maintained deliberately.
- **Worked examples teach the method.** Sudoku, Rubik's cube, and chess are not just demos; together they explain why convention mismatch and golden vectors matter.

## What Feels Weak or Risky

- **Repo identity is blurry.** The root `AGENTS.md` / `CLAUDE.md` still read like a Rust + Bazel product repo, but this checkout is actually a Claude plugin / skills / agents repo. The engine lives elsewhere. That mismatch is the first thing worth fixing.
- **Barrier enforcement is still partly prompt-discipline.** The repo is thoughtful about barrier integrity, but too much still depends on careful orchestration prose. The strongest future version should push more of the barrier into mechanical enforcement and replayable audits.
- **Behavioral hardening lags structural validation.** The validation script is valuable, but it mostly checks file structure, coverage cues, and conventions. The hard problems here are behavioral: whether the barrier actually holds, whether divergence restarts behave correctly, and whether different models stay within the intended lanes.
- **Docs are good but heavy.** The skills carry a lot of nuance, which is a strength, but also a reliability risk. Long skill docs are easier for models to partially obey. Smaller contracts and more executable checks would harden the system.
- **Repo hygiene looks rough in practice.** Example build artifacts and local debris should not dominate the working tree. A prompt/plugin repo should feel cleaner than the current local checkout.

## Recommended Next Moves

1. **Fix repo identity at the root.** Make it explicit that this repo is the public plugin/skills/agents layer and that the Rust engine lives in the private engine repo.
2. **Strengthen mechanical barrier enforcement.** Add more executable checks around prompt shaping, redaction, and prompt transcript auditing.
3. **Add behavioral smoke tests.** Validate end-to-end adversarial runs and barrier invariants, not just prompt-file structure.
4. **Improve hygiene around examples and generated artifacts.** Keep local churn from obscuring real plugin changes.
5. **Modularize the heaviest skills.** Preserve the method, but reduce the execution burden on the model by breaking giant instruction blocks into tighter contracts.

## Bottom Line

Foundry already has the hardest part: a non-trivial organizing idea. It does not feel gimmicky. It feels like a promising system that now needs product-level hardening: clearer repo identity, stronger mechanical guarantees, more behavioral validation, and cleaner operational surfaces.
