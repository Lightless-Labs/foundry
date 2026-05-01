---
date: 2026-04-07
topic: spec-divergence-feedback-loop
source_spec: docs/specs/2026-04-06-spec-divergence-feedback-loop-spec.md
status: reviewed
---

# Spec Divergence Feedback Loop NLSpec

The spec divergence feedback loop is a mechanism inside the `foundry:adversarial` skill that turns team divergences into spec improvement signals. When the red team writes a test referencing behavior absent from the NLSpec, or when the green team fails the same test three times in a row, the orchestrator routes to an ephemeral evaluator that judges whether the divergence represents a genuine spec gap. If it does, the NLSpec agent re-derives the spec and the pipeline restarts from Phase 1 with a change summary. This document is intended for coding agents implementing or extending the adversarial skill orchestrator.

## Table of Contents

- [1. Why](#1-why)
  - [1.1 Problem Statement](#11-problem-statement)
  - [1.2 Design Principles](#12-design-principles)
  - [1.3 Layering and Scope](#13-layering-and-scope)
- [2. What](#2-what)
  - [2.1 Data Model](#21-data-model)
  - [2.2 Architecture](#22-architecture)
  - [2.3 Vocabulary](#23-vocabulary)
- [3. How](#3-how)
  - [3.1 Phase 1b Divergence Check](#31-phase-1b-divergence-check)
  - [3.2 Phase 2b Divergence Check](#32-phase-2b-divergence-check)
  - [3.3 Spec Update and Pipeline Restart](#33-spec-update-and-pipeline-restart)
  - [3.4 Phase 1 Restart](#34-phase-1-restart)
- [4. Out of Scope](#4-out-of-scope)
- [5. Design Decision Rationale](#5-design-decision-rationale)
- [6. Definition of Done](#6-definition-of-done)

---

## 1. Why

### 1.1 Problem Statement

**Status quo.** The `foundry:adversarial` skill routes divergences — red tests referencing unspecified behavior, or green failing repeatedly — as team errors. The orchestrator corrects them in place and the pipeline continues.

**Pain.** Divergences are frequently spec gaps in disguise. A red test that references missing behavior may be surfacing a real requirement. A green implementation that can't pass a test after three iterations may be blocked by an ambiguous or incomplete spec, not a coding error. The current routing silently discards these signals, leaving the spec permanently incomplete.

**Solution.** Intercept every divergence before team correction. Route to an ephemeral evaluator that judges the divergence against the raw NLSpec. If the evaluator finds a genuine gap, the NLSpec agent re-derives the spec with enriched input and the pipeline restarts. No signal is discarded.

### 1.2 Design Principles

**Zero Framework Cognition (ZFC).** The orchestrator carries no intelligence about divergence categories. It routes raw artifacts to the evaluator and acts on the evaluator's structured output. All reasoning happens inside the model, not in the framework. Consequence: the orchestrator is stable even as divergence patterns evolve.

**Ephemeral evaluator, one divergence at a time.** The evaluator has no persistent context across invocations. Each invocation is scoped to exactly one divergence. Consequence: evaluator drift and cross-contamination between divergences are impossible by construction.

**NLSpec agent is the sole author.** The orchestrator MUST NOT amend the NLSpec directly — not even a single line. The NLSpec agent re-runs with enriched input and produces a complete replacement. Consequence: authorship is unambiguous and every NLSpec delta is attributable to the agent, not the orchestrator.

**Two-commit git discipline.** Before any NLSpec overwrite, the current version is committed. After the new NLSpec is produced, it is committed again. Consequence: the before-state is always recoverable and the delta is auditable.

**Red team owns test continuity.** At Phase 1 restart, red receives their existing tests plus the new NLSpec plus a plain-language change summary. Red is not asked to start from scratch. Consequence: prior test work is preserved and red knows exactly which behaviors changed.

**Sequential divergence processing.** The orchestrator MUST process divergences sequentially. Only one evaluator invocation may be in flight at a time per pipeline run. Consequence: revision state remains consistent and no two evaluator judgments can race to mutate `PipelineRevisionState`.

### 1.3 Layering and Scope

This spec covers the divergence detection and feedback path within the `foundry:adversarial` skill. It presupposes the adversarial phases (Phase 1, Phase 1b, Phase 2, Phase 2b) defined in the adversarial playbook.

This spec does NOT cover: the internal logic of Phase 1 or Phase 2 beyond the divergence trigger points; how the red-team-test-reviewer detects out-of-spec tests (that is its own spec); how the NLSpec agent re-derives (that is the `foundry:nlspec` skill spec); or any behavior before Phase 1b.

---

## 2. What

### 2.1 Data Model

```
-- Three-value outcome produced by the divergence evaluator
ENUM DivergenceOutcome:
    VALUABLE        -- genuine spec gap; NLSpec re-derivation warranted
    NOT_VALUABLE    -- team error; no spec change needed
    INCONCLUSIVE    -- evaluator cannot determine; escalate to user

-- The structured output produced by the divergence evaluator
RECORD DivergenceJudgment:
    outcome         : DivergenceOutcome   -- replaces the old Bool valuable field
    rationale       : String              -- mandatory; must explain reasoning
    gap_description : String | None       -- see invariant below

-- Invariant: gap_description MUST be present when outcome == VALUABLE,
-- and MUST be None otherwise.
-- ASSERT (outcome == VALUABLE) == (gap_description IS NOT None)

-- Input package assembled by the orchestrator for the evaluator
RECORD EvaluatorInput:
    nlspec_content       : String   -- full NLSpec document, raw
    diverging_artifact   : String   -- raw test scenario (Phase 1b) or failing impl snippet (Phase 2b)
    divergence_phase     : DivergencePhase

ENUM DivergencePhase:
    PHASE_1B   -- red test references behavior absent from NLSpec DoD
    PHASE_2B   -- same test fails N consecutive green iterations

-- State the orchestrator tracks across the pipeline run
RECORD PipelineRevisionState:
    run_id           : String
    revision_count   : Int      -- cumulative spec revisions this run; starts at 0
    revision_cap     : Int      -- default 10; pause and present to user when reached
    revision_history : List[RevisionRecord]

RECORD RevisionRecord:
    revision_number  : Int
    divergence_phase : DivergencePhase
    gap_description  : String
    commit_before    : String   -- git SHA of NLSpec before overwrite
    commit_after     : String   -- git SHA of new NLSpec

-- Input passed to the NLSpec agent on re-run
RECORD NLSpecRerunInput:
    original_spec_path : String   -- path to source spec document
    existing_nlspec_path : String -- path to current NLSpec (pre-overwrite)
    evaluator_feedback : String   -- gap_description from DivergenceJudgment

-- The orchestrator (as an LLM) generates the change summary by reading the
-- before and after NLSpec files and producing a structured description.
-- Fields:
--   sections_added       : List[String]  -- names/headings of sections added
--   sections_modified    : List[String]  -- names/headings of sections changed
--   requirements_delta   : List[String]  -- individual requirements added or changed
RECORD ChangeSummary:
    sections_added      : List[String]
    sections_modified   : List[String]
    requirements_delta  : List[String]

-- Red team restart package
RECORD Phase1RestartPackage:
    existing_tests  : List[String]  -- red team's previously submitted tests, unmodified
    new_nlspec_path : String        -- path to newly produced NLSpec
    change_summary  : ChangeSummary -- structured diff: what was added/changed vs previous NLSpec

-- Failure counter per test, used by orchestrator in Phase 2b.
-- This record is pipeline-run-scoped: all counters are re-initialized
-- (reset to 0) whenever Phase 1 restarts.
-- "Test changes" means: the content hash of the test file(s) associated
-- with the failing test changes between green iterations.
RECORD TestFailureTracker:
    test_id            : String
    consecutive_fails  : Int    -- resets to 0 when test passes, test content changes, or Phase 2b triggers
    threshold          : Int    -- default 3 (N=3)
    test_content_hash  : String -- hash of test file content; used to detect test changes between iterations
```

### 2.2 Architecture

```
foundry:adversarial orchestrator
    |
    |-- [Phase 1b] red-team-test-reviewer flags out-of-spec test
    |       |
    |       +--> divergence_check_phase1b(test_scenario, nlspec)
    |               |
    |               +--> spawn EphemeralDivergenceEvaluator
    |               |       receives: EvaluatorInput(PHASE_1B)
    |               |       produces: DivergenceJudgment
    |               |
    |               +--> route(judgment) --VALUABLE-----> spec_update_and_restart()
    |               |                   --NOT_VALUABLE-> send red team back
    |               |                   --INCONCLUSIVE-> present to user
    |
    |-- [Phase 2b] TestFailureTracker reaches threshold on same test
    |       |
    |       +--> divergence_check_phase2b(test_id, impl_snippet, nlspec)
    |               |
    |               +--> spawn EphemeralDivergenceEvaluator
    |               |       receives: EvaluatorInput(PHASE_2B)
    |               |       produces: DivergenceJudgment
    |               |
    |               +--> route(judgment) --same as above--
    |
    +--> spec_update_and_restart()
            |
            +--> git commit (before overwrite)
            +--> run NLSpec agent with NLSpecRerunInput
            +--> git commit (after new NLSpec)
            +--> generate ChangeSummary (orchestrator as LLM reads before/after NLSpec files)
            +--> increment revision_count; check against revision_cap
            +--> restart Phase 1 with Phase1RestartPackage
```

Component boundaries:
- **Orchestrator** — detects, routes, tracks revision state, generates change summary, manages git commits. MUST NOT produce NLSpec content.
- **EphemeralDivergenceEvaluator** — stateless agent; spawned per divergence; receives `EvaluatorInput`; produces `DivergenceJudgment`; terminated after each invocation.
- **NLSpec agent** — sole NLSpec author; receives `NLSpecRerunInput`; writes to the existing NLSpec path.
- **Red team** — receives `Phase1RestartPackage` at Phase 1 restart; produces revised test suite.

### 2.3 Vocabulary

| Term | Definition |
|------|-----------|
| **Divergence** | A signal where a team artifact (test or implementation) references behavior that the NLSpec does not cover, or cannot be satisfied by any conforming implementation |
| **Phase 1b divergence** | A divergence detected when the red-team-test-reviewer flags a test referencing behavior absent from the NLSpec DoD |
| **Phase 2b divergence** | A divergence detected when the same test fails on N=3 consecutive green iterations without the test changing |
| **Evaluator** | The ephemeral divergence evaluator agent; scoped to one divergence; produces `DivergenceJudgment` |
| **Valuable** | A divergence judgment where `outcome == VALUABLE`; indicates a genuine spec gap warranting NLSpec re-derivation |
| **Revision** | One complete cycle of: evaluator judges VALUABLE → NLSpec agent re-runs → pipeline restarts |
| **Revision cap** | The maximum number of revisions per pipeline run (default 10) before the orchestrator pauses and presents to the user |
| **Change summary** | A `ChangeSummary` record produced by the orchestrator (as an LLM) after NLSpec agent completes; describes sections added, sections modified, and requirements added or changed |
| **ZFC** | Zero Framework Cognition — all reasoning delegated to the model; the framework carries no embedded heuristics |

---

## 3. How

### 3.1 Phase 1b Divergence Check

Triggered when the red-team-test-reviewer flags a test referencing behavior not present in the NLSpec DoD.

```
FUNCTION divergence_check_phase1b(
    test_scenario  : String,        -- the flagged test, raw
    nlspec_path    : String,        -- path to current NLSpec
    red_test_paths : List[String]   -- paths to all current red team test files
) -> DivergenceJudgment | UserEscalation:

    -- Step 1: Assemble raw evaluator input (no orchestrator summarization)
    nlspec_content = read_file(nlspec_path)
    input = EvaluatorInput(
        nlspec_content     = nlspec_content,
        diverging_artifact = test_scenario,
        divergence_phase   = PHASE_1B
    )

    -- Step 2: Spawn ephemeral evaluator
    judgment = spawn_ephemeral_evaluator(input)
    -- evaluator is terminated after this call regardless of outcome

    -- Step 3: Route on judgment
    IF judgment.outcome == INCONCLUSIVE:
        RETURN UserEscalation(artifact=input, reason="evaluator inconclusive")

    IF judgment.outcome == NOT_VALUABLE:
        send_team_back(team=RED, feedback=judgment.rationale)
        RETURN judgment

    IF judgment.outcome == VALUABLE:
        -- gap_description MUST be present (enforced by invariant)
        ASSERT judgment.gap_description IS NOT None
        RETURN judgment
        -- caller invokes spec_update_and_restart(judgment, phase=PHASE_1B, red_test_paths=red_test_paths)
```

Behavior table:

| Input condition | Output |
|----------------|--------|
| Test references behavior present in NLSpec DoD | NOT triggered (reviewer does not flag it) |
| Test references behavior absent from NLSpec DoD; evaluator: NOT_VALUABLE | Red team sent back with `judgment.rationale` |
| Test references behavior absent from NLSpec DoD; evaluator: VALUABLE | `DivergenceJudgment` with `gap_description` returned to orchestrator; `spec_update_and_restart` invoked |
| Evaluator INCONCLUSIVE | `UserEscalation` raised; pipeline pauses |

### 3.2 Phase 2b Divergence Check

Triggered when `TestFailureTracker.consecutive_fails` reaches `threshold` (default 3) for the same test without the test changing.

```
FUNCTION divergence_check_phase2b(
    test_id        : String,        -- identifier of the repeatedly-failing test
    impl_snippet   : String,        -- most recent implementation written by green for this test, raw
    nlspec_path    : String,
    red_test_paths : List[String]   -- paths to all current red team test files
) -> DivergenceJudgment | UserEscalation:

    -- Step 1: Confirm threshold met (guard against spurious calls)
    tracker = get_tracker(test_id)
    ASSERT tracker.consecutive_fails >= tracker.threshold

    -- Step 2: Assemble raw evaluator input
    nlspec_content = read_file(nlspec_path)
    input = EvaluatorInput(
        nlspec_content     = nlspec_content,
        diverging_artifact = impl_snippet,
        divergence_phase   = PHASE_2B
    )

    -- Step 3: Spawn ephemeral evaluator
    judgment = spawn_ephemeral_evaluator(input)

    -- Step 4: Route on judgment (same routing as Phase 1b)
    IF judgment.outcome == INCONCLUSIVE:
        RETURN UserEscalation(artifact=input, reason="evaluator inconclusive")

    IF judgment.outcome == NOT_VALUABLE:
        send_team_back(team=GREEN, feedback=judgment.rationale)
        reset_tracker(test_id)   -- green gets a fresh attempt
        RETURN judgment

    IF judgment.outcome == VALUABLE:
        ASSERT judgment.gap_description IS NOT None
        RETURN judgment
        -- caller invokes spec_update_and_restart(judgment, phase=PHASE_2B, red_test_paths=red_test_paths)
```

Behavior table:

| Input condition | Output |
|----------------|--------|
| `consecutive_fails < threshold` | Function not called; green iteration continues normally |
| `consecutive_fails == threshold`; evaluator: NOT_VALUABLE | Green sent back with `judgment.rationale`; failure counter reset |
| `consecutive_fails == threshold`; evaluator: VALUABLE | `DivergenceJudgment` returned; `spec_update_and_restart` invoked |
| Evaluator INCONCLUSIVE | `UserEscalation` raised; pipeline pauses |
| Test changes between iterations | `consecutive_fails` reset to 0; Phase 2b not triggered |

Failure counter rules:
- Counter increments when: same `test_id` fails on the same iteration
- Counter resets to 0 when: test passes, test content hash changes (i.e., `test_content_hash` differs from the previously recorded hash), or Phase 2b is triggered (to prevent double-triggering on the same convergence)

### 3.3 Spec Update and Pipeline Restart

Triggered when either divergence check returns `outcome == VALUABLE`.

**Precondition:** No other evaluator invocation is in flight for this pipeline run (sequential processing guarantee — see §1.2 Design Principles).

```
FUNCTION spec_update_and_restart(
    judgment              : DivergenceJudgment,
    phase                 : DivergencePhase,       -- PHASE_1B or PHASE_2B
    original_spec_path    : String,
    existing_nlspec_path  : String,
    revision_state        : PipelineRevisionState,
    red_test_paths        : List[String]           -- paths to current red team test files
) -> Phase1RestartPackage | UserEscalation:

    -- Precondition: judgment.outcome MUST be VALUABLE
    ASSERT judgment.outcome == VALUABLE
    ASSERT judgment.gap_description IS NOT None

    -- Step 1: Check revision cap before proceeding
    IF revision_state.revision_count >= revision_state.revision_cap:
        -- Orchestrator finishes any in-progress step, then halts further
        -- evaluator spawning and NLSpec agent invocations until the user
        -- responds to this UserEscalation.
        RETURN UserEscalation(
            reason = "revision cap reached",
            history = revision_state.revision_history
        )

    -- Step 2: Commit current NLSpec before overwrite (attributed to nlspec agent)
    commit_before_sha = git_commit(
        path    = existing_nlspec_path,
        message = "nlspec: preserve pre-revision NLSpec before divergence update",
        author  = "nlspec-agent"
    )

    -- Step 3: Re-run NLSpec agent with enriched input
    rerun_input = NLSpecRerunInput(
        original_spec_path   = original_spec_path,
        existing_nlspec_path = existing_nlspec_path,
        evaluator_feedback   = judgment.gap_description
    )
    nlspec_agent_result = run_nlspec_agent(rerun_input)

    IF nlspec_agent_result.failed:
        RETURN UserEscalation(
            reason = "nlspec agent failed to incorporate feedback",
            gap    = judgment.gap_description
        )

    -- Step 4: Commit new NLSpec (attributed to nlspec agent)
    commit_after_sha = git_commit(
        path    = existing_nlspec_path,
        message = "nlspec: re-derive after divergence feedback",
        author  = "nlspec-agent"
    )

    -- Step 5: Generate change summary
    -- The orchestrator (as an LLM) reads the before and after NLSpec files
    -- and produces a ChangeSummary describing:
    --   (a) sections added (sections_added)
    --   (b) sections modified (sections_modified)
    --   (c) requirements added or changed (requirements_delta)
    -- The orchestrator MUST NOT amend the NLSpec while generating this summary.
    before_nlspec_content = git_show(commit_before_sha, existing_nlspec_path)
    after_nlspec_content  = read_file(existing_nlspec_path)
    change_summary = orchestrator_generate_change_summary(
        before = before_nlspec_content,
        after  = after_nlspec_content
    )
    -- change_summary : ChangeSummary

    -- Step 6: Record revision
    record = RevisionRecord(
        revision_number  = revision_state.revision_count + 1,
        divergence_phase = phase,
        gap_description  = judgment.gap_description,
        commit_before    = commit_before_sha,
        commit_after     = commit_after_sha
    )
    revision_state.revision_count += 1
    revision_state.revision_history.append(record)

    -- Step 7: Collect current red team tests from provided paths
    existing_tests = read_files(red_test_paths)

    -- Step 8: Return restart package (Phase 1 restart is invoked by caller)
    RETURN Phase1RestartPackage(
        existing_tests  = existing_tests,
        new_nlspec_path = existing_nlspec_path,
        change_summary  = change_summary
    )
```

Behavior table:

| Condition | Output |
|-----------|--------|
| `revision_count < revision_cap` and NLSpec agent succeeds | Two git commits produced; `Phase1RestartPackage` returned |
| `revision_count >= revision_cap` | `UserEscalation` with full revision history; orchestrator halts further evaluator spawning and NLSpec invocations until user responds |
| NLSpec agent fails to incorporate feedback | `UserEscalation` with gap description; NLSpec unchanged; no commits |
| Orchestrator attempts direct NLSpec edit | PROHIBITED — orchestrator MUST only call `run_nlspec_agent` |

### 3.4 Phase 1 Restart

Triggered when `spec_update_and_restart` returns a `Phase1RestartPackage`.

Note: `TestFailureTracker` is pipeline-run-scoped. When Phase 1 restarts, ALL `TestFailureTracker` instances for this run are re-initialized with `consecutive_fails = 0`. This prevents stale failure counts from triggering Phase 2b immediately after a spec revision.

```
FUNCTION phase1_restart(
    package        : Phase1RestartPackage,
    red_test_paths : List[String]   -- paths to current red team test files (used to reinitialize trackers)
) -> UpdatedTestSuite:

    -- Step 0: Re-initialize all TestFailureTracker instances for this run
    -- (counters reset to 0; test_content_hash updated to current values)
    reinitialize_all_trackers(red_test_paths)

    -- Step 1: Deliver restart package to red team
    -- Red team receives: existing tests (unmodified), new NLSpec path, change summary
    red_team_context = RedTeamRestartContext(
        existing_tests  = package.existing_tests,
        new_nlspec_path = package.new_nlspec_path,
        change_summary  = package.change_summary
    )

    -- Step 2: Red team reviews existing tests against new NLSpec + change summary
    -- Red team MUST revise or extend tests as needed
    -- Red team MUST NOT discard previously-passing tests without flagging
    updated_tests = red_team_review_and_revise(red_team_context)

    -- Step 3: Guard: flag if previously-passing tests were removed
    passing_before = get_previously_passing_tests(package.existing_tests)
    removed = passing_before - updated_tests
    IF removed IS NOT EMPTY:
        -- The orchestrator (as an LLM) reviews each removed test against the
        -- new NLSpec to determine if removal is justified:
        --   JUSTIFIED: the test covered behavior that was removed from scope
        --     in the new NLSpec → removal is accepted; pipeline continues.
        --   NOT JUSTIFIED: the test covered behavior still present in the new
        --     NLSpec → red team is sent back with the specific improperly
        --     removed test and MUST restore or replace it.
        orchestrator_review_removed_tests(
            removed_tests   = removed,
            new_nlspec_path = package.new_nlspec_path
        )

    -- Step 4: Run Phase 1b review against new NLSpec
    -- (Phase 1b divergence check runs on the updated test suite as normal)
    RETURN updated_tests
```

Behavior table:

| Condition | Output |
|-----------|--------|
| Red team revises tests; no previously-passing tests removed | `UpdatedTestSuite`; Phase 1b runs normally |
| Red team removes a test covering behavior removed from new NLSpec scope | Orchestrator reviews and accepts removal; pipeline continues |
| Red team removes a test covering behavior still in new NLSpec scope | Orchestrator sends red team back with the specific improperly removed test |
| Red team adds new tests that reference new NLSpec behavior | Normal; Phase 1b review accepts them |
| Red team adds new tests that still reference behavior absent from new NLSpec | Phase 1b divergence check triggers again on those tests |

---

## 4. Out of Scope

- **Phase 0 detection (pre-red).** Divergence detection before the red team submits any tests. The feedback loop is anchored to team artifacts; there is no artifact to evaluate before red submits. Extension point: a pre-submission spec completeness check could be added as a separate Phase 0b gate.

- **Multi-divergence batching.** Evaluating multiple divergences from the same phase in a single evaluator call. Each divergence is evaluated independently. Extension point: a batching coordinator could be layered on top of the evaluator, aggregating judgments before triggering a single NLSpec re-run.

- **Evaluator appeal or override mechanism.** No mechanism for teams to challenge an evaluator judgment. Extension point: a `UserEscalation` path exists for inconclusive judgments; appeal could be routed through the same path.

- **Prompt injection hardening beyond ephemeral scoping.** Teams could embed adversarial instructions in test variable names or comments. The ephemeral evaluator provides some isolation, but no content sanitization is applied. Extension point: a content sanitizer could pre-process `diverging_artifact` before evaluator invocation.

- **Pattern-based Phase 2 trigger.** The N=3 fixed threshold is intentionally simple. Adaptive thresholds and failure-pattern recognition are deferred. Extension point: `TestFailureTracker.threshold` is a field; a future strategy module can set it dynamically. See `todos/phase2-trigger-strategy.md`.

- **Convention mismatch escalation path.** When both teams are internally correct but mutually incompatible (e.g., different coordinate conventions), the fix requires golden test vectors added to the spec, not just NLSpec re-derivation. This is a distinct resolution path. Extension point: the evaluator's `gap_description` could include a `resolution_hint` field indicating "golden vectors needed".

---

## 5. Design Decision Rationale

**Why does the evaluator receive raw artifacts instead of orchestrator-written summaries?** ZFC. If the orchestrator summarizes the diverging artifact, it embeds heuristics about what is relevant. Providing the raw test scenario or implementation snippet lets the model reason from first principles. The orchestrator would have to encode every possible divergence pattern to write a reliable summary — that is precisely the intelligence that belongs in the model, not the framework. Rejected: orchestrator-mediated description.

**Why is there no pre-built divergence taxonomy (spec gap / convention mismatch / team error)?** ZFC again. A three-branch decision tree in the orchestrator is brittle — it hardens today's understanding of divergence categories into the framework. The evaluator will surface convention mismatches, golden vector needs, and other patterns without being told to look for them. If a new category emerges, no framework change is needed. Rejected: three-branch taxonomy in orchestrator.

**Why Phase 1 restart (full) instead of Phase 1b restart (review only)?** Red team needs to review their existing tests against the new spec before the Phase 1b reviewer sees them. If they restart at Phase 1b without this review, the same divergence will likely re-trigger immediately — the existing tests still reference the old behavior. The change summary gives red exactly the information they need to revise efficiently. Rejected: Phase 1b direct restart.

**Why does red team receive existing tests at restart (not a clean slate)?** Prior test work is valid. Most existing tests will still be correct against the new NLSpec. Throwing them away wastes effort and forces red to re-discover behavior that was already well-specified. The change summary scopes the revision to exactly what changed. Rejected: clean slate.

**Why N=3 for Phase 2b trigger?** Simplicity and predictability. A fixed threshold is easy to reason about and audit. The failure counter logic is transparent. Pattern-based triggering is a future optimization, not a day-one requirement. The threshold is a field, so it can be changed without redesign. Rejected: pattern-based triggering (deferred to `todos/phase2-trigger-strategy.md`).

**Why two git commits (before overwrite + after new NLSpec) instead of one?** The before-commit preserves the pre-overwrite state. Without it, the delta between the old and new NLSpec cannot be reconstructed after the fact. The two-commit pattern also makes the NLSpec agent's authorship unambiguous: both commits are attributed to the agent, not the orchestrator. The "third-thoughts incident" (commit dbf64c8) — where an orchestrator directly rewrote step definitions in a single commit — destroyed the before-state and voided the adversarial guarantee. Rejected: single commit after overwrite.

**Why is the revision cap 10?** Ten revisions in a single pipeline run is a strong signal that the source spec is fundamentally incomplete or that the evaluator is systematically over-triggering. Presenting the revision history to the user at that point surfaces the pattern for human judgment. The cap is configurable via `PipelineRevisionState.revision_cap`. Rejected: no cap (unbounded revision loops).

---

## 6. Definition of Done

### 6.1 Data Model

- [ ] `DivergenceOutcome` ENUM is defined with exactly three values: `VALUABLE`, `NOT_VALUABLE`, `INCONCLUSIVE`
- [ ] `DivergenceJudgment` RECORD is defined with `outcome: DivergenceOutcome`, `rationale: String`, and `gap_description: String | None`
- [ ] Invariant enforced: `(outcome == VALUABLE) == (gap_description IS NOT None)`
- [ ] `EvaluatorInput` carries `nlspec_content`, `diverging_artifact`, and `divergence_phase` — no orchestrator-written summaries
- [ ] `DivergencePhase` ENUM has exactly two values: `PHASE_1B` and `PHASE_2B`
- [ ] `PipelineRevisionState` tracks `revision_count`, `revision_cap` (default 10), and `revision_history`
- [ ] `RevisionRecord` captures `commit_before` and `commit_after` SHAs for every revision
- [ ] `NLSpecRerunInput` carries `original_spec_path`, `existing_nlspec_path`, and `evaluator_feedback`
- [ ] `ChangeSummary` RECORD is defined with `sections_added`, `sections_modified`, and `requirements_delta` (all `List[String]`)
- [ ] `Phase1RestartPackage` carries `existing_tests` (unmodified), `new_nlspec_path`, and `change_summary: ChangeSummary`
- [ ] `TestFailureTracker` tracks `consecutive_fails` per `test_id` with default `threshold=3` and `test_content_hash: String`
- [ ] `TestFailureTracker` is documented as pipeline-run-scoped; all counters re-initialized on Phase 1 restart

### 6.2 Architecture

- [ ] Orchestrator component exists with no NLSpec authoring capability
- [ ] `EphemeralDivergenceEvaluator` is spawned per divergence and terminated after each invocation
- [ ] NLSpec agent is the only component that writes to the NLSpec file
- [ ] Both git commits are attributed to the NLSpec agent, not the orchestrator
- [ ] Component boundaries (orchestrator / evaluator / NLSpec agent / red team) are enforced at interface level
- [ ] Only one evaluator invocation is in flight at a time per pipeline run (sequential processing)

### 6.3 Phase 1b Divergence Check

- [ ] Trigger fires when red-team-test-reviewer flags a test referencing behavior absent from NLSpec DoD
- [ ] Function signature includes `red_test_paths: List[String]` parameter
- [ ] `EvaluatorInput` contains raw test scenario (not a summary)
- [ ] `EvaluatorInput` contains full NLSpec content (not a summary)
- [ ] `divergence_phase` is set to `PHASE_1B`
- [ ] Evaluator is spawned ephemerally; no persistent context from prior invocations
- [ ] `outcome == NOT_VALUABLE` → red team sent back with `rationale`; pipeline continues
- [ ] `outcome == VALUABLE` → `spec_update_and_restart` invoked with `gap_description`, `phase=PHASE_1B`, and `red_test_paths`
- [ ] `outcome == INCONCLUSIVE` → `UserEscalation` raised; pipeline pauses for manual judgment
- [ ] Tests flagged as present in NLSpec DoD do NOT trigger Phase 1b divergence check

### 6.4 Phase 2b Divergence Check

- [ ] Trigger fires when `consecutive_fails == threshold` (default 3) for the same test without the test changing
- [ ] Function signature includes `red_test_paths: List[String]` parameter
- [ ] `EvaluatorInput` contains raw implementation snippet (most recently written by green for the failing test)
- [ ] `EvaluatorInput` contains full NLSpec content
- [ ] `divergence_phase` is set to `PHASE_2B`
- [ ] Failure counter resets to 0 when: test passes, `test_content_hash` changes between iterations, or Phase 2b is triggered
- [ ] `outcome == NOT_VALUABLE` → green sent back with `rationale`; failure counter reset
- [ ] `outcome == VALUABLE` → `spec_update_and_restart` invoked with `phase=PHASE_2B` and `red_test_paths`
- [ ] `outcome == INCONCLUSIVE` → `UserEscalation` raised
- [ ] Counter does not increment when the test content hash changes between iterations

### 6.5 Spec Update and Pipeline Restart

- [ ] Function signature includes `phase: DivergencePhase` and `red_test_paths: List[String]` parameters
- [ ] Precondition: `judgment.outcome == VALUABLE` asserted at entry
- [ ] Revision cap check occurs BEFORE any git commit or NLSpec overwrite
- [ ] When `revision_count >= revision_cap`, `UserEscalation` is raised with full `revision_history`; orchestrator halts evaluator spawning and NLSpec invocations until user responds
- [ ] Git commit of current NLSpec occurs BEFORE `run_nlspec_agent` is called
- [ ] NLSpec agent receives `NLSpecRerunInput` with `original_spec_path`, `existing_nlspec_path`, and `evaluator_feedback`
- [ ] NLSpec agent receives `gap_description` verbatim (not re-summarized by orchestrator)
- [ ] Git commit of new NLSpec occurs AFTER `run_nlspec_agent` completes successfully
- [ ] Both commits are attributed to the NLSpec agent
- [ ] Orchestrator MUST NOT write any content to the NLSpec file (verifiable: no orchestrator file writes to NLSpec path)
- [ ] `change_summary` is generated by orchestrator (as LLM) reading before/after NLSpec files and producing a `ChangeSummary` record
- [ ] `revision_count` is incremented by exactly 1 per completed revision
- [ ] `RevisionRecord.divergence_phase` is set from the `phase` parameter (not from a phantom `judgment_phase`)
- [ ] `RevisionRecord` is appended to `revision_history` for every completed revision
- [ ] If NLSpec agent fails, `UserEscalation` is raised; no commits are made; NLSpec is unchanged

### 6.6 Phase 1 Restart

- [ ] All `TestFailureTracker` instances are re-initialized (`consecutive_fails = 0`) at Phase 1 restart
- [ ] Red team receives `existing_tests` (unmodified from prior submission)
- [ ] Red team receives `new_nlspec_path` pointing to the newly produced NLSpec
- [ ] Red team receives `change_summary: ChangeSummary` describing what changed
- [ ] Red team is NOT asked to discard existing tests and start from scratch
- [ ] If red team removes a previously-passing test, orchestrator reviews it against new NLSpec
- [ ] Orchestrator accepts removal if behavior was removed from new NLSpec scope; rejects and sends red back otherwise
- [ ] After red team revision, Phase 1b review runs against the updated test suite
- [ ] New tests referencing behavior in the new NLSpec pass Phase 1b review
- [ ] New tests referencing behavior still absent from the new NLSpec trigger Phase 1b divergence check again

### 6.7 Integration Smoke Tests

#### Phase 1b Valuable Path

```
FUNCTION integration_smoke_test_phase1b_valuable():

    -- Setup: create adversarial pipeline run with a known spec gap
    pipeline = create_adversarial_pipeline(
        spec_path   = "fixtures/simple-calculator-spec.md",
        nlspec_path = "fixtures/simple-calculator.nlspec.md"
        -- NLSpec intentionally omits "divide by zero returns error" behavior
    )
    revision_state = PipelineRevisionState(revision_count=0, revision_cap=10)

    -- Red team submits test referencing missing divide-by-zero behavior
    red_test = "SCENARIO: divide(10, 0) returns DivisionByZeroError"
    red_test_paths = ["fixtures/tests/calculator_tests.md"]
    red_team_submit(pipeline, test=red_test)

    -- Phase 1b: reviewer flags the test as referencing behavior absent from NLSpec DoD
    flag = red_team_test_reviewer_result(pipeline)
    ASSERT flag.out_of_spec == true
    ASSERT flag.test_id == red_test

    -- Divergence check: evaluator judges VALUABLE
    judgment = divergence_check_phase1b(red_test, pipeline.nlspec_path, red_test_paths)
    ASSERT judgment.outcome == VALUABLE
    ASSERT judgment.gap_description IS NOT None
    ASSERT judgment.rationale IS NOT EMPTY

    -- Evaluator is terminated after call
    ASSERT evaluator_is_terminated(judgment.evaluator_instance)

    -- Spec update: two commits produced, NLSpec agent is author
    restart_package = spec_update_and_restart(
        judgment             = judgment,
        phase                = PHASE_1B,
        original_spec_path   = pipeline.spec_path,
        existing_nlspec_path = pipeline.nlspec_path,
        revision_state       = revision_state,
        red_test_paths       = red_test_paths
    )
    ASSERT revision_state.revision_count == 1
    commits = git_log(pipeline.nlspec_path, limit=2)
    ASSERT commits[0].author == "nlspec-agent"   -- after commit
    ASSERT commits[1].author == "nlspec-agent"   -- before commit
    ASSERT commits[0].sha != commits[1].sha

    -- New NLSpec contains divide-by-zero behavior in DoD
    new_nlspec = read_file(pipeline.nlspec_path)
    ASSERT new_nlspec CONTAINS "divide by zero" OR "DivisionByZeroError"

    -- Change summary is present and structured
    ASSERT restart_package.change_summary IS NOT None
    ASSERT (restart_package.change_summary.sections_added IS NOT EMPTY
            OR restart_package.change_summary.sections_modified IS NOT EMPTY
            OR restart_package.change_summary.requirements_delta IS NOT EMPTY)

    -- Phase 1 restart: red team receives existing tests + new NLSpec + change summary
    ASSERT restart_package.existing_tests CONTAINS red_test
    ASSERT restart_package.new_nlspec_path == pipeline.nlspec_path

    -- Phase 1b runs on updated red tests against new NLSpec
    phase1b_result = run_phase1b_review(restart_package)

    -- Strengthened assertion: verify the specific behavior is now covered
    ASSERT phase1b_result.divergences IS EMPTY
    ASSERT new_nlspec CONTAINS "divide by zero" OR "DivisionByZeroError"  -- (a) behavior present in new NLSpec
    ASSERT phase1b_result.all_tests_accepted == true                       -- (b) red's tests now pass Phase 1b
    ASSERT revision_state.revision_count == 1                              -- (c) exactly one revision occurred

    RETURN PASS
```

#### Phase 2b Valuable Path

```
FUNCTION integration_smoke_test_phase_2b():

    -- Setup: create adversarial pipeline run where green repeatedly fails the same test
    pipeline = create_adversarial_pipeline(
        spec_path   = "fixtures/simple-calculator-spec.md",
        nlspec_path = "fixtures/simple-calculator.nlspec.md"
        -- NLSpec is ambiguous about rounding behavior; green cannot resolve it
    )
    revision_state = PipelineRevisionState(revision_count=0, revision_cap=10)
    red_test_paths = ["fixtures/tests/calculator_tests.md"]

    failing_test_id = "test_rounding_behavior"
    tracker = TestFailureTracker(
        test_id           = failing_test_id,
        consecutive_fails = 0,
        threshold         = 3,
        test_content_hash = hash_file("fixtures/tests/calculator_tests.md")
    )

    -- Green fails the same test 3 consecutive times (test content unchanged)
    REPEAT 3 TIMES:
        impl_snippet = green_attempt(pipeline, failing_test_id)
        ASSERT impl_snippet IS NOT None
        tracker.consecutive_fails += 1

    -- Threshold reached: Phase 2b triggers
    ASSERT tracker.consecutive_fails == tracker.threshold
    judgment = divergence_check_phase2b(
        failing_test_id,
        impl_snippet,
        pipeline.nlspec_path,
        red_test_paths
    )
    ASSERT judgment.outcome == VALUABLE
    ASSERT judgment.gap_description IS NOT None

    -- Evaluator is terminated after call
    ASSERT evaluator_is_terminated(judgment.evaluator_instance)

    -- Spec update: NLSpec re-derived, two commits produced
    restart_package = spec_update_and_restart(
        judgment             = judgment,
        phase                = PHASE_2B,
        original_spec_path   = pipeline.spec_path,
        existing_nlspec_path = pipeline.nlspec_path,
        revision_state       = revision_state,
        red_test_paths       = red_test_paths
    )
    ASSERT revision_state.revision_count == 1
    commits = git_log(pipeline.nlspec_path, limit=2)
    ASSERT commits[0].author == "nlspec-agent"
    ASSERT commits[1].author == "nlspec-agent"

    -- New NLSpec clarifies the previously ambiguous behavior
    new_nlspec = read_file(pipeline.nlspec_path)
    ASSERT new_nlspec CONTAINS "rounding"   -- rounding behavior now specified

    -- Phase 1 restart: TestFailureTracker re-initialized
    updated_tests = phase1_restart(restart_package, red_test_paths)
    reinitialized_tracker = get_tracker(failing_test_id)
    ASSERT reinitialized_tracker.consecutive_fails == 0

    -- Phase 1b runs on updated test suite against new NLSpec; no divergence triggered
    phase1b_result = run_phase1b_review(restart_package)
    ASSERT phase1b_result.divergences IS EMPTY
    ASSERT revision_state.revision_count == 1   -- exactly one revision occurred

    RETURN PASS
```
