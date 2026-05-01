---
date: 2026-04-06
topic: spec-divergence-feedback-loop
status: active
research: docs/research/2026-04-05-spec-divergence-feedback-loop-research.md
---

# Specification: Spec Divergence Feedback Loop

## Problem Statement

The `foundry:adversarial` skill silently corrects team work that diverges from the NLSpec. When red's tests reference behavior not in the spec, or green can't pass a test after repeated attempts, the orchestrator treats these as team errors and routes accordingly — discarding the signal. But divergence is often a symptom of a spec gap, not a team mistake. The current skill loses this signal and perpetuates an incomplete spec.

The fix: treat every divergence as a spec dispute first. An ephemeral evaluator judges whether the divergence is valuable. If it is, the nlspec agent is re-run with enriched input — never patched in place — and the pipeline restarts from Phase 1.

## Actors and Boundaries

- **Orchestrator** — detects divergences, routes to evaluator, acts on judgment
- **Divergence evaluator** — ephemeral agent; sees NLSpec + diverging artifact; produces a judgment (valuable / not valuable) and rationale
- **NLSpec agent** — re-run with enriched input when divergence is valuable; sole author of NLSpec documents
- **Red team** — restarts from Phase 1 with existing tests + new NLSpec + change summary when spec is updated
- **Green team** — sent back to fix implementation when divergence is not valuable

## Requirements

- R1. The adversarial skill detects divergences at Phase 1b (red test references behavior not in NLSpec DoD) and Phase 2b (same test fails on N consecutive green iterations; N=3 default)
- R2. On divergence detection, the orchestrator spawns an ephemeral divergence evaluator
- R3. The evaluator receives: the full NLSpec + the specific diverging artifact (test scenario or failing implementation snippet) — raw, not summarized
- R4. The evaluator produces a structured judgment: valuable (yes/no), rationale, and if valuable, a description of the gap
- R5. The evaluator reasons from first principles — no pre-built divergence taxonomy in the orchestrator or evaluator prompt
- R6. If valuable: the nlspec agent is re-run with original spec + existing NLSpec + evaluator feedback. The pipeline restarts from Phase 1
- R7. At Phase 1 restart: red team receives their existing tests + the new NLSpec + a plain-language summary of what changed
- R8. If not valuable: the responsible team is sent back with targeted feedback
- R9. The NLSpec is never amended directly by the orchestrator. The nlspec agent is the sole author
- R10. The evaluator is ephemeral — no persistent context across invocations; scoped to one divergence at a time
- R11. The orchestrator tracks cumulative spec revisions per pipeline run. After 10 revisions, pause and present the revision history to the user for judgment before continuing
- R12. Before overwriting the NLSpec, the orchestrator commits the current version to git. After the nlspec agent produces the new NLSpec, it is committed again. Both commits are attributed to the nlspec agent, not the orchestrator

## Behaviors

### Behavior: Phase 1b Divergence Check

- **Trigger:** Red team submits tests; red-team-test-reviewer flags a test that references behavior not present in the NLSpec DoD
- **Input:** The flagged test scenario + the NLSpec
- **Process:** Orchestrator spawns divergence evaluator with the test + NLSpec. Evaluator judges: valuable or not
- **Output:** Judgment + rationale
- **Errors:** If evaluator is inconclusive, orchestrator presents to user for manual judgment

### Behavior: Phase 2b Divergence Check

- **Trigger:** The same test has failed on N=3 consecutive green iterations without the test changing
- **Input:** The failing test name + the implementation snippet most recently written by green + the NLSpec
- **Process:** Orchestrator spawns divergence evaluator. Evaluator judges: valuable or not
- **Output:** Judgment + rationale
- **Errors:** If evaluator is inconclusive, orchestrator presents to user for manual judgment

### Behavior: Spec Update and Pipeline Restart

- **Trigger:** Evaluator returns valuable=true
- **Input:** Original spec path + existing NLSpec path + evaluator feedback (gap description)
- **Process:** Orchestrator re-runs the nlspec agent with enriched input. nlspec agent produces a new NLSpec. Orchestrator generates a plain-language change summary (what was added/changed vs previous NLSpec)
- **Output:** New NLSpec at same path (previous version preserved in git); change summary
- **Errors:** If nlspec agent fails to incorporate feedback, present gap to user for manual resolution

### Behavior: Phase 1 Restart

- **Trigger:** New NLSpec produced after spec update
- **Input:** Red team's existing tests + new NLSpec + change summary
- **Process:** Red team reviews existing tests against the new NLSpec and the change summary. Red team revises or extends tests as needed. Phase 1b review runs again
- **Output:** Updated test suite against new NLSpec
- **Errors:** If red team removes tests that were previously passing, flag for orchestrator review

## Key Decisions

- **Decision:** Evaluator sees raw artifacts (test scenario or implementation snippet), not orchestrator-written summaries
  - **Rationale:** ZFC — all reasoning to the model. Orchestrator-written summaries are local intelligence in the framework. Raw artifacts let the model reason from first principles
  - **Rejected:** Orchestrator-mediated descriptions — would embed heuristics about what matters in the framework

- **Decision:** No pre-built divergence taxonomy (spec gap / convention mismatch / team error)
  - **Rationale:** ZFC — categories emerge from the evaluator's reasoning, not from a decision tree in the orchestrator. The model will surface convention mismatch, golden vector needs, etc. without being told to look for them
  - **Rejected:** Three-branch taxonomy — would make the orchestrator intelligent in a way that's brittle and hard to maintain

- **Decision:** Phase 1 restart (not Phase 1b) after spec update
  - **Rationale:** Red team needs to own their tests against the new spec. Sending them into 1b review without review of their existing tests first risks surfacing the same divergence again
  - **Rejected:** Phase 1b restart — red team wouldn't know what changed or why

- **Decision:** Red team receives existing tests + new NLSpec + change summary at restart
  - **Rationale:** Red team shouldn't rebuild from scratch — they have good tests. The change summary tells them exactly what to revisit
  - **Rejected:** Clean slate (no existing tests) — wastes prior work; red team loses context

- **Decision:** Phase 2 trigger = N=3 consecutive failures on same test
  - **Rationale:** Start dumb. Simple, predictable, easy to reason about
  - **Rejected:** Pattern-based triggering — deferred to `todos/phase2-trigger-strategy.md`

- **Decision:** NLSpec agent is sole author; orchestrator never amends in place; git captures before and after
  - **Rationale:** Direct orchestrator amendments break provenance (third-thoughts incident, commit dbf64c8). Two-commit pattern (before overwrite + after new NLSpec) makes the delta auditable and authorship unambiguous
  - **Rejected:** In-place amendment — proven to void the adversarial guarantee; single commit — loses the before state

## Scope Boundaries

- **In scope:** Phase 1b and Phase 2b detection; divergence evaluator agent; nlspec re-run with enriched input; Phase 1 restart with change summary
- **Out of scope:** Phase 0 detection (pre-red); multi-divergence batching; evaluator appeal/override mechanism; prompt injection hardening beyond ephemeral scoping
- **Future:** Pattern-based Phase 2 trigger; adaptive N threshold; evaluator confidence calibration tuning (`todos/phase2-trigger-strategy.md`)

## Success Criteria

- A test referencing behavior not in the NLSpec triggers evaluator invocation at Phase 1b
- A test failing N=3 consecutive green iterations triggers evaluator invocation at Phase 2b
- When evaluator returns valuable=true, a new NLSpec is produced without direct orchestrator amendment
- Red team restart at Phase 1 receives existing tests + new NLSpec + change summary
- The pipeline converges on a passing test suite reflecting the enriched spec
- No NLSpec is ever modified by the orchestrator directly (verifiable via git authorship)

## Open Questions

### Resolved

- Evaluator input format → raw artifacts (ZFC; option B)
- Divergence taxonomy → none; model reasons from first principles (ZFC)
- Pipeline restart point → Phase 1
- Phase 2 trigger → N=3 fixed threshold
- NLSpec authorship → nlspec agent only; never orchestrator

### Deferred

- Phase 2 trigger strategy re-assessment → `todos/phase2-trigger-strategy.md`
- Prompt injection hardening → when to address TBD
