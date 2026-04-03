//! Integration tests for the rubiks-solver CLI binary.
//!
//! These tests exercise the solver as a black-box CLI, verifying all
//! Definition of Done items from the NLSpec (sections 6.1 through 6.13).
//! They invoke the `rubiks-solver` binary via `std::process::Command`
//! and assert on exit codes, stdout, and stderr.

use std::process::Command;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

/// Superflip: all edges flipped in place, corners correct.
/// Known to require exactly 20 moves in HTM (half-turn metric).
const SUPERFLIP: &str = "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB";

/// R U R' U' applied to the solved cube (from NLSpec section 6.13).
const SCRAMBLE_R_U_RP_UP: &str = "LUUUUBUUBBRRFRRFRRURRFFFFFFDDDDDDDDRLLFLLLLLLDBBUBBUBB";

/// A known hard 20-move scramble: R U2 D' B D' R2 U' B2 L U2 D F2 R' U' D B2 L U' B2 R2.
const HARD_SCRAMBLE: &str = "FLDDUURFLFRBLRLRBFULUDFBLDBRRBFDUDUUDUUBLFBRLFBRDBRLFD";

/// Wrong sticker counts string (from NLSpec section 6.13): extra U, missing R.
const BAD_STICKER_COUNTS: &str = "UUUUUUUUURRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBUBBBBBBBB";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the path to the compiled binary under test.
fn solver_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rubiks-solver"))
}

/// Run the solver with the given facelet string as a CLI argument.
fn run_with_arg(input: &str) -> std::process::Output {
    solver_bin()
        .arg(input)
        .output()
        .expect("failed to execute rubiks-solver binary")
}

/// Run the solver with the given facelet string piped to stdin.
fn run_with_stdin(input: &str) -> std::process::Output {
    use std::io::Write;
    let mut child = solver_bin()
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn rubiks-solver binary");

    child
        .stdin
        .take()
        .expect("failed to open stdin")
        .write_all(input.as_bytes())
        .expect("failed to write to stdin");

    child.wait_with_output().expect("failed to wait on child")
}

fn stdout_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Parse a Singmaster move sequence (e.g. "R U R' U2 F'") into individual move tokens.
fn parse_moves(solution: &str) -> Vec<&str> {
    solution.trim().split_whitespace().collect()
}

/// Validate that every token in the solution is a legal Singmaster move.
fn is_valid_singmaster(solution: &str) -> bool {
    const VALID_MOVES: &[&str] = &[
        "U", "U'", "U2", "D", "D'", "D2", "R", "R'", "R2",
        "L", "L'", "L2", "F", "F'", "F2", "B", "B'", "B2",
    ];
    let moves = parse_moves(solution);
    !moves.is_empty() && moves.iter().all(|m| VALID_MOVES.contains(m))
}

/// Check that a solution has no consecutive moves on the same face
/// and that opposite-face pairs are in canonical order.
fn has_no_redundant_moves(solution: &str) -> bool {
    let moves = parse_moves(solution);
    for window in moves.windows(2) {
        let face_a = window[0].chars().next().unwrap();
        let face_b = window[1].chars().next().unwrap();
        if face_a == face_b {
            return false;
        }
        // Opposite faces: second should not precede first in canonical order
        if matches!(
            (face_a, face_b),
            ('D', 'U') | ('L', 'R') | ('B', 'F')
        ) {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Move simulator for solution verification
// ---------------------------------------------------------------------------
//
// We define each of the 6 clockwise face moves as a complete 54-element
// facelet permutation table, where perm[i] = j means new[i] = old[j].
// These tables are derived from the NLSpec cubie definitions (section 2.1)
// and the corner/edge cycle + orientation data (section 3.5).
//
// Counter-clockwise and double moves are computed from the CW permutation
// via inversion and composition respectively.

/// The 6 CW facelet permutations, derived from the NLSpec cubie-level
/// corner/edge cycles with orientation deltas applied to the three/two
/// facelets of each cubie.
///
/// Facelet numbering (Kociemba standard):
///          U face
///      0  1  2
///      3  4  5
///      6  7  8
///
///  L     F     R     B
/// 36 37 38  18 19 20   9 10 11  45 46 47
/// 39 40 41  21 22 23  12 13 14  48 49 50
/// 42 43 44  24 25 26  15 16 17  51 52 53
///
///          D face
///     27 28 29
///     30 31 32
///     33 34 35
#[rustfmt::skip]
const PERM_U: [usize; 54] = [
     6,  3,  0,  7,  4,  1,  8,  5,  2,
    18, 19, 20, 12, 13, 14, 15, 16, 17,
    36, 37, 38, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35,
    45, 46, 47, 39, 40, 41, 42, 43, 44,
     9, 10, 11, 48, 49, 50, 51, 52, 53,
];

#[rustfmt::skip]
const PERM_D: [usize; 54] = [
     0,  1,  2,  3,  4,  5,  6,  7,  8,
     9, 10, 11, 12, 13, 14, 51, 52, 53,
    18, 19, 20, 21, 22, 23, 15, 16, 17,
    29, 32, 35, 28, 31, 34, 27, 30, 33,
    36, 37, 38, 39, 40, 41, 24, 25, 26,
    45, 46, 47, 48, 49, 50, 42, 43, 44,
];

#[rustfmt::skip]
const PERM_R: [usize; 54] = [
     0,  1, 51,  3,  4, 48,  6,  7, 45,
    15, 12,  9, 16, 13, 10, 17, 14, 11,
    18, 19,  2, 21, 22,  5, 24, 25,  8,
    27, 28, 20, 30, 31, 23, 33, 34, 26,
    36, 37, 38, 39, 40, 41, 42, 43, 44,
    35, 46, 47, 32, 49, 50, 29, 52, 53,
];

#[rustfmt::skip]
const PERM_L: [usize; 54] = [
    18,  1,  2, 21,  4,  5, 24,  7,  8,
     9, 10, 11, 12, 13, 14, 15, 16, 17,
    33, 19, 20, 30, 22, 23, 27, 25, 26,
    47, 28, 29, 50, 31, 32, 53, 34, 35,
    38, 41, 44, 37, 40, 43, 36, 39, 42,
    45, 46,  6, 48, 49,  3, 51, 52,  0,
];

#[rustfmt::skip]
const PERM_F: [usize; 54] = [
     0,  1,  2,  3,  4,  5, 44, 41, 38,
     6, 10, 11,  7, 13, 14,  8, 16, 17,
    24, 21, 18, 25, 22, 19, 26, 23, 20,
    15, 12,  9, 30, 31, 32, 33, 34, 35,
    36, 37, 27, 39, 40, 28, 42, 43, 29,
    45, 46, 47, 48, 49, 50, 51, 52, 53,
];

#[rustfmt::skip]
const PERM_B: [usize; 54] = [
    11, 14, 17,  3,  4,  5,  6,  7,  8,
     9, 10, 35, 12, 13, 34, 15, 16, 33,
    18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 36, 39, 42,
     2, 37, 38,  1, 40, 41,  0, 43, 44,
    47, 50, 53, 46, 49, 52, 45, 48, 51,
];

fn apply_perm(facelets: &[u8; 54], perm: &[usize; 54]) -> [u8; 54] {
    let mut result = [0u8; 54];
    for i in 0..54 {
        result[i] = facelets[perm[i]];
    }
    result
}

fn invert_perm(perm: &[usize; 54]) -> [usize; 54] {
    let mut inv = [0usize; 54];
    for i in 0..54 {
        inv[perm[i]] = i;
    }
    inv
}

fn double_perm(perm: &[usize; 54]) -> [usize; 54] {
    let mut p2 = [0usize; 54];
    for i in 0..54 {
        p2[i] = perm[perm[i]];
    }
    p2
}

/// Apply a Singmaster move to a facelet array.
fn apply_move(facelets: &mut [u8; 54], mv: &str) {
    let u_ccw = invert_perm(&PERM_U);
    let u_double = double_perm(&PERM_U);
    let d_ccw = invert_perm(&PERM_D);
    let d_double = double_perm(&PERM_D);
    let r_ccw = invert_perm(&PERM_R);
    let r_double = double_perm(&PERM_R);
    let l_ccw = invert_perm(&PERM_L);
    let l_double = double_perm(&PERM_L);
    let f_ccw = invert_perm(&PERM_F);
    let f_double = double_perm(&PERM_F);
    let b_ccw = invert_perm(&PERM_B);
    let b_double = double_perm(&PERM_B);

    let p = match mv {
        "U" => &PERM_U,
        "U'" => &u_ccw,
        "U2" => &u_double,
        "D" => &PERM_D,
        "D'" => &d_ccw,
        "D2" => &d_double,
        "R" => &PERM_R,
        "R'" => &r_ccw,
        "R2" => &r_double,
        "L" => &PERM_L,
        "L'" => &l_ccw,
        "L2" => &l_double,
        "F" => &PERM_F,
        "F'" => &f_ccw,
        "F2" => &f_double,
        "B" => &PERM_B,
        "B'" => &b_ccw,
        "B2" => &b_double,
        other => panic!("unknown move: {other}"),
    };

    *facelets = apply_perm(facelets, p);
}

/// Parse a facelet string into a [u8; 54] array (U=0..B=5).
fn parse_facelets(s: &str) -> [u8; 54] {
    let mut arr = [0u8; 54];
    for (i, ch) in s.chars().enumerate() {
        arr[i] = match ch.to_ascii_uppercase() {
            'U' => 0,
            'R' => 1,
            'F' => 2,
            'D' => 3,
            'L' => 4,
            'B' => 5,
            _ => panic!("bad facelet char: {ch}"),
        };
    }
    arr
}

fn facelets_to_string(f: &[u8; 54]) -> String {
    f.iter()
        .map(|&c| match c {
            0 => 'U',
            1 => 'R',
            2 => 'F',
            3 => 'D',
            4 => 'L',
            5 => 'B',
            _ => '?',
        })
        .collect()
}

// ===========================================================================
// 6.1 Input Parsing
// ===========================================================================

#[test]
fn t01_accepts_solved_cube() {
    let output = run_with_arg(SOLVED);
    assert_eq!(output.status.code(), Some(0), "solved cube should exit 0");
}

#[test]
fn t02_handles_lowercase_input() {
    let lower = SOLVED.to_lowercase();
    let output = run_with_arg(&lower);
    assert_eq!(output.status.code(), Some(0), "lowercase input should be accepted");
}

#[test]
fn t03_handles_mixed_case_input() {
    let mixed = "UuUuUuUuUrRrRrRrRrFfFfFfFfFdDdDdDdDlLlLlLlLlBbBbBbBbB";
    let output = run_with_arg(mixed);
    assert_eq!(output.status.code(), Some(0), "mixed case input should be accepted");
}

#[test]
fn t04_rejects_too_short_input() {
    let output = run_with_arg("UUUUUUUUU"); // 9 chars
    assert_eq!(output.status.code(), Some(1), "short input should exit 1");
    let err = stderr_str(&output);
    assert!(
        err.to_lowercase().contains("length") || err.contains("54") || err.contains("9"),
        "error should mention length, got: {err}"
    );
}

#[test]
fn t05_rejects_too_long_input() {
    let long_input = format!("{}U", SOLVED); // 55 chars
    let output = run_with_arg(&long_input);
    assert_eq!(output.status.code(), Some(1), "long input should exit 1");
}

#[test]
fn t06_rejects_invalid_character() {
    let bad = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBX";
    let output = run_with_arg(bad);
    assert_eq!(output.status.code(), Some(1), "invalid char should exit 1");
    let err = stderr_str(&output);
    assert!(
        err.to_lowercase().contains("character") || err.contains("'X'") || err.contains("'x'"),
        "error should mention invalid character, got: {err}"
    );
}

#[test]
fn t07_reports_invalid_character_position() {
    // 'X' at position 53 (0-indexed)
    let bad = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBX";
    let output = run_with_arg(bad);
    let err = stderr_str(&output);
    assert!(
        err.contains("53") || err.contains("position"),
        "error should report the position of the invalid character, got: {err}"
    );
}

#[test]
fn t08_strips_whitespace() {
    let padded = format!("  {}  ", SOLVED);
    let output = run_with_arg(&padded);
    assert_eq!(
        output.status.code(),
        Some(0),
        "whitespace-padded input should be accepted"
    );
}

// ===========================================================================
// 6.4 Validation
// ===========================================================================

#[test]
fn t09_rejects_wrong_sticker_count() {
    // 18 U's, 0 R's
    let bad = "UUUUUUUUUUUUUUUUUUFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = run_with_arg(bad);
    assert_eq!(output.status.code(), Some(1), "wrong sticker count should exit 1");
}

#[test]
fn t10_rejects_wrong_sticker_count_reports_color() {
    let output = run_with_arg(BAD_STICKER_COUNTS);
    assert_eq!(output.status.code(), Some(1));
    let err = stderr_str(&output);
    assert!(
        err.to_lowercase().contains("count")
            || err.to_lowercase().contains("sticker")
            || err.contains("9"),
        "error should mention sticker count issue, got: {err}"
    );
}

#[test]
fn t11_rejects_wrong_center() {
    // Swap centers of U (pos 4) and R (pos 13)
    let mut chars: Vec<char> = SOLVED.chars().collect();
    chars[4] = 'R';
    chars[13] = 'U';
    let bad: String = chars.into_iter().collect();
    let output = run_with_arg(&bad);
    assert_eq!(output.status.code(), Some(1), "wrong center should exit 1");
    let err = stderr_str(&output);
    assert!(
        err.to_lowercase().contains("center"),
        "error should mention center, got: {err}"
    );
}

#[test]
fn t12_rejects_corner_orientation_parity() {
    // Twist URF corner CW by 1: rotate facelets 8, 9, 20 cyclically.
    // URF facelets: indices 8(U), 9(R), 20(F). On solved: U, R, F.
    // After one CW twist: 8=R, 9=F, 20=U. Orientation sum mod 3 != 0.
    let bad = "UUUUUUUURFRRRRRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = run_with_arg(bad);
    assert_eq!(
        output.status.code(),
        Some(1),
        "corner orientation parity violation should exit 1"
    );
}

#[test]
fn t13_rejects_edge_orientation_parity() {
    // Flip a single edge (UR: indices 5, 10). Swap U5 and R10.
    // Edge orientation sum = 1 (odd), violating parity.
    let bad = "UUUUURUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = run_with_arg(bad);
    assert_eq!(
        output.status.code(),
        Some(1),
        "edge orientation parity violation should exit 1"
    );
}

#[test]
fn t14_rejects_permutation_parity_mismatch() {
    // Swap two edges (UR and UF) without swapping corners.
    // UR: (5, 10), UF: (7, 19). Swap the entire edge pairs.
    // Creates odd edge permutation with even corner permutation.
    let bad = "UUUUUUUUURFRRRRRRRFRFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = run_with_arg(bad);
    assert_eq!(
        output.status.code(),
        Some(1),
        "permutation parity mismatch should exit 1"
    );
}

#[test]
fn t15_rejects_duplicate_cubies() {
    // Create a state where the same corner appears in two slots.
    // Put URF colors at UFL position: index 6=U(ok), 18=R(was F), 44=F(was L).
    // Fix counts: change one R to L elsewhere (index 14: R->L).
    let bad = "UUUUUUUURURRRLRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = run_with_arg(bad);
    assert_eq!(
        output.status.code(),
        Some(1),
        "duplicate cubie should exit 1"
    );
}

#[test]
fn t16_accepts_solved_cube_validation() {
    let output = run_with_arg(SOLVED);
    assert_eq!(output.status.code(), Some(0));
}

// ===========================================================================
// 6.7 Phase 1 / 6.9 Full Solve -- solution correctness
// ===========================================================================

#[test]
fn t17_solves_known_4_move_scramble() {
    let output = run_with_arg(SCRAMBLE_R_U_RP_UP);
    assert_eq!(output.status.code(), Some(0), "should solve the scramble");
    let solution = stdout_str(&output);
    let moves = parse_moves(&solution);
    assert!(
        moves.len() <= 25,
        "solution should be at most 25 moves, got {}",
        moves.len()
    );
    assert!(
        is_valid_singmaster(&solution),
        "solution should be valid Singmaster notation, got: {solution}"
    );
}

#[test]
fn t18_solves_hard_20_move_scramble() {
    let output = run_with_arg(HARD_SCRAMBLE);
    assert_eq!(output.status.code(), Some(0), "should solve the hard scramble");
    let solution = stdout_str(&output);
    let moves = parse_moves(&solution);
    assert!(
        moves.len() <= 25,
        "solution should be at most 25 moves, got {}",
        moves.len()
    );
}

#[test]
fn t19_solves_superflip() {
    let output = run_with_arg(SUPERFLIP);
    assert_eq!(output.status.code(), Some(0), "should solve the superflip");
    let solution = stdout_str(&output);
    let moves = parse_moves(&solution);
    assert!(
        moves.len() <= 25,
        "superflip solution should be at most 25 moves, got {}",
        moves.len()
    );
    assert!(
        is_valid_singmaster(&solution),
        "solution should be valid Singmaster notation, got: {solution}"
    );
}

// ===========================================================================
// 6.8 Phase 2 -- solution move restrictions
// ===========================================================================

#[test]
fn t20_solution_uses_valid_singmaster_notation() {
    let output = run_with_arg(SCRAMBLE_R_U_RP_UP);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    assert!(
        is_valid_singmaster(&solution),
        "all moves must be valid Singmaster, got: {solution}"
    );
}

// ===========================================================================
// 6.7 Move redundancy pruning
// ===========================================================================

#[test]
fn t21_no_same_face_consecutive_moves() {
    let output = run_with_arg(HARD_SCRAMBLE);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    assert!(
        has_no_redundant_moves(&solution),
        "solution should have no same-face consecutive or opposite-face wrong-order moves, got: {solution}"
    );
}

// ===========================================================================
// 6.10 Solution Quality
// ===========================================================================

#[test]
fn t22_no_solution_exceeds_25_moves() {
    let scrambles = [SCRAMBLE_R_U_RP_UP, HARD_SCRAMBLE, SUPERFLIP];
    for scramble in &scrambles {
        let output = run_with_arg(scramble);
        assert_eq!(output.status.code(), Some(0));
        let solution = stdout_str(&output);
        let count = parse_moves(&solution).len();
        assert!(
            count <= 25,
            "solution for scramble has {count} moves, exceeding the 25-move limit"
        );
    }
}

// ===========================================================================
// 6.11 CLI
// ===========================================================================

#[test]
fn t23_accepts_input_from_stdin() {
    let output = run_with_stdin(SOLVED);
    assert_eq!(
        output.status.code(),
        Some(0),
        "should accept input from stdin"
    );
    let out = stdout_str(&output);
    assert!(
        out.trim().to_lowercase().contains("already solved"),
        "stdin solved cube should output 'Already solved', got: {out}"
    );
}

#[test]
fn t24_exit_0_on_success() {
    let output = run_with_arg(SCRAMBLE_R_U_RP_UP);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn t25_exit_1_on_invalid_input() {
    let output = run_with_arg("INVALID");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn t26_exit_1_on_validation_error() {
    let bad = "UUUUUUUUUUUUUUUUUUFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = run_with_arg(bad);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn t27_solution_printed_to_stdout() {
    let output = run_with_arg(SCRAMBLE_R_U_RP_UP);
    assert_eq!(output.status.code(), Some(0));
    let out = stdout_str(&output);
    assert!(!out.trim().is_empty(), "solution should be printed to stdout");
    assert!(
        is_valid_singmaster(&out),
        "stdout should contain valid Singmaster moves, got: {out}"
    );
}

#[test]
fn t28_errors_printed_to_stderr() {
    let output = run_with_arg("UUUUUUUUU"); // too short
    assert_eq!(output.status.code(), Some(1));
    let err = stderr_str(&output);
    assert!(!err.trim().is_empty(), "error messages should go to stderr");
    let out = stdout_str(&output);
    assert!(out.trim().is_empty(), "stdout should be empty on error, got: {out}");
}

#[test]
fn t29_already_solved_message() {
    let output = run_with_arg(SOLVED);
    assert_eq!(output.status.code(), Some(0));
    let out = stdout_str(&output);
    assert!(
        out.trim().to_lowercase().contains("already solved"),
        "solved cube should output 'Already solved', got: {out}"
    );
}

// ===========================================================================
// 6.12 Edge Cases
// ===========================================================================

#[test]
fn t30_solved_cube_edge_case() {
    let output = run_with_arg(SOLVED);
    assert_eq!(output.status.code(), Some(0));
    let out = stdout_str(&output);
    assert_eq!(
        out.trim(),
        "Already solved",
        "solved cube stdout should be exactly 'Already solved', got: {out}"
    );
}

#[test]
fn t31_superflip_edge_case() {
    let output = run_with_arg(SUPERFLIP);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    assert!(
        is_valid_singmaster(&solution),
        "superflip solution must be valid Singmaster, got: {solution}"
    );
}

// ===========================================================================
// 6.9 Multiple solves produce valid solutions
// ===========================================================================

#[test]
fn t32_multiple_solves_produce_valid_solutions() {
    for _ in 0..2 {
        let output = run_with_arg(HARD_SCRAMBLE);
        assert_eq!(output.status.code(), Some(0));
        let solution = stdout_str(&output);
        assert!(
            is_valid_singmaster(&solution),
            "each solve should produce valid moves, got: {solution}"
        );
        let count = parse_moves(&solution).len();
        assert!(count <= 25, "each solve should be at most 25 moves");
    }
}

// ===========================================================================
// 6.13 Integration Smoke Test
// ===========================================================================

#[test]
fn t33_smoke_test_solved() {
    let output = run_with_arg(SOLVED);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_str(&output).trim(), "Already solved");
}

#[test]
fn t34_smoke_test_too_short() {
    let output = run_with_arg("UUUUU");
    assert_eq!(output.status.code(), Some(1));
    let err = stderr_str(&output);
    assert!(
        err.to_lowercase().contains("length") || err.contains("54") || err.contains("5"),
        "error should mention length, got: {err}"
    );
}

#[test]
fn t35_smoke_test_invalid_char() {
    // 53 chars with X at position 0 (from spec 6.13)
    let bad = "XUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBB";
    let output = run_with_arg(bad);
    assert_eq!(output.status.code(), Some(1));
    let err = stderr_str(&output);
    assert!(
        err.to_lowercase().contains("character") || err.contains("'X'") || err.contains("'x'"),
        "error should mention character, got: {err}"
    );
}

#[test]
fn t36_smoke_test_wrong_sticker_count() {
    let output = run_with_arg(BAD_STICKER_COUNTS);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn t37_smoke_test_known_scramble() {
    let output = run_with_arg(SCRAMBLE_R_U_RP_UP);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    let count = parse_moves(&solution).len();
    assert!(
        count <= 25,
        "R U R' U' scramble solution should be at most 25 moves, got {count}"
    );
}

#[test]
fn t38_smoke_test_hard_scramble() {
    let output = run_with_arg(HARD_SCRAMBLE);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    let count = parse_moves(&solution).len();
    assert!(
        count <= 25,
        "hard scramble solution should be at most 25 moves, got {count}"
    );
}

// ===========================================================================
// 6.9 Applying solution moves produces solved state
// ===========================================================================

#[test]
fn t39_applying_solution_to_scramble_yields_solved() {
    let output = run_with_arg(SCRAMBLE_R_U_RP_UP);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    let moves = parse_moves(&solution);

    let mut cube = parse_facelets(SCRAMBLE_R_U_RP_UP);
    for mv in &moves {
        apply_move(&mut cube, mv);
    }
    let result = facelets_to_string(&cube);
    assert_eq!(
        result, SOLVED,
        "applying solution to scrambled cube should yield solved state"
    );
}

#[test]
fn t40_applying_solution_to_hard_scramble_yields_solved() {
    let output = run_with_arg(HARD_SCRAMBLE);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    let moves = parse_moves(&solution);

    let mut cube = parse_facelets(HARD_SCRAMBLE);
    for mv in &moves {
        apply_move(&mut cube, mv);
    }
    let result = facelets_to_string(&cube);
    assert_eq!(
        result, SOLVED,
        "applying solution to hard scramble should yield solved state"
    );
}

#[test]
fn t41_applying_solution_to_superflip_yields_solved() {
    let output = run_with_arg(SUPERFLIP);
    assert_eq!(output.status.code(), Some(0));
    let solution = stdout_str(&output);
    let moves = parse_moves(&solution);

    let mut cube = parse_facelets(SUPERFLIP);
    for mv in &moves {
        apply_move(&mut cube, mv);
    }
    let result = facelets_to_string(&cube);
    assert_eq!(
        result, SOLVED,
        "applying superflip solution should yield solved state"
    );
}

// ===========================================================================
// 6.12 One-move scramble
// ===========================================================================

#[test]
fn t42_one_move_scramble_solution_restores_solved() {
    // Apply R to solved, get the facelet string, solve it, verify solution restores solved.
    let mut cube = parse_facelets(SOLVED);
    apply_move(&mut cube, "R");
    let r_scrambled = facelets_to_string(&cube);

    let output = run_with_arg(&r_scrambled);
    assert_eq!(output.status.code(), Some(0), "one-move scramble should solve");
    let solution = stdout_str(&output);
    let moves = parse_moves(&solution);

    let mut verify = parse_facelets(&r_scrambled);
    for mv in &moves {
        apply_move(&mut verify, mv);
    }
    let result = facelets_to_string(&verify);
    assert_eq!(
        result, SOLVED,
        "applying solution to one-move scramble should yield solved"
    );
}

// ===========================================================================
// Additional: exit code 2 for unsolvable (6.11)
// ===========================================================================

#[test]
fn t43_validation_errors_use_exit_code_1() {
    // Corner orientation parity violation should be exit 1 (validation error)
    let bad = "UUUUUUUURFRRRRRRRRFFUFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let output = run_with_arg(bad);
    assert_ne!(output.status.code(), Some(0), "parity-violated cube should not exit 0");
    assert_eq!(
        output.status.code(),
        Some(1),
        "validation errors should use exit code 1"
    );
}

// ===========================================================================
// 6.11 Solution format: space-separated Singmaster notation
// ===========================================================================

#[test]
fn t44_solution_is_space_separated_singmaster() {
    let output = run_with_arg(HARD_SCRAMBLE);
    assert_eq!(output.status.code(), Some(0));
    let out = stdout_str(&output);
    let trimmed = out.trim();
    assert!(
        !trimmed.contains('\n'),
        "solution should be a single line, got: {trimmed}"
    );
    let valid_moves = [
        "U", "U'", "U2", "D", "D'", "D2", "R", "R'", "R2",
        "L", "L'", "L2", "F", "F'", "F2", "B", "B'", "B2",
    ];
    for token in trimmed.split_whitespace() {
        assert!(
            valid_moves.contains(&token),
            "unexpected token in solution: '{token}', full solution: {trimmed}"
        );
    }
}

// ===========================================================================
// Move simulator self-tests (verify our test infrastructure is correct)
// ===========================================================================

#[test]
fn t45_move_simulator_r_then_r_prime_is_identity() {
    let mut cube = parse_facelets(SOLVED);
    apply_move(&mut cube, "R");
    apply_move(&mut cube, "R'");
    assert_eq!(
        facelets_to_string(&cube),
        SOLVED,
        "R then R' should yield identity"
    );
}

#[test]
fn t46_move_simulator_all_moves_self_inverse() {
    // For each face: CW then CCW = identity, and 4x CW = identity
    let face_moves = ["U", "D", "R", "L", "F", "B"];
    for mv in &face_moves {
        // CW then CCW
        let mut cube = parse_facelets(SOLVED);
        apply_move(&mut cube, mv);
        let ccw = format!("{mv}'");
        apply_move(&mut cube, &ccw);
        assert_eq!(
            facelets_to_string(&cube),
            SOLVED,
            "{mv} then {mv}' should yield identity"
        );

        // 4x CW = identity
        let mut cube = parse_facelets(SOLVED);
        for _ in 0..4 {
            apply_move(&mut cube, mv);
        }
        assert_eq!(
            facelets_to_string(&cube),
            SOLVED,
            "4x {mv} should yield identity"
        );
    }
}
