# Example: Sudoku Solver — Built with Foundry's Adversarial Workflow

This directory contains a complete Sudoku solver built using the Foundry adversarial red/green workflow. Every artifact is preserved: research, spec, NLSpec, red team tests, green team implementation, and the orchestration decisions.

## What Happened

```
research → spec → NLSpec → review → red team writes tests → green team implements → 30/30 pass
```

**The red team** wrote 30 integration tests from the NLSpec Definition of Done (section 6). They never saw the implementation.

**The green team** implemented the solver from the NLSpec How section (section 3) + test outcome labels only. They never saw the test code, assertions, error messages, or the Definition of Done section.

**The orchestrator** (Claude) mediated: ran tests, filtered outcomes to `test_name: PASS/FAIL`, sent filtered results to the green team.

## Artifacts

| Phase | Artifact | Description |
|-------|----------|-------------|
| **Research** | [`docs/research/`](docs/research/2026-04-01-sudoku-solver-research.md) | Domain research (solving approaches, input conventions, Rust patterns) |
| **Spec** | [`docs/specs/`](docs/specs/2026-04-01-sudoku-solver-spec.md) | 8 requirements, 4 behaviors, key decisions, scope boundaries |
| **NLSpec** | [`docs/nlspecs/`](docs/nlspecs/2026-04-01-sudoku-solver.nlspec.md) | Full Why/What/How/Done specification with pseudocode and 30 DoD checkboxes |
| **NLSpec Review** | (inline in conversation) | nlspec-fidelity-reviewer checked coverage, fidelity, structure, scope creep, ambiguity — all PASS |
| **Red Team Tests** | [`red/tests/solver_tests.rs`](red/tests/solver_tests.rs) | 30 integration tests covering all DoD items, using `std::process::Command` |
| **Green Team Implementation** | [`green/src/main.rs`](green/src/main.rs) | Constraint propagation + backtracking solver, ~400 lines of Rust |

## The Information Barrier in Practice

What the green team received for each test run (the ONLY signal):

```
test_accepts_81_digit_string: FAIL
test_accepts_argument: FAIL
test_accepts_dots_as_blanks: FAIL
test_accepts_stdin: FAIL
...
30 tests, 0 passed, 30 failed.
```

What the green team **never saw**:
- The test file (`red/tests/solver_tests.rs`)
- Assertion text (e.g., what specific output was expected)
- Error messages from failed assertions
- The NLSpec Definition of Done section
- The specific puzzle strings used in tests

The green team worked from the NLSpec How section's pseudocode + test names as behavioral hints.

## Running It

```bash
# Solve a puzzle (digits 1-9 for givens, 0 or . for blanks)
cargo run -- "530070000600195000098000060800060003400803001700020006060000280000419005000080079"

# From stdin
echo "530070000600195000098000060800060003400803001700020006060000280000419005000080079" | cargo run

# Run the red team's tests
cargo test
```

## Key Observations

1. **The adversarial pattern produced a correct implementation on the green team's first attempt.** 30/30 tests passed without the green team ever seeing the test code.

2. **Test names are meaningful signals.** The green team used test names like `test_rejects_duplicate_in_row` and `test_exit_code_2_on_unsolvable` as behavioral hints about what the tests expected.

3. **The NLSpec How section was sufficient for implementation.** The pseudocode in sections 3.1-3.7 gave the green team everything they needed — data structures, algorithms, and error handling — without exposing test criteria.

4. **The information barrier held.** The green team's implementation is a faithful translation of the NLSpec's algorithm, not a reverse-engineering of test assertions. This is the point of the adversarial approach.

## Install Foundry

```bash
claude plugin marketplace add github:Lightless-Labs/foundry
claude plugin install foundry
```

Then run `/foundry:forge "your feature description"` to use the same workflow on your own features.
