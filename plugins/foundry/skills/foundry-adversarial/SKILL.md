---
name: foundry:adversarial
description: "Run the adversarial red/green implementation process. A red team writes tests from the NLSpec's Definition of Done, a green team implements from the NLSpec's How section, and the orchestrator mediates with strict information barriers. Use when you have a reviewed NLSpec and want adversarial implementation. Triggers on 'adversarial', 'red green', 'forge it', 'implement adversarially'."
argument-hint: "[path to NLSpec document]"
---

# Foundry Adversarial

Run the adversarial red/green implementation process. You (the orchestrating agent) mediate between a red team (test writers) and a green team (implementers) with strict information barriers.

## Prerequisites

A reviewed NLSpec document (from `foundry:nlspec`). If not provided, search `docs/nlspecs/` for the most recent reviewed NLSpec.

## The Information Barrier

This is the core invariant. Violating it defeats the purpose.

| Entity | Sees | Never sees |
|--------|------|------------|
| Red team | NLSpec (full), spec | Implementation code |
| Red reviewer | NLSpec, test code | Implementation code |
| Green team | NLSpec How section only, test outcome labels (pass/fail) | Test code, assertions, error messages, NLSpec Done section |
| Green reviewer | NLSpec How section, implementation, test outcomes | Test code, NLSpec Done section |
| You (orchestrator) | Everything | — |

**You are the only entity that crosses the barrier. You enforce it by controlling what each subagent receives in its prompt.**

## Workflow

### Phase 0: Setup

1. Read the NLSpec. Extract:
   - The **How section** (implementation guidance for green team)
   - The **Definition of Done** (test criteria for red team)
   - The **Data Model** (shared types for both teams)
   - The **Integration Smoke Test** (end-to-end test for red team)

2. Detect the project language (inspect manifests) and determine Cucumber/Gherkin bindings.

3. **Search institutional knowledge** — spawn the learnings-researcher to check `docs/solutions/` for past solutions relevant to this feature. If relevant learnings exist, factor them into the red team's test design and the green team's implementation guidance.

```
Agent(
    subagent_type="foundry:research:learnings-researcher",
    prompt="Search docs/solutions/ for learnings relevant to: [feature topic from NLSpec].
    Return any past bugs, patterns, or best practices that should inform the implementation."
)
```

4. Create the workspace structure (or use git worktrees for real filesystem isolation):
   - `shared/` — NLSpec data model, types, interfaces
   - `red/` — test workspace (features/, step_definitions/)
   - `green/` — implementation workspace (src/)

### Phase 1: Red Team — Write Tests

Spawn the red team subagent:

```
Agent(
    name="red-team",
    mode="bypassPermissions",
    prompt="You are the RED TEAM. Your job is to write comprehensive tests
    that will validate an implementation you cannot see.

    ## What you see
    - The NLSpec Definition of Done: [paste DoD section]
    - The NLSpec Data Model: [paste data model section]
    - The project language and test framework: [language, Cucumber/Gherkin]

    ## What you CANNOT see
    - Any implementation code (it doesn't exist yet)
    - The NLSpec How section (that's for the green team)

    ## Your task
    For each Definition of Done item, write a Gherkin .feature file with
    concrete scenarios. Then write step definitions that implement the
    assertions.

    Rules:
    - Every DoD checkbox must have at least one scenario
    - The integration smoke test from the NLSpec must be a feature file
    - Include edge cases and error paths from the DoD
    - Tests must be runnable via [Cucumber command for language]
    - Write tests to [workspace path]

    When done, list all .feature files and the DoD items they cover."
)
```

### Phase 1b: Review Red Team Tests Against NLSpec

Spawn two reviewers in parallel:

```
Agent(
    subagent_type="foundry:review:red-team-test-reviewer",
    prompt="Review these test files against the NLSpec Definition of Done.

    NLSpec DoD: [paste DoD section]
    Test files: [paths to .feature files and step definitions]

    Return findings as JSON matching the findings schema."
)

Agent(
    subagent_type="foundry:review:cucumber-reviewer",
    prompt="Review these Gherkin feature files and step definitions for quality.

    Test files: [paths to .feature files and step definitions]

    Return findings as JSON matching the findings schema."
)
```

The red-team-test-reviewer checks DoD coverage, assertion specificity, trivially satisfiable tests, and scope creep. The cucumber-reviewer checks Gherkin quality (declarative style, scenario independence, step discipline).

Also spawn the barrier-integrity-auditor to verify no implementation code leaked into the red team's context:

```
Agent(
    subagent_type="foundry:review:barrier-integrity-auditor",
    prompt="Audit the red team's prompt and workspace for barrier violations.
    Red team should see: NLSpec, spec. Red team must NOT see: implementation code, green workspace paths.
    Red team prompt: [paste the prompt that was sent to the red team]
    Red workspace contents: [list files in red workspace]"
)
```

If there are UNCOVERED or WEAK items, send feedback to the red team (as a new message to the same agent) with the specific gaps. Iterate until reviewers pass.
#### Divergence Check (Phase 1b)

If the red-team-test-reviewer flags a test referencing behavior **not present in the NLSpec DoD** (distinct from a quality or coverage issue), trigger the divergence evaluator:

1. Assemble `EvaluatorInput`: raw flagged test scenario (not a summary) + full NLSpec content + `divergence_phase=PHASE_1B`. Capture `red_test_paths` (paths to red team test files) for use at Phase 1 restart.
2. Spawn ephemeral divergence evaluator (spawned per divergence, terminated after invocation):
```
Agent(
    subagent_type="foundry:review:divergence-evaluator",
    prompt="EvaluatorInput: [nlspec_content: <full NLSpec text>, diverging_artifact: <raw test scenario>, divergence_phase: PHASE_1B]"
)
```
3. Route on `DivergenceJudgment.outcome`:
   - `VALUABLE` → invoke `spec_update_and_restart` (see Spec Update section below), passing `red_test_paths`; then restart Phase 1
   - `NOT_VALUABLE` → send red team back with `judgment.rationale`
   - `INCONCLUSIVE` → escalate to user (UserEscalation); pause for manual judgment

Only one evaluator invocation may be in flight at a time (sequential processing).


### Phase 2: Green Team — Implement

**Before spawning green:** run the red team's tests to get the initial failure list. Capture ONLY the test names and pass/fail status.

Spawn the green team subagent:

```
Agent(
    name="green-team",
    mode="bypassPermissions",
    prompt="You are the GREEN TEAM. Your job is to implement a feature
    that passes tests you cannot see.

    ## What you see
    - The NLSpec How section: [paste How section ONLY — not the Done section]
    - The NLSpec Data Model: [paste data model section]
    - The project language: [language]
    - Test outcomes (pass/fail only):
      [test_name_1: FAIL]
      [test_name_2: FAIL]
      [...]

    ## What you CANNOT see
    - Test code, assertions, error messages, step definitions
    - The NLSpec Definition of Done section
    - The .feature files

    ## Your task
    Implement the feature following the NLSpec How section guidance.
    Write code to [workspace path].

    You know which tests exist by name and whether they pass or fail.
    Use the test NAMES as hints about what behavior is expected.
    You must NOT try to read test files or access the red workspace.

    When done, tell me you're ready for a test run."
)
```

### Phase 2b: Test-Fix Inner Loop

**State: TestFailureTracker** (pipeline-run-scoped — reset all counters on Phase 1 restart)

Maintain per failing test:
- `test_id`: test identifier
- `consecutive_fails`: consecutive green iterations where this test failed (Phase 2b trigger fires → resets to 0; test passes → resets to 0; test content changes → resets to 1)
- `threshold`: default 3
- `test_content_hash`: hash of test file content; detects test changes between iterations

This is where you mediate. Loop:

1. **Assemble runner workspace** — Copy green's implementation + red's tests into a temporary directory
2. **Run tests** — Execute the test suite
3. **Filter outcomes** — Extract ONLY `test_name: PASS/FAIL`. Discard assertions, errors, stack traces.
4. **Update trackers** — PASS: reset `consecutive_fails=0`. FAIL: if test content hash changed, reset to 1; else increment `consecutive_fails`.
5. **Check divergence threshold** — For any test where `consecutive_fails >= threshold` (default 3):
   - Assemble `EvaluatorInput`: raw impl snippet most recently written by green + full NLSpec + `divergence_phase=PHASE_2B`
   - Spawn ephemeral divergence evaluator (foundry:review:divergence-evaluator)
   - Route on outcome:
     - Phase 2b: `VALUABLE` → invoke `spec_update_and_restart`, then restart Phase 1
     - `NOT_VALUABLE` → send green back with rationale; reset this test's tracker (`consecutive_fails=0`)
     - Phase 2b: `INCONCLUSIVE` → escalate to user (UserEscalation); pause for manual judgment
6. **Check termination** — All pass → Phase 3. Any fail → send filtered outcomes to green.
7. **Check bounds** — If green has iterated more than the configured limit (default 20), pause and ask the user.

**Send to green team ONLY:**
```
Test results:
  test_name: PASS/FAIL
N tests total, X passed, Y failed.
```
Never include: assertions, expected vs actual, stack traces, line numbers from test code.

### Spec Update and Pipeline Restart (`spec_update_and_restart`)

Triggered when a divergence check returns `VALUABLE`. **You MUST NOT write NLSpec content directly. The NLSpec agent is the sole author.**

1. **Check revision cap** — Read `PipelineRevisionState.revision_count`. If `revision_count >= revision_cap` (default 10), pause and present full `revision_history` to user before continuing.
2. **Commit current NLSpec (pre-overwrite)** — attributed to nlspec-agent:
   ```bash
   git add <nlspec_path> && git commit --author="nlspec-agent <nlspec-agent@foundry>" -m "nlspec: preserve pre-revision NLSpec before divergence update"
   ```
3. **Re-run NLSpec agent** with `NLSpecRerunInput`:
   - `original_spec_path`: path to original spec document
   - `existing_nlspec_path`: path to current NLSpec
   - `evaluator_feedback`: `judgment.gap_description` verbatim (not paraphrased)
4. **If NLSpec agent fails**: pause; present `judgment.gap_description` to user; do NOT commit; NLSpec unchanged.
5. **Commit new NLSpec (post-update)** — `commit_after` SHA attributed to nlspec-agent.
6. **Generate `ChangeSummary`** — Read before/after NLSpec files and produce:
   - `sections_added`: list of new section headings
   - `sections_modified`: list of changed section headings
   - `requirements_delta`: list of added/removed requirements
7. **Update revision state** — Increment `revision_count`; append `RevisionRecord(commit_before, commit_after)` to `revision_history`.
8. **Restart Phase 1** — Pass `Phase1RestartPackage` to red team:
   - `existing_tests`: current red team test files, unmodified (red team receives existing tests unchanged)
   - `new_nlspec_path`: path to new NLSpec (red team receives new_nlspec_path)
   - `change_summary`: `ChangeSummary` from step 6 (red team receives change_summary)
   - `red_test_paths`: paths to current red team test files

   At restart: re-initialize TestFailureTracker (reset all counters; pipeline-run-scoped state cleared). Orchestrator reviews removed tests against new NLSpec before continuing. Red team reviews existing tests against new NLSpec + change summary, revises or extends as needed. Red team MUST NOT discard previously-passing tests without flagging. Phase 1b review runs after revision.

### Phase 3: Review

When all tests pass, spawn two reviewers in parallel:

Spawn these reviewers in parallel:

**Green team reviewer** (sees implementation, not tests):
```
Agent(
    subagent_type="foundry:review:green-team-reviewer",
    prompt="Review this implementation for code quality under information barrier constraints.

    NLSpec How section: [paste How section]
    Implementation: [read green workspace files]
    Test outcomes: [all passing]

    CRITICAL: You must NOT see test code, .feature files, step definitions, or the NLSpec Done section.

    Return findings as JSON matching the findings schema."
)
```

**Red team test reviewer** (sees tests, not implementation):
```
Agent(
    subagent_type="foundry:review:red-team-test-reviewer",
    prompt="Final review of test suite thoroughness.

    NLSpec Definition of Done: [paste DoD]
    Test files: [read red workspace files]

    You do NOT see the implementation.

    Return findings as JSON matching the findings schema."
)
```

**Language-specific reviewer** (conditional — dispatch based on detected language):
```
# Dispatch the appropriate language reviewer based on the project:
# Rust  → foundry:review:rust-reviewer
# Swift → foundry:review:swift-reviewer
# TS    → foundry:review:typescript-reviewer
# Also dispatch foundry:review:bazel-reviewer if BUILD files exist
# Also dispatch foundry:review:uniffi-bridge-reviewer if .udl files exist

Agent(
    subagent_type="foundry:review:[language]-reviewer",
    prompt="Review the implementation for [language]-specific issues.
    Implementation: [read green workspace files]
    Return findings as JSON matching the findings schema."
)
```

**Barrier integrity auditor** (always — final barrier check):
```
Agent(
    subagent_type="foundry:review:barrier-integrity-auditor",
    prompt="Final barrier audit. Check ALL prompts sent during this workflow.

    Green team prompt: [paste]
    Green reviewer prompt: [paste]
    Red team prompt: [paste]
    Red reviewer prompt: [paste]
    Test outcome labels sent to green: [paste]

    Verify no barrier violations occurred at any point."
)
```

**Always-on reviewers** (dispatch in parallel with the above):
- `foundry:review:correctness-reviewer` — logic errors, edge cases, state bugs in the implementation
- `foundry:review:testing-reviewer` — coverage gaps, weak assertions in the test suite
- `foundry:review:reliability-reviewer` — error handling, timeouts, retry logic (if the implementation touches I/O)

Merge all reviewer findings. Deduplicate across reviewers (same file + line + issue = one finding, keep highest severity).

If any reviewer rejects:
- Green-team-reviewer rejects → send feedback to green team, re-enter test-fix loop
- Red-team-test-reviewer rejects → send feedback to red team, red rewrites tests, green re-tests against new suite
- Barrier-integrity-auditor finds violations → **STOP** — fix the barrier leak before continuing
- Language/correctness/reliability reviewers find P0/P1 → send to appropriate team for fixing

### Phase 4: Finalize

When all reviewers approve (zero P0/P1 findings, barrier audit clean):
1. Commit the implementation and tests
2. Update the NLSpec frontmatter: `status: implemented`
3. Report summary: which DoD items are covered, test count, iteration count

### Configuration

These can be set via the conversation or a config file:

| Setting | Default | Description |
|---------|---------|-------------|
| `inner_loop_limit` | 20 | Max green fix iterations before pausing |
| `too_easily_threshold` | 3 | Consecutive passes before flagging "too easy" |
| `test_timeout` | 120s | Per-test-run timeout |
| `provider` | current model | Which model to use for subagents |

### Troubleshooting

**Green is stuck (keeps failing the same test):**
- Check if the test name gives enough information
- Consider spawning a temporary arbiter: an agent that sees ONLY the spec + the one failing test + the one test result, and judges whether the test or implementation is wrong

**Red tests are trivially satisfiable:**
- The red reviewer should catch this
- If it persists, the "too easily" threshold triggers red iteration

**Both teams are iterating without convergence:**
- Pause after the configured limit
- Ask the user to inspect both sides and arbitrate
