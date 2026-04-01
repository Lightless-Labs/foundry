---
date: 2026-04-01
topic: sudoku-solver
source_spec: docs/specs/2026-04-01-sudoku-solver-spec.md
status: reviewed
---

# Sudoku Solver NLSpec

A command-line Sudoku solver that reads a 9x9 puzzle, validates it, solves it using constraint propagation and backtracking, and prints the completed board. Intended for use as a Foundry adversarial workflow example.

## Table of Contents

1. [Why](#1-why)
2. [What](#2-what)
3. [How](#3-how)
4. [Out of Scope](#4-out-of-scope)
5. [Design Decision Rationale](#5-design-decision-rationale)
6. [Definition of Done](#6-definition-of-done)

---

## 1. Why

### 1.1 Problem Statement

Sudoku solving is a well-understood constraint satisfaction problem. We need a clean, testable implementation to demonstrate the Foundry adversarial red/green workflow end-to-end — where the red team writes tests from this NLSpec's Definition of Done and the green team implements from the How section, neither seeing the other's work.

### 1.2 Design Principles

**Zero dependencies.** The solver uses only Rust's standard library. The algorithm is simple enough that external crates add complexity without value. This keeps the example self-contained and the build instant.

**Constraint propagation first.** The solver must attempt to reduce the search space via constraint propagation before falling back to backtracking. Pure backtracking is O(9^81) worst case; propagation solves most puzzles without search.

**Fail fast on bad input.** Invalid input is rejected immediately with a clear error message and appropriate exit code, before any solving attempt.

### 1.3 Layering and Scope

This spec covers: parsing, validation, solving, and output of standard 9x9 Sudoku puzzles via a CLI interface. It does NOT cover: puzzle generation, difficulty rating, variant grids (16x16, Samurai), GUI, or interactive mode.

---

## 2. What

### 2.1 Data Model

```
RECORD Cell:
    row: u8        -- 0-8
    column: u8     -- 0-8
    box_id: u8     -- 0-8, derived: (row / 3) * 3 + (column / 3)

RECORD Board:
    cells: [[u8; 9]; 9]   -- 0 = empty, 1-9 = digit

RECORD CandidateBoard:
    candidates: [[CandidateSet; 9]; 9]  -- per-cell candidate sets

RECORD CandidateSet:
    bits: u16   -- bitmask, bit N set means digit N is a candidate (bits 1-9 used)

ENUM SolveResult:
    SOLVED(Board)
    UNSOLVABLE
    INVALID(String)   -- validation error message

ENUM ParseError:
    WRONG_LENGTH { actual: usize }
    INVALID_CHARACTER { char: char, position: usize }

ENUM ValidationError:
    DUPLICATE_IN_ROW { digit: u8, row: u8 }
    DUPLICATE_IN_COLUMN { digit: u8, column: u8 }
    DUPLICATE_IN_BOX { digit: u8, box_id: u8 }
```

### 2.2 Architecture

Three modules:
- `parse` — input parsing and validation
- `solve` — constraint propagation + backtracking
- `main` — CLI entry point, output formatting

### 2.3 Vocabulary

- **Given**: a digit provided in the puzzle input (1-9)
- **Blank**: an empty cell (represented as 0 or `.` in input)
- **Candidate set**: the set of digits that could legally occupy a blank cell
- **Peer**: cells that share a row, column, or box with a given cell (20 peers per cell)
- **Naked single**: a cell with exactly one candidate — it must be that digit
- **Hidden single**: a digit that appears as candidate in only one cell within a row/column/box — that cell must be that digit
- **Propagation**: the process of eliminating candidates based on assignments
- **Backtracking**: guessing a digit for a cell, then recursing; undoing the guess if it leads to contradiction

---

## 3. How

### 3.1 Parse Input

```
FUNCTION parse(input: String) -> Result<Board, ParseError>:
    -- Step 1: Normalize
    cleaned = input.replace_whitespace("").replace("\n", "")

    -- Step 2: Validate length
    IF cleaned.length != 81:
        RETURN Err(WRONG_LENGTH { actual: cleaned.length })

    -- Step 3: Parse characters
    board = empty 9x9 grid
    FOR i IN 0..81:
        char = cleaned[i]
        IF char == '.' OR char == '0':
            board[i / 9][i % 9] = 0
        ELSE IF char >= '1' AND char <= '9':
            board[i / 9][i % 9] = char.to_digit()
        ELSE:
            RETURN Err(INVALID_CHARACTER { char, position: i })

    RETURN Ok(board)
```

### 3.2 Validate Board

```
FUNCTION validate(board: Board) -> Result<(), ValidationError>:
    -- Check rows
    FOR row IN 0..9:
        seen = empty set
        FOR col IN 0..9:
            digit = board[row][col]
            IF digit != 0:
                IF digit IN seen:
                    RETURN Err(DUPLICATE_IN_ROW { digit, row })
                seen.insert(digit)

    -- Check columns
    FOR col IN 0..9:
        seen = empty set
        FOR row IN 0..9:
            digit = board[row][col]
            IF digit != 0:
                IF digit IN seen:
                    RETURN Err(DUPLICATE_IN_COLUMN { digit, column: col })
                seen.insert(digit)

    -- Check boxes
    FOR box_id IN 0..9:
        seen = empty set
        box_row = (box_id / 3) * 3
        box_col = (box_id % 3) * 3
        FOR r IN box_row..box_row+3:
            FOR c IN box_col..box_col+3:
                digit = board[r][c]
                IF digit != 0:
                    IF digit IN seen:
                        RETURN Err(DUPLICATE_IN_BOX { digit, box_id })
                    seen.insert(digit)

    RETURN Ok(())
```

### 3.3 Initialize Candidates

```
FUNCTION initialize_candidates(board: Board) -> Option<CandidateBoard>:
    candidates = 9x9 grid of full candidate sets (all digits 1-9)

    FOR row IN 0..9:
        FOR col IN 0..9:
            digit = board[row][col]
            IF digit != 0:
                -- Given cell: set candidate to just this digit
                candidates[row][col] = singleton(digit)
                -- Eliminate from all peers
                IF NOT eliminate(candidates, row, col, digit):
                    RETURN None   -- contradiction from givens alone

    RETURN Some(candidates)
```

### 3.4 Constraint Propagation

```
FUNCTION eliminate(candidates: &mut CandidateBoard, row: u8, col: u8, digit: u8) -> bool:
    -- Remove digit from all peers of (row, col)
    FOR (pr, pc) IN peers(row, col):
        IF candidates[pr][pc].contains(digit):
            candidates[pr][pc].remove(digit)

            IF candidates[pr][pc].is_empty():
                RETURN false   -- contradiction: no candidates left

            IF candidates[pr][pc].count() == 1:
                -- Naked single: propagate recursively
                single = candidates[pr][pc].only_digit()
                IF NOT eliminate(candidates, pr, pc, single):
                    RETURN false

    -- Hidden singles: check if digit now appears in only one cell in any unit
    FOR unit IN units_containing(row, col):
        places = cells in unit where digit is still a candidate
        IF places.count() == 0:
            RETURN false   -- contradiction: digit has no home
        IF places.count() == 1:
            target = places[0]
            IF candidates[target.row][target.col].count() > 1:
                candidates[target.row][target.col] = singleton(digit)
                IF NOT eliminate(candidates, target.row, target.col, digit):
                    RETURN false

    RETURN true

FUNCTION peers(row: u8, col: u8) -> Vec<(u8, u8)>:
    -- All cells in same row, column, or box, excluding (row, col) itself
    -- 20 peers per cell

FUNCTION units_containing(row: u8, col: u8) -> [Vec<(u8, u8)>; 3]:
    -- The row unit, column unit, and box unit containing (row, col)
```

### 3.5 Solve with Backtracking

```
FUNCTION solve(candidates: CandidateBoard) -> Option<Board>:
    -- Check if solved
    IF all cells have exactly one candidate:
        RETURN Some(extract_board(candidates))

    -- Find the unfilled cell with fewest candidates (MRV heuristic)
    (row, col) = cell with minimum candidates.count() where count > 1

    -- Try each candidate
    FOR digit IN candidates[row][col]:
        trial = deep_clone(candidates)
        trial[row][col] = singleton(digit)
        IF eliminate(trial, row, col, digit):
            result = solve(trial)
            IF result IS Some:
                RETURN result

    RETURN None   -- all candidates tried, none worked = unsolvable
```

-- Behavior:
- On a valid puzzle with one solution, returns that solution
- On an unsolvable puzzle, returns None after exhausting all branches
- Uses MRV (minimum remaining values) heuristic to pick the next cell, minimizing branching factor
- Constraint propagation prunes most branches before they are explored

### 3.6 Output

```
FUNCTION format_board(board: Board) -> String:
    result = ""
    FOR row IN 0..9:
        FOR col IN 0..9:
            result.push(board[row][col].to_char())
        result.push('\n')
    RETURN result
```

### 3.7 CLI Entry Point

```
FUNCTION main():
    -- Read input from argument or stdin
    input = IF args.len() > 1: args[1] ELSE: read_stdin_line()

    -- Parse
    board = parse(input)
    IF board IS Err(e):
        eprintln("Error: {e}")
        exit(1)

    -- Validate
    validation = validate(board)
    IF validation IS Err(e):
        eprintln("Error: {e}")
        exit(1)

    -- Initialize candidates
    candidates = initialize_candidates(board)
    IF candidates IS None:
        eprintln("Error: Unsolvable puzzle (contradiction in givens)")
        exit(2)

    -- Solve
    result = solve(candidates)
    IF result IS None:
        eprintln("Error: Unsolvable puzzle")
        exit(2)

    -- Output
    print(format_board(result))
    exit(0)
```

---

## 4. Out of Scope

- **Puzzle generation.** Generating puzzles with a unique solution requires the solver plus additional constraint logic (minimality checking). Extension point: add a `generate` subcommand that uses the solver as a subroutine.
- **Difficulty rating.** Rating puzzles by human difficulty requires tracking which techniques the solver needed. Extension point: instrument the solver to log which techniques fired (naked singles, hidden singles, backtracking depth).
- **Variant grids.** 16x16, Samurai, or other non-standard grids. Extension point: parameterize the grid size and box dimensions.
- **GUI or interactive mode.** Extension point: expose the solver as a library crate that a GUI can call.

---

## 5. Design Decision Rationale

**Why constraint propagation + backtracking instead of pure backtracking?** Pure backtracking is O(9^81) worst case. Constraint propagation with naked singles and hidden singles solves most puzzles without any backtracking. For hard puzzles, propagation dramatically reduces the branching factor. The combination is the standard approach in the literature and practical solvers.

**Why bitmask for candidate sets instead of HashSet?** A 9-bit bitmask fits in a u16, requires no heap allocation, and supports constant-time set operations (union, intersection, count, remove). This matters in the backtracking loop where candidate sets are cloned at each branch point.

**Why MRV (minimum remaining values) heuristic?** When backtracking, choosing the cell with fewest remaining candidates minimizes the branching factor. A cell with 2 candidates creates 2 branches; a cell with 9 creates 9. MRV is the standard heuristic for CSP solvers and is trivial to implement.

**Why no external dependencies?** The algorithm needs only arrays, bitmasks, and recursion — all available in std. Adding crates would complicate the build, increase the example's surface area, and add nothing of value.

---

## 6. Definition of Done

### 6.1 Input Parsing (mirrors 3.1)
- [ ] Accepts 81-character string of digits 0-9 and dots
- [ ] Strips whitespace and newlines before parsing
- [ ] Rejects input with fewer than 81 valid characters
- [ ] Rejects input with more than 81 valid characters
- [ ] Rejects input containing characters other than 0-9 and `.`
- [ ] Reports the invalid character and its position in the error message
- [ ] Treats both `0` and `.` as blank cells

### 6.2 Board Validation (mirrors 3.2)
- [ ] Detects duplicate digits in a row
- [ ] Detects duplicate digits in a column
- [ ] Detects duplicate digits in a 3x3 box
- [ ] Reports which digit is duplicated and in which row/column/box
- [ ] Passes validation for a board with no duplicates (including an empty board)

### 6.3 Constraint Propagation (mirrors 3.3, 3.4)
- [ ] Initializes candidate sets correctly (each blank cell has digits 1-9 minus its peers' givens)
- [ ] Naked singles: when a cell has one candidate, assigns it and eliminates from peers
- [ ] Hidden singles: when a digit appears in only one cell in a unit, assigns it
- [ ] Detects contradictions during propagation (cell with zero candidates, digit with no home in a unit)
- [ ] Propagation alone solves easy/medium puzzles without backtracking

### 6.4 Backtracking (mirrors 3.5)
- [ ] Selects the cell with fewest candidates (MRV heuristic)
- [ ] Tries each candidate, recursing on a cloned state
- [ ] Returns the solution when all cells are assigned
- [ ] Returns None (unsolvable) when all branches are exhausted
- [ ] Hard puzzles (requiring backtracking) are solved correctly

### 6.5 Output (mirrors 3.6)
- [ ] Outputs 9 lines of 9 digits
- [ ] Each line terminated by a newline character (including the last line), no other separators or borders
- [ ] Output is a valid, complete Sudoku board

### 6.6 CLI (mirrors 3.7)
- [ ] Accepts puzzle as first command-line argument
- [ ] Accepts puzzle from stdin if no argument provided
- [ ] Exits with code 0 on successful solve
- [ ] Exits with code 1 on invalid input (parse error or validation error)
- [ ] Exits with code 2 on unsolvable puzzle
- [ ] Prints error messages to stderr, solution to stdout

### 6.7 Edge Cases (mirrors R8)
- [ ] Already-solved board: outputs it unchanged
- [ ] Board with one blank: fills the single missing digit
- [ ] Empty board (all zeros): finds a valid solution (any valid complete board)
- [ ] Minimum-clue puzzle (17 givens): solves correctly

### 6.8 Integration Smoke Test

```
FUNCTION integration_smoke_test():
    -- Easy puzzle (solvable by propagation alone)
    easy = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    result = run_solver(easy)
    ASSERT result.exit_code == 0
    ASSERT result.stdout == "534678912\n672195348\n198342567\n859761423\n426853791\n713924856\n961537284\n287419635\n345286179\n"

    -- Invalid input (too short)
    result = run_solver("12345")
    ASSERT result.exit_code == 1
    ASSERT result.stderr.contains("length")

    -- Invalid board (duplicate in row)
    bad = "11" + "0".repeat(79)
    result = run_solver(bad)
    ASSERT result.exit_code == 1
    ASSERT result.stderr.contains("duplicate")

    -- Unsolvable puzzle
    unsolvable = "516849732307605000809010020603924815125060000948370206030106074060400501701580063"
    result = run_solver(unsolvable)
    ASSERT result.exit_code == 2

    -- Already solved board
    solved = "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    result = run_solver(solved)
    ASSERT result.exit_code == 0
    ASSERT result.stdout == "534678912\n672195348\n198342567\n859761423\n426853791\n713924856\n961537284\n287419635\n345286179\n"
```
