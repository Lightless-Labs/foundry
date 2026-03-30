---
date: 2026-03-29
topic: red-green-adversarial-workflow
---

# Red/Green Adversarial Development Workflow

## What We're Building

An adversarial red/green team workflow within Foundry where two isolated agent teams — one writing tests, one writing implementation — develop a feature from a shared spec and API contract without seeing each other's work. The test runner is the only entity that crosses the information barrier, reporting pass/fail outcomes to the green team and triggering red team iteration when needed.

Foundry is a single Rust binary that spawns AI CLI subprocesses (Claude Code, Codex, Gemini, or others) in controlled working directories to enforce the information barrier at the filesystem level.

## Workflow Phases

### Phase 1: Spec Refinement

```
raw spec -> refine agent -> refined spec -> review -> (iterate or approve)
```

A single agent refines the raw spec. A reviewer evaluates the refined spec against the original intent. Iterates until the reviewer approves.

### Phase 2: Contract Derivation

```
refined spec -> contract agent -> API/interface contract -> review -> (iterate or approve)
```

The contract defines types, behaviors, shapes, and valid errors. No example payloads — those would bias both teams. The contract is the shared artifact that both red and green teams work from.

### Phase 3: Parallel Red/Green Execution

**Red Team** (test definition):

```
(spec + contract) -> define tests (Gherkin + step definitions) -> review -> iterate
```

Iteration triggers:
- Review rejects the tests
- Green passes a given test the first n times (passes "too easily")
- Green fails a test (test might need fixing)

Red team uses Gherkin/Cucumber and the appropriate language-specific testing library. Step definitions contain the actual assertions. The `.feature` files contain specific scenarios with values — both are invisible to the green team.

**Green Team** (implementation):

```
(spec + contract) -> implement -> run red's tests -> see outcomes only ->
  if any fail -> fix based on outcome labels -> re-run (inner loop, no review)
  if all pass -> review -> if rejected -> fix based on feedback -> re-run
```

The green team's inner loop (implement -> test -> fix) does not escalate to review until all tests pass. This keeps the reviewer from wasting cycles on broken code.

### Information Barriers

| Entity         | Sees                                           | Never sees                          |
|----------------|------------------------------------------------|-------------------------------------|
| Green team     | spec, contract, test outcome labels (pass/fail per test name) | test code, assertions, error messages, step definitions |
| Green reviewer | spec, contract, implementation, test outcomes  | test code                           |
| Red team       | spec, contract                                 | implementation                      |
| Red reviewer   | spec, contract, test code (Gherkin + step defs)| implementation                      |
| Test runner    | both (execution only, no judgment)             | -                                   |

### Enforcement

The information barrier is enforced at the filesystem level. Each team operates in a controlled working directory containing only the artifacts it is permitted to see. The test runner executes in a directory that contains both, but produces only outcome labels (test name + pass/fail) visible to the green team.

## Key Decisions

- **This is Foundry.** The red/green workflow is not a separate tool. It extends Foundry's graph engine, review gates, and clean-room role separation with filesystem-level information barriers and CLI subprocess spawning.
- **Single binary, subprocess spawning.** Foundry spawns AI CLI subprocesses (Claude Code, Codex, Gemini) in controlled working directories. The binary is the orchestrator; the AI CLIs are the workers.
- **No example payloads in the contract.** Types, behaviors, shapes, valid errors only. Examples bias both teams toward specific implementations rather than spec-faithful ones.
- **Green sees outcome labels only.** Not `.feature` files, not assertions, not error messages. Just `test_name: PASS/FAIL`. This prevents gaming specific assertions.
- **Green inner loop skips review.** Implement -> test -> fix cycles repeat without review escalation until all tests pass. Review is only for code that satisfies the test suite.
- **Red team is a live participant.** It iterates when review rejects, when green passes too easily, or when green fails. The red team is not fire-and-forget.
- **Red and green are single-model for now.** Reviewer can be a multi-model panel. Future: competing parallel implementations, cooperative multi-model teams within each role.
- **Gherkin/Cucumber for test definitions.** Red team writes `.feature` files and step definitions using the language-appropriate Cucumber library. The structured format separates intent (scenarios) from mechanism (step defs).
- **Code quality is the reviewer's domain.** If tests pass but the implementation is bad (hardcoded returns, poor structure), the green reviewer sends feedback. This is a code quality issue, not a test/spec issue.

## Coordination Protocol

Red and green cannot see each other's work, but they coordinate through the test runner as mediator:

1. Red defines initial test suite, gets it reviewed
2. Green implements against spec + contract
3. Green requests test run — blocked if red is running or in review (red holds write lock on the test suite)
4. Test runner executes; outcomes flow to green (labels only) and to red (full results)
5. Red may iterate tests based on: review feedback, green passing too easily, green failing
6. While red iterates, green continues implementing (can't run tests, but can work on code)
7. When red's cycle completes, green picks up the updated suite on next test run
8. Cycle terminates when: all tests pass AND green reviewer approves AND red reviewer approves

## Resolved Questions

- **Red iteration timing.** Green must not run tests while red is running or being reviewed. The test suite is a shared resource with red holding the write lock — green blocks until red's current cycle (write + review) completes. This prevents green from testing against a stale or mid-edit suite.
- **Test outcome granularity.** Test-level only: `test_name: PASS/FAIL`. No assertion counts, no error messages. Keeps the barrier strong.
- **Deadlock detection.** Maps to Foundry's retry limits. When exceeded, the workflow pauses and asks a human. Future improvement: multi-round multi-model escalation before human involvement.
- **Step definition language.** Auto-detect from the project (inspect Cargo.toml, package.json, Gemfile, etc.) with explicit override via configuration. Both modes supported.

## Open Questions

None at the product-definition level. Remaining choices belong in implementation planning.

## Future Improvements

- **Arbiter agent.** An ephemeral agent that sees spec + code + one specific test + its result, scoped to judge "is the test wrong or the implementation wrong?" A controlled, scoped breach of the information barrier. (See `todos/` for tracking.)
- **"Too easily" heuristic.** Replace the naive "passes first n times" trigger with an arbiter-like agent that evaluates whether a passing test is actually exercising the implementation meaningfully.
- **Competing implementations.** Multiple green teams implementing in parallel, with selection based on review quality, performance, or other criteria.
- **Multi-model cooperation within teams.** Multiple models collaborating within a single team role (e.g., one model writes, another reviews within the green team before escalating to the formal reviewer).
- **Prompt injection hardening.** Prevent green team from injecting instructions into test outcome labels or filenames that could influence the red team (and vice versa).
- **Multi-round escalation for deadlocks.** Before pausing for a human, attempt a multi-model escalation process (e.g., a different model arbitrates, or both teams present their case to a panel).

## Relationship to Existing Foundry

The red/green workflow extends Foundry's existing model:

- **Graph engine**: Red and green are parallel graph nodes with a shared dependency on the contract derivation phase. The test runner is a barrier-crossing edge.
- **Review gates**: Each team has its own review gate. Both must pass for the workflow to complete.
- **Clean-room separation**: Already the default in Foundry. The red/green model strengthens it from context bounding to filesystem isolation.
- **Runner trait**: The concrete `Runner` implementation spawns CLI subprocesses in isolated directories rather than calling AI APIs directly.
- **Retry limits**: Map to deadlock detection — max cycles before escalation.
- **Split execution**: Red/green is a split with information barrier constraints on artifact visibility.

## Next Steps

1. Create a TODO for the arbiter agent pattern as a future improvement
2. Move to planning: define the concrete graph shape, filesystem layout, CLI spawning mechanics, and Gherkin integration
