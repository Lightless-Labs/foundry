---
date: 2026-04-01
topic: sudoku-solver
status: active
research: docs/research/2026-04-01-sudoku-solver-research.md
---

# Specification: Sudoku Solver

## Problem Statement

**Status quo:** Sudoku solving is a well-understood constraint satisfaction problem, but implementing one correctly involves handling validation, constraint propagation, and backtracking — a good test of specification-driven development.

**Pain:** We need a concrete example to demonstrate the Foundry adversarial red/green workflow end-to-end.

**Solution:** A Rust CLI sudoku solver that reads a puzzle, validates it, solves it using constraint propagation + backtracking, and outputs the solution.

## Actors and Boundaries

- **User** provides a puzzle as 81 characters via stdin or command-line argument
- **Solver** validates, solves, and outputs the completed board
- No network, no persistence, no GUI — pure computation

## Requirements

- R1. Accept input as 81 characters (digits 1-9 for givens, `0` or `.` for blanks)
- R2. Accept input from stdin (one line) or as a CLI argument
- R3. Validate the input board: exactly 81 cells, only valid characters, no duplicate digits in any row/column/box
- R4. Solve using constraint propagation (naked singles + hidden singles) before backtracking
- R5. Detect and report unsolvable puzzles (contradiction found during solving)
- R6. Output the solved board as 9 lines of 9 digits
- R7. Exit with code 0 on success, 1 on invalid input, 2 on unsolvable puzzle
- R8. Handle edge cases: already-solved board (pass through), empty board (find a solution), board with one blank (trivial)

## Behaviors

### Behavior: Parse Input
- **Trigger:** Program invoked with puzzle string
- **Input:** String of 81 characters
- **Process:** Strip whitespace, validate length is 81, validate each character is `0-9` or `.`, convert `.` to `0`
- **Output:** 9x9 grid of digits (0 = empty)
- **Errors:** Input too short/long → error message + exit 1. Invalid character → error message + exit 1.

### Behavior: Validate Board
- **Trigger:** Board parsed successfully
- **Input:** 9x9 grid
- **Process:** For each given digit, check no duplicate in same row, column, or 3x3 box
- **Output:** Board is valid → proceed to solve. Board is invalid → error message + exit 1.
- **Errors:** Duplicate digit in row/column/box → report which constraint is violated.

### Behavior: Solve
- **Trigger:** Valid board
- **Input:** 9x9 grid with some cells empty (0)
- **Process:**
  1. Initialize candidate sets (digits 1-9 minus peers' givens) for each empty cell
  2. Propagate constraints: eliminate assigned digits from peers, apply naked singles and hidden singles
  3. If all cells assigned → solved
  4. If any cell has zero candidates → contradiction → backtrack or report unsolvable
  5. If stuck (no singles found) → pick the cell with fewest candidates, try each, recurse
- **Output:** Solved 9x9 grid
- **Errors:** No solution exists → "Unsolvable puzzle" + exit 2

### Behavior: Output
- **Trigger:** Board solved
- **Input:** Solved 9x9 grid
- **Process:** Print 9 lines, each with 9 digits, no separators
- **Output:** Printed to stdout
- **Errors:** None

## Key Decisions

- **Decision:** Constraint propagation before backtracking
  - **Rationale:** Pure backtracking is O(9^81) worst case. Constraint propagation solves most puzzles without backtracking and dramatically reduces the search space for hard puzzles.
  - **Rejected:** Pure backtracking (too slow for hard puzzles), advanced techniques like X-wing (unnecessary complexity for a solver).

- **Decision:** No external dependencies
  - **Rationale:** The algorithm is simple enough with std. Keeping it dependency-free makes it a clean example.
  - **Rejected:** Using a SAT solver library (overkill), using a constraint programming library (hides the algorithm).

## Scope Boundaries

- **In scope:** Parse, validate, solve, output for standard 9x9 Sudoku
- **Out of scope:** Puzzle generation, difficulty rating, 16x16 variants, GUI, interactive mode
- **Future:** Could extend to support multiple solutions (print all), puzzle generation, and difficulty rating

## Success Criteria

- Solves any valid 9x9 Sudoku puzzle
- Rejects invalid inputs with clear error messages
- Detects unsolvable puzzles
- Runs in under 1 second on hard puzzles

## Open Questions

### Resolved
- Input format → 81 characters, `0` or `.` for blanks
- Output format → 9 lines of 9 digits

### Deferred
- Performance benchmarking against known hard puzzles (runtime experimentation)
