# Example: Rubik's Cube Solver — Convention Mismatch Case Study

This directory contains a Rubik's cube solver built using the Foundry adversarial workflow. Unlike the sudoku example (30/30 clean pass), this one hit a **convention mismatch** — the most instructive failure mode of the adversarial process.

## What Happened

```
research → spec → NLSpec → red team (46 tests) → green team implements → 31/46 pass → DEADLOCK
```

The red team and green team independently derived incompatible facelet permutation conventions from the same NLSpec. Both were internally self-consistent (self-tests passed) but produced different results for the same move applied to the same cube state.

## The Convention Mismatch

The NLSpec described the Kociemba facelet convention with corner mappings like `URF: [8, 9, 20]` and move cycles like `corners: [URF, UBR, DRB, DFR]`. Both teams interpreted this correctly in the abstract — but the mapping from abstract cycles to concrete 54-element permutation arrays has multiple valid interpretations depending on:

1. Push vs pull permutation convention
2. Sticker cycling direction on adjacent faces (reversed for B face)
3. Corner orientation convention (which facelet is twist-0)

The red team derived one set of permutation tables. The green team derived another. Both pass their own self-tests (R then R' = identity). But R applied to the solved cube produces different strings.

## Why This Matters

This is the adversarial process working as designed — it **surfaced a spec defect** that conventional development would hide. In a normal workflow, one developer writes both tests and implementation using the same convention, so the mismatch never appears. The adversarial barrier forces independent derivation, which exposes convention ambiguity.

## The Fix (for future specs)

**Golden test vectors.** The NLSpec should include:

```
### Golden Test Vectors (from Kociemba reference implementation)
- Applying R to solved → "UUBUURUURFRRRRRRRRDFFDFFDFFDDLDDLDDLLLLLLLLLLBBBBBBBBBB"
- Applying U R to solved → "[exact string]"
- Applying R U R' U' to solved → "[exact string]"
```

These vectors are sourced from a reference implementation, not derived by the spec author. Both teams must match them. Convention mismatches become test failures on the golden vectors before they propagate into complex scenarios.

The `spec-completeness-reviewer` agent has been updated to flag the absence of golden test vectors for any spec involving state transformations as P0.

## Artifacts

| Phase | Artifact | Description |
|-------|----------|-------------|
| **Research** | [`docs/research/`](docs/research/) | Kociemba algorithm, facelet conventions, Rust implementations |
| **Spec** | [`docs/specs/`](docs/specs/) | 10 requirements, 5 behaviors |
| **NLSpec** | [`docs/nlspecs/`](docs/nlspecs/) | 820-line spec with pseudocode — missing golden test vectors |
| **Red Team Tests** | [`red/tests/`](red/tests/) | 46 integration tests |
| **Green Team** | [`green/src/`](green/src/) | Full Kociemba two-phase solver |
| **Mismatch Analysis** | [docs/solutions/](../../docs/solutions/workflow-issues/orchestrator-reconciliation-breaks-provenance-20260401.md) | Institutional learning |

## Current State

- 31/46 tests pass
- 15 failures are all convention mismatch (not implementation bugs)
- Solver correctly handles: parsing, validation, solved cube, superflip
- Solver produces valid solutions (internally consistent) that the test can't verify due to convention disagreement

## Running It

```bash
# Solve a cube (54-char facelet string, URFDLB order)
cargo run -- "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
# → "Already solved"

# Run tests (31 pass, 15 fail due to convention mismatch)
cargo test
```
