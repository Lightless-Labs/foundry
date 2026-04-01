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

This is where you mediate. Loop:

1. **Assemble runner workspace** — Copy green's implementation + red's tests into a temporary directory
2. **Run tests** — Execute the Cucumber test suite
3. **Filter outcomes** — Extract ONLY `test_name: PASS/FAIL` from the output. Discard everything else (assertions, error messages, stack traces, expected values).
4. **Check termination:**
   - All pass → proceed to Phase 3 (review)
   - Any fail → send filtered outcomes to green team via `SendMessage(to="green-team", ...)`
5. **Check bounds** — If green has iterated more than the configured limit (default 20), pause and ask the user what to do.

**Critical: when sending outcomes to the green team, include ONLY:**
```
Test results:
  test_login_valid_credentials: PASS
  test_login_invalid_password: FAIL
  test_login_expired_token: FAIL

3 tests total, 1 passed, 2 failed.
```

**Never include:** assertion text, expected vs actual values, stack traces, line numbers from test code, or any content from .feature files.

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
