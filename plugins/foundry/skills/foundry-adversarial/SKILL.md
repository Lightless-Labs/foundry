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

3. Create the workspace structure (or use git worktrees for real filesystem isolation):
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

Spawn a reviewer subagent:

```
Agent(
    subagent_type="general-purpose",
    prompt="Review these test files against the NLSpec Definition of Done.

    NLSpec DoD: [paste DoD section]
    Test files: [paths to .feature files and step definitions]

    For each DoD item:
    - Is it covered by at least one test scenario?
    - Does the scenario actually test the requirement (not just touch it)?
    - Are the assertions specific enough to catch a wrong implementation?

    For each test:
    - Does it test something NOT in the DoD? (scope creep — flag it)
    - Is the assertion meaningful or trivially satisfiable?

    Return: COVERED items, UNCOVERED items, WEAK items (covered but poorly),
    EXTRA items (not in DoD)."
)
```

If there are UNCOVERED or WEAK items, send feedback to the red team (as a new message to the same agent) with the specific gaps. Iterate until the reviewer passes.

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

**Green reviewer** (sees implementation, not tests):
```
Agent(
    subagent_type="general-purpose",
    prompt="Review this implementation for code quality.

    NLSpec How section: [paste How section]
    Implementation: [read green workspace files]
    Test outcomes: [all passing]

    You do NOT see the test code. Evaluate:
    - Does the implementation follow the NLSpec How section guidance?
    - Is the code well-structured and maintainable?
    - Are there hardcoded values or shortcuts that would break on edge cases?
    - Is error handling robust?

    Return: APPROVE or REJECT with specific feedback."
)
```

**Red reviewer** (sees tests, not implementation):
```
Agent(
    subagent_type="general-purpose",
    prompt="Review these tests for thoroughness.

    NLSpec Definition of Done: [paste DoD]
    Test files: [read red workspace files]

    You do NOT see the implementation. Evaluate:
    - Do the tests cover all DoD items?
    - Are assertions specific enough?
    - Could a trivially wrong implementation pass these tests?
    - Are edge cases tested?

    Return: APPROVE or REJECT with specific feedback."
)
```

If either reviewer rejects:
- Green rejection → send feedback to green team, re-enter test-fix loop
- Red rejection → send feedback to red team, red rewrites tests, green re-tests against new suite

### Phase 4: Finalize

When both reviewers approve:
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
