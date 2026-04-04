---
title: "Adversarial red/green development methodology for AI-driven software engineering"
module: foundry-workflow
date: 2026-03-31
problem_type: best_practice
component: development_workflow
severity: high
applies_when:
  - "Building features where implementation correctness matters more than speed"
  - "Working with AI agents that may game test assertions"
  - "Wanting independent verification of implementation against spec"
  - "Implementing features from NLSpec or structured specifications"
tags:
  - adversarial-workflow
  - red-green
  - methodology
  - information-barrier
  - nlspec
  - gherkin
  - agent-orchestration
---

# Adversarial Red/Green Development Methodology

## Context

Standard agentic development runs a single agent through implement-test-review cycles with full visibility of all artifacts. This creates a feedback loop where the implementation can be shaped to game specific test assertions rather than faithfully satisfy the specification. The agent sees the test, writes code to pass the test, not code to solve the problem.

## Guidance

### The Core Pattern

Two isolated agent teams develop a feature from a shared specification without seeing each other's work:

- **Red team** writes tests (Gherkin/Cucumber) from the spec's acceptance criteria
- **Green team** implements from the spec's implementation guidance + test outcome labels only
- **Orchestrator** mediates: runs tests, filters outcomes, iterates

The test runner is the only entity that crosses the information barrier. Green sees `test_login_valid_credentials: FAIL`, not the assertion text, not the expected values, not the step definitions.

### The Information Barrier

| Entity | Sees | Never sees |
|--------|------|------------|
| Red team | Spec (full), NLSpec Done section | Implementation code |
| Green team | Spec How section, test outcome labels | Test code, assertions, NLSpec Done section |
| Green reviewer | Implementation, test outcomes | Test code |
| Red reviewer | Test code | Implementation |
| Orchestrator | Everything | — |

The barrier is enforced structurally, not by instruction:
- **Filesystem isolation**: each team works in a controlled directory containing only permitted artifacts
- **Typed contexts**: role-scoped types make barrier violations compile-time errors (a `GreenContext` has no field for the test workspace path)
- **Filtered outcomes**: green receives `test_name: PASS/FAIL` only — no assertion text, no expected values, no stack traces

### The Full Pipeline

```
research → brainstorm → spec → NLSpec → adversarial implementation
```

Each artifact is reviewed before advancing:
1. **Spec** reviewed for completeness and testability
2. **NLSpec** reviewed against spec for coverage and fidelity (DoD mirrors body sections 1:1)
3. **Red team tests** reviewed against NLSpec DoD (every checkbox has a scenario)
4. **Green team implementation** reviewed for code quality (passes tests it can't see)

### NLSpec as the Shared Format

StrongDM's NLSpec (Why/What/How/Done) maps naturally:
- **Done section** → Red team formalizes into Gherkin tests
- **How section** → Green team implementation guidance
- **Recursive mirroring** (What→How→Done) creates a closed audit loop
- **Requirement density**: ~1 testable requirement per 25 lines of spec

### Coordination Protocol

1. Red defines initial test suite, gets it reviewed
2. Green implements against spec (not tests)
3. Green requests test run — blocked if red is running (red holds write lock)
4. Test runner executes; outcomes flow to green (labels only) and to red (full results)
5. Red may iterate when: review rejects, green passes "too easily", green fails
6. Cycle terminates when: all tests pass AND both reviewers approve

### Green Inner Loop

Green's implement → test → fix cycle does NOT escalate to review until all tests pass. This keeps the reviewer from wasting cycles on broken code. The inner loop has a separate iteration limit (default 20) — when exceeded, the workflow pauses for human intervention.

## Why This Matters

### It produces spec-faithful implementations

The green team implements against the spec's intent, not the test suite's specific assertions. It can't see what values the test expects, so it must implement the general behavior described in the spec.

### It catches agent gaming

Agents that hardcode return values, pattern-match on test inputs, or implement narrow special cases will fail when the red team writes comprehensive scenarios from the spec's acceptance criteria.

### It validates the spec itself

If the red team can't write meaningful tests from the spec, the spec is underspecified. If the green team can't implement from the spec + outcome labels, the spec's implementation guidance is insufficient. The process surfaces spec defects early.

### It provides independent verification

The red and green teams arrive at the same behavior from different directions — one from "what should be true" (tests) and one from "how to build it" (implementation). Agreement between independently derived artifacts is stronger evidence of correctness than a single agent's self-consistency.

## When to Apply

**Use adversarial red/green when:**
- The feature has clear acceptance criteria (testable requirements)
- Implementation correctness matters more than implementation speed
- The spec is structured enough to derive both tests and implementation independently
- You want independent verification that the implementation satisfies the spec

**Don't use when:**
- Exploratory prototyping (the spec doesn't exist yet)
- Trivial changes (config, renames, simple CRUD)
- The feature is inherently untestable (UI aesthetics, UX feel)
- Speed is the only constraint

## Examples

### Manual orchestration (no engine needed)

```python
# 1. Red team writes tests
red = Agent(prompt=f"""
You are the RED TEAM. Write Gherkin tests from this NLSpec Done section:
{nlspec_done_section}
You cannot see any implementation code.
""")

# 2. Run tests (all fail — nothing implemented yet)
outcomes = run_cucumber(red_workspace, green_workspace)
filtered = [f"{t.name}: {'PASS' if t.passed else 'FAIL'}" for t in outcomes]

# 3. Green team implements
green = Agent(prompt=f"""
You are the GREEN TEAM. Implement from this NLSpec How section:
{nlspec_how_section}
Test outcomes: {filtered}
You cannot see test code or assertions.
""")

# 4. Iterate until all pass
while not all_pass(outcomes):
    outcomes = run_cucumber(red_workspace, green_workspace)
    filtered = [f"{t.name}: {'PASS' if t.passed else 'FAIL'}" for t in outcomes]
    SendMessage(to="green-team", prompt=f"Test results:\n{filtered}")
```

### Session evidence

During Foundry's implementation, the red/green pattern was applied to the CliRunner:
- Red team wrote 19 tests before any implementation existed
- Green team implemented from spec + outcome labels only
- Result: 19/19 passed on first implementation
- Neither team saw the other's work

## Related

- [Compile-time information barriers](../best-practices/compile-time-information-barriers-via-role-scoped-types.md)
- [Adversarial self-application](../best-practices/adversarial-self-application-for-validation.md)
- [Integration gaps from parallel implementation](../workflow-issues/parallel-subagent-waves-require-integration-tests.md)
- [Orchestrator reconciliation breaks provenance](../workflow-issues/orchestrator-reconciliation-breaks-provenance-20260401.md) — field report from first real application (middens Phase 3)
- NLSpec format: https://jhugman.com/posts/on-nlspecs/
- StrongDM Attractor: https://github.com/strongdm/attractor
