---
date: 2026-04-02
topic: rubiks-solver
status: active
research: docs/research/2026-04-02-rubiks-solver-research.md
---

# Specification: Rubik's Cube Solver

## Problem Statement

**Status quo:** Rubik's cube solving is a well-studied combinatorial problem with ~4.3×10¹⁹ possible states. Efficient solvers use Kociemba's two-phase algorithm to find near-optimal solutions.

**Pain:** We need a second, harder example to demonstrate the Foundry adversarial workflow on a non-trivial algorithm with precise state representation, group theory, and lookup tables.

**Solution:** A Rust CLI solver that reads a cube state as a 54-character facelet string, validates it, solves it using Kociemba's two-phase algorithm, and outputs the solution as a move sequence.

## Actors and Boundaries

- **User** provides a cube state as a 54-character facelet string (URFDLB face order) via CLI argument or stdin
- **Solver** validates, generates tables (on first run or from cache), solves, and outputs the move sequence
- No GUI, no interactive mode, no physical cube interface

## Requirements

- R1. Accept input as a 54-character facelet string using face letters U/R/F/D/L/B (Kociemba format, URFDLB order)
- R2. Accept input from CLI argument or stdin
- R3. Validate the cube state: exactly 54 characters, 9 of each face letter, valid center facelets, solvable orientation/permutation parity
- R4. Solve using Kociemba's two-phase algorithm with IDA* search
- R5. Generate move and pruning tables on first run; cache them to disk for subsequent runs
- R6. Output the solution as a space-separated move sequence in Singmaster notation (U, R, F, D, L, B with ' for inverse, 2 for double)
- R7. Exit with code 0 on success, 1 on invalid input, 2 on unsolvable state
- R8. Handle edge cases: already-solved cube (output empty or "Already solved"), identity state
- R9. Find solutions of 25 moves or fewer (near-optimal)
- R10. Solve within 5 seconds for any valid scramble

## Behaviors

### Behavior: Parse Input
- **Trigger:** Program invoked with facelet string
- **Input:** 54-character string of face letters (U/R/F/D/L/B)
- **Process:** Validate length, validate characters, map to internal facelet representation
- **Output:** Facelet array of 54 values
- **Errors:** Wrong length → exit 1. Invalid character → exit 1 with position.

### Behavior: Validate Cube
- **Trigger:** Facelets parsed
- **Input:** 54 facelets
- **Process:** Check 9 of each face, centers are correct (fixed positions 4,13,22,31,40,49), convert to cubie representation, check corner orientation sum ≡ 0 (mod 3), edge orientation sum ≡ 0 (mod 2), corner permutation parity = edge permutation parity
- **Output:** Valid → proceed. Invalid → error message + exit 1.
- **Errors:** Wrong sticker counts, wrong centers, orientation parity violation, permutation parity violation.

### Behavior: Generate Tables
- **Trigger:** Solver needs tables and they don't exist on disk
- **Input:** None (tables are derived from group theory)
- **Process:** Generate move tables (how coordinates change under each move) and pruning tables (lower bound on moves needed). Cache to a binary file.
- **Output:** Tables in memory, cached to `~/.rubiks-solver/tables.bin` or local directory
- **Errors:** Disk write failure → warn but continue (tables stay in memory only)

### Behavior: Solve
- **Trigger:** Valid cube + tables loaded
- **Input:** Cube coordinates + tables
- **Process:** Phase 1: IDA* search to reduce cube to G1 subgroup (corner orientation = 0, edge orientation = 0, UD slice edges in UD slice). Phase 2: IDA* search within G1 to solve completely. Iterate with decreasing max total length to find shorter solutions.
- **Output:** Move sequence
- **Errors:** No solution within move limit → exit 2 (should not happen for valid cubes)

### Behavior: Output
- **Trigger:** Solution found
- **Input:** Move sequence
- **Process:** Format as space-separated Singmaster notation
- **Output:** Printed to stdout, one line
- **Errors:** None

## Key Decisions

- **Decision:** Kociemba two-phase with split pruning tables
  - **Rationale:** Near-optimal solutions (≤25 moves) in milliseconds. Split pruning tables (~2MB) avoid the memory cost of full symmetry-reduced tables (~67MB).
  - **Rejected:** Korf's IDA* (>1GB tables, slower), beginner's method (too many moves), Thistlethwaite (4 phases, more complex for similar results).

- **Decision:** Facelet string input (Kociemba format)
  - **Rationale:** Industry standard, used by Kociemba's solver and most online tools.
  - **Rejected:** Scramble moves as input (requires starting from solved + applying moves — less general).

- **Decision:** No external dependencies except `clap` for CLI
  - **Rationale:** The algorithm is self-contained. Table generation and IDA* search need only arrays and arithmetic.

## Scope Boundaries

- **In scope:** Parse, validate, solve, output for standard 3×3×3 Rubik's cube
- **Out of scope:** 2×2, 4×4, or larger cubes. Cube visualization. Optimal solver. Move animation. Scramble generation.
- **Future:** Optimal solving via extended tables, multi-threaded search, WASM build for web

## Success Criteria

- Solves any valid 3×3×3 Rubik's cube state
- Solutions are ≤25 moves
- Solve time <5 seconds for any scramble (after table generation)
- Table generation <30 seconds on first run
- Rejects invalid cube states with clear error messages

## Open Questions

### Resolved
- Input format → 54-char facelet string, URFDLB order
- Output format → space-separated Singmaster notation
- Table caching → binary file, local directory or ~/.rubiks-solver/

### Deferred
- Table generation optimization (could parallelize)
- Exact pruning table strategy (split vs symmetry-reduced — implement split first, optimize later)
