---
date: 2026-04-01
topic: sudoku-solver
---

# Research: Sudoku Solver

## Codebase Context
- Language: Rust (per Lightless Labs convention)
- Build system: Cargo
- No existing sudoku code in the repo

## Domain Knowledge

Sudoku is a constraint satisfaction problem on a 9x9 grid divided into nine 3x3 boxes. Each row, column, and box must contain digits 1-9 exactly once.

### Solving Approaches (in order of sophistication)

1. **Brute-force backtracking** — try digits 1-9 in each empty cell, backtrack on contradiction. Simple but slow (O(9^81) worst case).

2. **Constraint propagation** — maintain a set of candidates per cell. When a cell is assigned, eliminate that digit from all peers (same row, column, box). Two key strategies:
   - **Naked singles**: cell with exactly one candidate → assign it
   - **Hidden singles**: digit appears as candidate in only one cell in a row/column/box → assign it

3. **Constraint propagation + backtracking** — use propagation to reduce the search space, then backtrack when stuck. This is the standard approach — solves any valid puzzle efficiently.

4. **Advanced constraint techniques** — naked pairs/triples, X-wing, swordfish, etc. Needed for rating puzzle difficulty, not for solving.

### Input/Output Conventions

- **Standard input**: 81 characters, left-to-right top-to-bottom. Digits 1-9 for givens, `0` or `.` for blanks.
- **Grid format**: 9 lines of 9 characters, optionally with separators.
- **Output**: same format as input, all cells filled.

### Validation Rules

- Input must be exactly 81 cells
- Given digits must not violate row/column/box uniqueness
- A valid puzzle has exactly one solution (but solvers typically find the first solution)

### Rust Libraries

- No external dependencies needed — the algorithm is straightforward with `std` only.

## Test Landscape

- Easy to test with known puzzle/solution pairs
- Edge cases: empty board (multiple solutions), already solved board, invalid board, unsolvable board
- Well-known test puzzles available (e.g., "Arto Inkala's hardest Sudoku")
