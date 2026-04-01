//! Red team integration tests for the sudoku-solver binary.
//!
//! These tests exercise the solver exclusively via its CLI interface using
//! `std::process::Command`. They are derived from the NLSpec Definition of Done
//! (sections 6.1 through 6.8) and are written WITHOUT knowledge of the
//! implementation.

use std::io::Write;
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Path to the compiled binary. Cargo puts it in the target directory.
fn solver_bin() -> String {
    // `cargo test` sets this env var to the directory containing built binaries
    let dir = env!("CARGO_BIN_EXE_sudoku-solver");
    dir.to_string()
}

/// Run the solver with the puzzle passed as a CLI argument.
fn run_with_arg(input: &str) -> std::process::Output {
    Command::new(solver_bin())
        .arg(input)
        .output()
        .expect("failed to execute solver binary")
}

/// Run the solver with the puzzle piped via stdin.
fn run_with_stdin(input: &str) -> std::process::Output {
    let mut child = Command::new(solver_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn solver binary");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write to stdin");
    }

    child.wait_with_output().expect("failed to wait on solver")
}

fn stdout_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Validate that a string represents a complete, valid Sudoku board.
/// Returns true if every row, column, and 3x3 box contains digits 1-9 exactly once.
fn is_valid_sudoku(board_str: &str) -> bool {
    let lines: Vec<&str> = board_str.lines().collect();
    if lines.len() != 9 {
        return false;
    }

    let mut grid = [[0u8; 9]; 9];
    for (r, line) in lines.iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() != 9 {
            return false;
        }
        for (c, ch) in chars.iter().enumerate() {
            match ch.to_digit(10) {
                Some(d) if d >= 1 && d <= 9 => grid[r][c] = d as u8,
                _ => return false,
            }
        }
    }

    // Check rows
    for r in 0..9 {
        let mut seen = [false; 10];
        for c in 0..9 {
            let d = grid[r][c] as usize;
            if seen[d] {
                return false;
            }
            seen[d] = true;
        }
    }

    // Check columns
    for c in 0..9 {
        let mut seen = [false; 10];
        for r in 0..9 {
            let d = grid[r][c] as usize;
            if seen[d] {
                return false;
            }
            seen[d] = true;
        }
    }

    // Check 3x3 boxes
    for box_r in 0..3 {
        for box_c in 0..3 {
            let mut seen = [false; 10];
            for r in 0..3 {
                for c in 0..3 {
                    let d = grid[box_r * 3 + r][box_c * 3 + c] as usize;
                    if seen[d] {
                        return false;
                    }
                    seen[d] = true;
                }
            }
        }
    }

    true
}

// ---------------------------------------------------------------------------
// Known puzzles and solutions
// ---------------------------------------------------------------------------

const EASY_PUZZLE: &str =
    "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

const EASY_SOLUTION: &str = "\
534678912
672195348
198342567
859761423
426853791
713924856
961537284
287419635
345286179
";

const EASY_PUZZLE_DOTS: &str =
    "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

const HARD_PUZZLE: &str =
    "800000000003600000070090200050007000000045700000100030001000068008500010090000400";

const SOLVED_BOARD: &str =
    "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

const SOLVED_BOARD_OUTPUT: &str = "\
534678912
672195348
198342567
859761423
426853791
713924856
961537284
287419635
345286179
";

/// A known unsolvable puzzle (contradictory givens).
const UNSOLVABLE_PUZZLE: &str =
    "516849732307605000809010020603924815125060000948370206030106074060400501701580063";

/// A puzzle with 17 givens (the proven minimum for a unique solution).
/// This is a well-known 17-clue puzzle.
const MINIMUM_CLUE_PUZZLE: &str =
    "000000010400000000020000000000050407008000300001090000300400200050100000000806000";

// ===========================================================================
// 6.1 Input Parsing
// ===========================================================================

#[test]
fn test_accepts_81_digit_string() {
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(output.status.code(), Some(0), "should accept valid 81-digit input");
    assert!(!stdout_str(&output).is_empty(), "should produce output");
}

#[test]
fn test_accepts_dots_as_blanks() {
    let output = run_with_arg(EASY_PUZZLE_DOTS);
    assert_eq!(output.status.code(), Some(0), "should accept dots as blanks");
    assert_eq!(stdout_str(&output), EASY_SOLUTION);
}

#[test]
fn test_accepts_zeros_as_blanks() {
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(output.status.code(), Some(0), "should accept zeros as blanks");
    assert_eq!(stdout_str(&output), EASY_SOLUTION);
}

#[test]
fn test_strips_whitespace() {
    // Insert spaces throughout the puzzle
    let spaced: String = EASY_PUZZLE
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && i % 9 == 0 {
                vec![' ', c]
            } else {
                vec![c]
            }
        })
        .collect();
    let output = run_with_arg(&spaced);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should strip whitespace before parsing. stderr: {}",
        stderr_str(&output)
    );
    assert_eq!(stdout_str(&output), EASY_SOLUTION);
}

#[test]
fn test_strips_newlines() {
    // Insert newlines every 9 characters
    let with_newlines: String = EASY_PUZZLE
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && i % 9 == 0 {
                vec!['\n', c]
            } else {
                vec![c]
            }
        })
        .collect();
    let output = run_with_arg(&with_newlines);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should strip newlines before parsing. stderr: {}",
        stderr_str(&output)
    );
    assert_eq!(stdout_str(&output), EASY_SOLUTION);
}

#[test]
fn test_rejects_too_short() {
    let output = run_with_arg("12345");
    assert_eq!(
        output.status.code(),
        Some(1),
        "should reject input shorter than 81 characters"
    );
    let err = stderr_str(&output);
    assert!(!err.is_empty(), "should print error to stderr");
}

#[test]
fn test_rejects_too_long() {
    let too_long = "0".repeat(82);
    let output = run_with_arg(&too_long);
    assert_eq!(
        output.status.code(),
        Some(1),
        "should reject input longer than 81 characters"
    );
    let err = stderr_str(&output);
    assert!(!err.is_empty(), "should print error to stderr");
}

#[test]
fn test_rejects_invalid_character() {
    // Place an 'x' at position 5 in an otherwise valid-length string
    let mut input: Vec<char> = "0".repeat(81).chars().collect();
    input[5] = 'x';
    let input_str: String = input.into_iter().collect();
    let output = run_with_arg(&input_str);
    assert_eq!(
        output.status.code(),
        Some(1),
        "should reject input with invalid characters"
    );
    let err = stderr_str(&output);
    // The NLSpec says: "Reports the invalid character and its position"
    assert!(
        err.contains('x'),
        "error should mention the invalid character 'x'. got: {err}"
    );
    assert!(
        err.contains('5'),
        "error should mention the position 5. got: {err}"
    );
}

#[test]
fn test_treats_dot_and_zero_as_equivalent() {
    // Build two equivalent inputs: one with zeros, one with dots for blanks
    let with_zeros = EASY_PUZZLE;
    let with_dots = EASY_PUZZLE_DOTS;

    let output_zeros = run_with_arg(with_zeros);
    let output_dots = run_with_arg(with_dots);

    assert_eq!(output_zeros.status.code(), Some(0));
    assert_eq!(output_dots.status.code(), Some(0));

    let sol_zeros = stdout_str(&output_zeros);
    let sol_dots = stdout_str(&output_dots);
    assert!(
        !sol_zeros.is_empty(),
        "solver should produce output for zero-based input"
    );
    assert_eq!(
        sol_zeros, sol_dots,
        "dot and zero inputs should produce identical solutions"
    );
}

// ===========================================================================
// 6.2 Board Validation
// ===========================================================================

#[test]
fn test_rejects_duplicate_in_row() {
    // Row 0 has two 1s at positions 0 and 1
    let mut input = "0".repeat(81);
    // Set first two chars to '1'
    let mut chars: Vec<char> = input.chars().collect();
    chars[0] = '1';
    chars[1] = '1';
    input = chars.into_iter().collect();
    let output = run_with_arg(&input);
    assert_eq!(
        output.status.code(),
        Some(1),
        "should reject board with duplicate in row"
    );
    let err = stderr_str(&output).to_lowercase();
    assert!(
        err.contains("duplicate") || err.contains("row"),
        "error should mention duplication issue. got: {err}"
    );
}

#[test]
fn test_rejects_duplicate_in_column() {
    // Column 0 has two 1s: at (0,0) and (1,0)
    let mut chars: Vec<char> = "0".repeat(81).chars().collect();
    chars[0] = '1'; // row 0, col 0
    chars[9] = '1'; // row 1, col 0
    let input: String = chars.into_iter().collect();
    let output = run_with_arg(&input);
    assert_eq!(
        output.status.code(),
        Some(1),
        "should reject board with duplicate in column"
    );
    let err = stderr_str(&output).to_lowercase();
    assert!(
        err.contains("duplicate") || err.contains("column"),
        "error should mention duplication issue. got: {err}"
    );
}

#[test]
fn test_rejects_duplicate_in_box() {
    // Box 0 (top-left 3x3) has two 1s: at (0,0) and (1,1)
    let mut chars: Vec<char> = "0".repeat(81).chars().collect();
    chars[0] = '1';  // row 0, col 0
    chars[10] = '1'; // row 1, col 1
    let input: String = chars.into_iter().collect();
    let output = run_with_arg(&input);
    assert_eq!(
        output.status.code(),
        Some(1),
        "should reject board with duplicate in box"
    );
    let err = stderr_str(&output).to_lowercase();
    assert!(
        err.contains("duplicate") || err.contains("box"),
        "error should mention duplication issue. got: {err}"
    );
}

#[test]
fn test_reports_which_duplicate() {
    // Duplicate digit 5 in row 0, positions 0 and 1
    let mut chars: Vec<char> = "0".repeat(81).chars().collect();
    chars[0] = '5';
    chars[1] = '5';
    let input: String = chars.into_iter().collect();
    let output = run_with_arg(&input);
    assert_eq!(output.status.code(), Some(1));
    let err = stderr_str(&output);
    // Should report which digit is duplicated
    assert!(
        err.contains('5'),
        "error should report the duplicate digit '5'. got: {err}"
    );
    // Should report location (row number)
    assert!(
        err.contains('0') || err.contains("row"),
        "error should report the location. got: {err}"
    );
}

#[test]
fn test_passes_valid_board() {
    // A valid puzzle should not produce a validation error
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(
        output.status.code(),
        Some(0),
        "valid board should pass validation and solve. stderr: {}",
        stderr_str(&output)
    );
    assert!(
        is_valid_sudoku(&stdout_str(&output)),
        "output should be a valid solved board"
    );
}

// ===========================================================================
// 6.3 Constraint Propagation
// ===========================================================================

#[test]
fn test_solves_easy_puzzle_without_backtracking() {
    // The easy puzzle should be solvable by constraint propagation alone.
    // We verify the output is correct; the "without backtracking" property
    // is an internal implementation detail, but a correct answer for this
    // known-easy puzzle validates that propagation works.
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout_str(&output),
        EASY_SOLUTION,
        "easy puzzle should be solved correctly (propagation-solvable)"
    );
}

// ===========================================================================
// 6.4 Backtracking
// ===========================================================================

#[test]
fn test_solves_hard_puzzle() {
    // Arto Inkala's hard puzzle requires backtracking
    let output = run_with_arg(HARD_PUZZLE);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should solve hard puzzle. stderr: {}",
        stderr_str(&output)
    );
    let solution = stdout_str(&output);
    assert!(
        is_valid_sudoku(&solution),
        "solution to hard puzzle must be a valid sudoku. got:\n{solution}"
    );
}

// ===========================================================================
// 6.5 Output
// ===========================================================================

#[test]
fn test_output_is_9_lines_of_9_digits() {
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(output.status.code(), Some(0));
    let out = stdout_str(&output);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 9, "output should have exactly 9 lines, got {}", lines.len());
    for (i, line) in lines.iter().enumerate() {
        assert_eq!(
            line.len(),
            9,
            "line {i} should have 9 characters, got {} ('{line}')",
            line.len()
        );
        assert!(
            line.chars().all(|c| c.is_ascii_digit() && c != '0'),
            "line {i} should contain only digits 1-9, got '{line}'"
        );
    }
}

#[test]
fn test_output_has_trailing_newline() {
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(output.status.code(), Some(0));
    let out = stdout_str(&output);
    assert!(
        out.ends_with('\n'),
        "output should end with a newline character"
    );
}

#[test]
fn test_output_is_valid_sudoku() {
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(output.status.code(), Some(0));
    let out = stdout_str(&output);
    assert!(
        is_valid_sudoku(&out),
        "output should be a valid complete sudoku board. got:\n{out}"
    );
}

// ===========================================================================
// 6.6 CLI
// ===========================================================================

#[test]
fn test_accepts_argument() {
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should accept puzzle as CLI argument"
    );
    assert_eq!(stdout_str(&output), EASY_SOLUTION);
}

#[test]
fn test_accepts_stdin() {
    let output = run_with_stdin(EASY_PUZZLE);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should accept puzzle from stdin. stderr: {}",
        stderr_str(&output)
    );
    assert_eq!(
        stdout_str(&output),
        EASY_SOLUTION,
        "stdin and arg should produce identical results"
    );
}

#[test]
fn test_exit_code_0_on_success() {
    let output = run_with_arg(EASY_PUZZLE);
    assert_eq!(
        output.status.code(),
        Some(0),
        "exit code should be 0 on successful solve"
    );
    assert_eq!(
        stdout_str(&output),
        EASY_SOLUTION,
        "successful solve should output the correct solution"
    );
}

#[test]
fn test_exit_code_1_on_invalid_input() {
    // Too short
    let output_short = run_with_arg("12345");
    assert_eq!(
        output_short.status.code(),
        Some(1),
        "exit code should be 1 on invalid input (too short)"
    );

    // Invalid character
    let mut chars: Vec<char> = "0".repeat(81).chars().collect();
    chars[0] = 'z';
    let bad_char: String = chars.into_iter().collect();
    let output_bad_char = run_with_arg(&bad_char);
    assert_eq!(
        output_bad_char.status.code(),
        Some(1),
        "exit code should be 1 on invalid character"
    );

    // Duplicate in row (validation error)
    let mut chars2: Vec<char> = "0".repeat(81).chars().collect();
    chars2[0] = '1';
    chars2[1] = '1';
    let dup: String = chars2.into_iter().collect();
    let output_dup = run_with_arg(&dup);
    assert_eq!(
        output_dup.status.code(),
        Some(1),
        "exit code should be 1 on validation error (duplicate)"
    );
}

#[test]
fn test_exit_code_2_on_unsolvable() {
    let output = run_with_arg(UNSOLVABLE_PUZZLE);
    assert_eq!(
        output.status.code(),
        Some(2),
        "exit code should be 2 on unsolvable puzzle. stderr: {}",
        stderr_str(&output)
    );
}

#[test]
fn test_errors_to_stderr_solution_to_stdout() {
    // On success: solution goes to stdout, stderr should be empty
    let success = run_with_arg(EASY_PUZZLE);
    assert_eq!(success.status.code(), Some(0));
    assert!(
        !stdout_str(&success).is_empty(),
        "solution should be on stdout"
    );
    assert!(
        stderr_str(&success).is_empty(),
        "stderr should be empty on success. got: {}",
        stderr_str(&success)
    );

    // On error: error goes to stderr, stdout should be empty
    let failure = run_with_arg("12345");
    assert_eq!(failure.status.code(), Some(1));
    assert!(
        stdout_str(&failure).is_empty(),
        "stdout should be empty on error. got: {}",
        stdout_str(&failure)
    );
    assert!(
        !stderr_str(&failure).is_empty(),
        "error message should be on stderr"
    );
}

// ===========================================================================
// 6.7 Edge Cases
// ===========================================================================

#[test]
fn test_already_solved_board() {
    let output = run_with_arg(SOLVED_BOARD);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should accept an already-solved board. stderr: {}",
        stderr_str(&output)
    );
    assert_eq!(
        stdout_str(&output),
        SOLVED_BOARD_OUTPUT,
        "already-solved board should be output unchanged"
    );
}

#[test]
fn test_board_with_one_blank() {
    // Take the solved board and blank out one cell (last digit: position 80)
    // The solved board ends with ...345286179
    // We blank the last digit (9 at position 80)
    let mut chars: Vec<char> = SOLVED_BOARD.chars().collect();
    chars[80] = '0';
    let one_blank: String = chars.into_iter().collect();
    let output = run_with_arg(&one_blank);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should solve board with one blank. stderr: {}",
        stderr_str(&output)
    );
    assert_eq!(
        stdout_str(&output),
        SOLVED_BOARD_OUTPUT,
        "board with one blank should produce the same completed board"
    );
}

#[test]
fn test_empty_board() {
    let empty = "0".repeat(81);
    let output = run_with_arg(&empty);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should solve an empty board. stderr: {}",
        stderr_str(&output)
    );
    let solution = stdout_str(&output);
    assert!(
        is_valid_sudoku(&solution),
        "solution to empty board must be a valid sudoku. got:\n{solution}"
    );
}

#[test]
fn test_minimum_clue_puzzle_17_givens() {
    // Verify the puzzle actually has exactly 17 givens
    let givens = MINIMUM_CLUE_PUZZLE
        .chars()
        .filter(|&c| c != '0' && c != '.')
        .count();
    assert_eq!(givens, 17, "test puzzle should have exactly 17 givens");

    let output = run_with_arg(MINIMUM_CLUE_PUZZLE);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should solve minimum-clue (17 givens) puzzle. stderr: {}",
        stderr_str(&output)
    );
    let solution = stdout_str(&output);
    assert!(
        is_valid_sudoku(&solution),
        "solution to 17-clue puzzle must be a valid sudoku. got:\n{solution}"
    );
}

// ===========================================================================
// 6.8 Integration Smoke Test
// ===========================================================================

#[test]
fn test_integration_smoke() {
    // --- Easy puzzle (solvable by propagation alone) ---
    let easy = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    let result = run_with_arg(easy);
    assert_eq!(
        result.status.code(),
        Some(0),
        "easy puzzle should exit 0"
    );
    assert_eq!(
        stdout_str(&result),
        "534678912\n672195348\n198342567\n859761423\n426853791\n713924856\n961537284\n287419635\n345286179\n",
        "easy puzzle solution mismatch"
    );

    // --- Invalid input (too short) ---
    let result = run_with_arg("12345");
    assert_eq!(
        result.status.code(),
        Some(1),
        "too-short input should exit 1"
    );
    let err = stderr_str(&result).to_lowercase();
    assert!(
        err.contains("length") || err.contains("81"),
        "error should mention length. got: {err}"
    );

    // --- Invalid board (duplicate in row) ---
    let bad = format!("11{}", "0".repeat(79));
    let result = run_with_arg(&bad);
    assert_eq!(
        result.status.code(),
        Some(1),
        "duplicate-in-row board should exit 1"
    );
    let err = stderr_str(&result).to_lowercase();
    assert!(
        err.contains("duplicate"),
        "error should mention 'duplicate'. got: {err}"
    );

    // --- Unsolvable puzzle ---
    let result = run_with_arg(UNSOLVABLE_PUZZLE);
    assert_eq!(
        result.status.code(),
        Some(2),
        "unsolvable puzzle should exit 2"
    );

    // --- Already solved board ---
    let result = run_with_arg(SOLVED_BOARD);
    assert_eq!(
        result.status.code(),
        Some(0),
        "already-solved board should exit 0"
    );
    assert_eq!(
        stdout_str(&result),
        "534678912\n672195348\n198342567\n859761423\n426853791\n713924856\n961537284\n287419635\n345286179\n",
        "already-solved board output mismatch"
    );
}
