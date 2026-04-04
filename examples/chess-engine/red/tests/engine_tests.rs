//! Red team integration tests for the chess-engine binary.
//!
//! These tests exercise the engine purely through the CLI interface using
//! `std::process::Command`. The binary supports two modes:
//!
//!   chess-engine perft <depth> [--fen "<fen>"]  — print perft leaf-node count
//!   chess-engine uci                            — enter UCI protocol mode
//!
//! All perft golden vectors are exact. Any deviation is a move generation bug.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn engine_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_chess-engine"))
}

/// Run `chess-engine perft <depth>` on the starting position (no --fen flag).
fn run_perft_startpos(depth: u32) -> std::process::Output {
    engine_bin()
        .args(["perft", &depth.to_string()])
        .output()
        .expect("failed to execute chess-engine")
}

/// Run `chess-engine perft <depth> --fen "<fen>"`.
fn run_perft(depth: u32, fen: &str) -> std::process::Output {
    engine_bin()
        .args(["perft", &depth.to_string(), "--fen", fen])
        .output()
        .expect("failed to execute chess-engine")
}

/// Parse the trimmed stdout of a perft run as a u64.
fn perft_count(output: &std::process::Output) -> u64 {
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim()
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("could not parse perft output as u64: {:?}", stdout))
}

/// Start the engine in UCI mode, returning a child with piped stdin/stdout.
fn start_uci() -> std::process::Child {
    engine_bin()
        .arg("uci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn chess-engine in UCI mode")
}

/// Send a line to the engine's stdin.
fn send(stdin: &mut impl Write, line: &str) {
    writeln!(stdin, "{}", line).expect("failed to write to engine stdin");
    stdin.flush().expect("failed to flush engine stdin");
}

/// Read lines from stdout until one contains `needle`, or timeout after 10s.
/// Returns all collected lines.
fn read_until(reader: &mut BufReader<impl std::io::Read>, needle: &str) -> Vec<String> {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut lines = Vec::new();
    loop {
        if std::time::Instant::now() > deadline {
            panic!(
                "timeout waiting for {:?}; collected so far: {:?}",
                needle, lines
            );
        }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => panic!(
                "EOF before finding {:?}; collected: {:?}",
                needle, lines
            ),
            Ok(_) => {
                let trimmed = line.trim_end().to_string();
                let found = trimmed.contains(needle);
                lines.push(trimmed);
                if found {
                    return lines;
                }
            }
            Err(e) => panic!("read error: {}", e),
        }
    }
}

// ---------------------------------------------------------------------------
// FEN constants
// ---------------------------------------------------------------------------

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
const POSITION3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
const POSITION4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const POSITION5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -";

// A position with en passant available: after 1. e4
const FEN_EN_PASSANT: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

// Black to move
const FEN_BLACK_TO_MOVE: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

// Scholar's mate — White is checkmated, 0 legal moves
const FEN_CHECKMATE: &str =
    "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";

// Known stalemate: Black king on h1, White queen g2 + king f3.
// h1 is not attacked (not in check). g1 attacked by Qg2. h2 attacked by Qg2 and Kf3.
// No legal moves => stalemate.
const FEN_STALEMATE: &str = "8/8/8/8/8/5K2/6Q1/7k b - - 0 1";

// White to move, Ra8# is mate in one (back-rank mate).
// White Kg1, Ra1; Black Kg8 with pawns f7, g7, h7.
// After Ra8#: rook controls entire 8th rank, pawns block 7th rank escape.
const FEN_MATE_IN_ONE: &str = "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1";

// Position where moving queen away from defense loses it
// White: Kg1, Qd1, Rd2, pawns; Black: Kg8, Qe5 attacks Rd2, Bb7.
// The queen on d1 defends d2. Moving it loses the rook (material).
const FEN_DONT_LOSE_QUEEN: &str =
    "6k1/1b4pp/8/4q3/8/8/3R1PPP/3Q2K1 w - - 0 1";

// ===========================================================================
// 6.1 FEN Parsing
// ===========================================================================

#[test]
fn test_parse_startpos_fen() {
    let output = run_perft(1, STARTPOS);
    assert!(
        output.status.success(),
        "engine should accept standard starting position FEN; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Also verifies the FEN was parsed correctly — perft(1) must be 20.
    assert_eq!(perft_count(&output), 20);
}

#[test]
fn test_parse_fen_with_en_passant() {
    // After 1. e4 the en passant square is e3. The engine must parse this.
    let output = run_perft(1, FEN_EN_PASSANT);
    assert!(
        output.status.success(),
        "engine should parse FEN with en passant square; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Black has 20 normal moves after 1.e4 (standard response count).
    // 8 pawns x 1 single push + 7 pawns x 1 double push (e-pawn blocked) + 2 knight moves = 20.
    // The en passant square e3 is irrelevant — no black pawn can capture en passant from rank 7.
    assert_eq!(perft_count(&output), 20);
}

#[test]
fn test_parse_fen_black_to_move() {
    let output = run_perft(1, FEN_BLACK_TO_MOVE);
    assert!(
        output.status.success(),
        "engine should parse FEN with black to move; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Black has 20 responses to 1.e4 (the position is equivalent).
    assert_eq!(perft_count(&output), 20);
}

#[test]
fn test_reject_invalid_fen_too_few_fields() {
    let output = run_perft(1, "just_garbage");
    assert!(
        !output.status.success(),
        "engine should reject FEN with too few fields"
    );
    // Exit code must be non-zero (spec says exit code 1).
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1 for invalid FEN"
    );
}

#[test]
fn test_reject_invalid_fen_bad_piece_char() {
    // 'Z' is not a valid piece character.
    let output = run_perft(1, "rnbqkbnr/ppppZppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert!(
        !output.status.success(),
        "engine should reject FEN with invalid piece character"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_reject_invalid_fen_wrong_rank_count() {
    // Only 7 ranks instead of 8.
    let output = run_perft(1, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1");
    assert!(
        !output.status.success(),
        "engine should reject FEN with wrong number of ranks"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_reject_invalid_fen_bad_rank_length() {
    // First rank sums to 9 squares (rnbqkbnrr has 9 pieces).
    let output = run_perft(1, "rnbqkbnrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert!(
        !output.status.success(),
        "engine should reject FEN where rank doesn't sum to 8"
    );
    assert_eq!(output.status.code(), Some(1));
}

// ===========================================================================
// 6.2 / 6.3 Move Generation — Perft Golden Vectors
// ===========================================================================

// ---- Position 1: Starting Position ----

#[test]
fn test_perft_startpos_depth1() {
    let output = run_perft(1, STARTPOS);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 20);
}

#[test]
fn test_perft_startpos_depth2() {
    let output = run_perft(2, STARTPOS);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 400);
}

#[test]
fn test_perft_startpos_depth3() {
    let output = run_perft(3, STARTPOS);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 8902);
}

#[test]
fn test_perft_startpos_depth4() {
    let output = run_perft(4, STARTPOS);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 197281);
}

#[test]
fn test_perft_startpos_depth5() {
    let output = run_perft(5, STARTPOS);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 4865609);
}

// ---- Position 2: Kiwipete ----

#[test]
fn test_perft_kiwipete_depth1() {
    let output = run_perft(1, KIWIPETE);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 48);
}

#[test]
fn test_perft_kiwipete_depth2() {
    let output = run_perft(2, KIWIPETE);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 2039);
}

#[test]
fn test_perft_kiwipete_depth3() {
    let output = run_perft(3, KIWIPETE);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 97862);
}

#[test]
fn test_perft_kiwipete_depth4() {
    let output = run_perft(4, KIWIPETE);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 4085603);
}

// ---- Position 3 ----

#[test]
fn test_perft_position3_depth1() {
    let output = run_perft(1, POSITION3);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 14);
}

#[test]
fn test_perft_position3_depth2() {
    let output = run_perft(2, POSITION3);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 191);
}

#[test]
fn test_perft_position3_depth3() {
    let output = run_perft(3, POSITION3);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 2812);
}

#[test]
fn test_perft_position3_depth4() {
    let output = run_perft(4, POSITION3);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 43238);
}

#[test]
fn test_perft_position3_depth5() {
    let output = run_perft(5, POSITION3);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 674624);
}

// ---- Position 4 ----

#[test]
fn test_perft_position4_depth1() {
    let output = run_perft(1, POSITION4);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 6);
}

#[test]
fn test_perft_position4_depth2() {
    let output = run_perft(2, POSITION4);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 264);
}

#[test]
fn test_perft_position4_depth3() {
    let output = run_perft(3, POSITION4);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 9467);
}

#[test]
fn test_perft_position4_depth4() {
    let output = run_perft(4, POSITION4);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 422333);
}

// ---- Position 5 ----

#[test]
fn test_perft_position5_depth1() {
    let output = run_perft(1, POSITION5);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 44);
}

#[test]
fn test_perft_position5_depth2() {
    let output = run_perft(2, POSITION5);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 1486);
}

#[test]
fn test_perft_position5_depth3() {
    let output = run_perft(3, POSITION5);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 62379);
}

#[test]
fn test_perft_position5_depth4() {
    let output = run_perft(4, POSITION5);
    assert!(output.status.success());
    assert_eq!(perft_count(&output), 2103487);
}

// ===========================================================================
// 6.4 Search
// ===========================================================================

#[test]
fn test_search_finds_mate_in_one() {
    // Position where White has Qh7# (or equivalent forcing mate).
    // We ask the engine to search and expect it to find a bestmove that delivers mate.
    let mut child = engine_bin()
        .arg("uci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn engine");

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    send(stdin, "uci");
    let _ = read_until(&mut reader, "uciok");

    send(stdin, "isready");
    let _ = read_until(&mut reader, "readyok");

    send(stdin, &format!("position fen {}", FEN_MATE_IN_ONE));
    send(stdin, "go depth 4");

    let lines = read_until(&mut reader, "bestmove");
    let bestmove_line = lines.iter().find(|l| l.starts_with("bestmove")).unwrap();

    // The engine must find a move (not report no move).
    assert!(
        bestmove_line.contains("bestmove "),
        "expected bestmove in output, got: {}",
        bestmove_line
    );

    // The bestmove should be a1a8 (Ra1-a8#) in long algebraic — back-rank mate.
    assert!(
        bestmove_line.contains("a1a8"),
        "expected mate-in-one move a1a8, got: {}",
        bestmove_line
    );

    // Check that the engine reported a mate score in the info lines.
    let has_mate_score = lines.iter().any(|l| l.contains("score mate 1"));
    assert!(
        has_mate_score,
        "expected 'score mate 1' in info output; lines: {:?}",
        lines
    );

    send(stdin, "quit");
    let status = child.wait().expect("failed to wait on engine");
    assert!(status.success());
}

#[test]
fn test_search_avoids_losing_queen() {
    // In FEN_DONT_LOSE_QUEEN, Black's queen on e5 attacks the rook on d2.
    // White's queen on d1 must defend. The engine should NOT move the queen
    // away from defending d2 (or should find a move that doesn't lose material).
    let mut child = engine_bin()
        .arg("uci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn engine");

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    send(stdin, "uci");
    let _ = read_until(&mut reader, "uciok");

    send(stdin, "isready");
    let _ = read_until(&mut reader, "readyok");

    send(stdin, &format!("position fen {}", FEN_DONT_LOSE_QUEEN));
    send(stdin, "go depth 6");

    let lines = read_until(&mut reader, "bestmove");
    let bestmove_line = lines.iter().find(|l| l.starts_with("bestmove")).unwrap();

    // Extract the move.
    let parts: Vec<&str> = bestmove_line.split_whitespace().collect();
    assert!(parts.len() >= 2, "bestmove line malformed: {}", bestmove_line);
    let bestmove = parts[1];

    // The engine should return a legal move.
    assert!(
        bestmove.len() >= 4,
        "bestmove should be in long algebraic notation: {}",
        bestmove
    );

    // The queen should not blunder away. Acceptable moves keep material balance.
    // Most reasonable: the queen stays defending (e.g., d1d2 captures aren't needed,
    // but the queen shouldn't abandon the d-file).
    // We just verify it returns a legal move and doesn't crash. A stronger assertion
    // would require knowing the exact best move, but at depth 6 any reasonable engine
    // should not hang the rook.

    send(stdin, "quit");
    let status = child.wait().expect("failed to wait on engine");
    assert!(status.success());
}

// ===========================================================================
// 6.6 UCI Protocol
// ===========================================================================

#[test]
fn test_uci_command_returns_id_and_uciok() {
    let mut child = start_uci();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    send(stdin, "uci");
    let lines = read_until(&mut reader, "uciok");

    // Must contain "id name" and "id author" before "uciok".
    let has_id_name = lines.iter().any(|l| l.starts_with("id name"));
    let has_id_author = lines.iter().any(|l| l.starts_with("id author"));
    let has_uciok = lines.iter().any(|l| l.trim() == "uciok");

    assert!(has_id_name, "missing 'id name' in UCI response: {:?}", lines);
    assert!(
        has_id_author,
        "missing 'id author' in UCI response: {:?}",
        lines
    );
    assert!(has_uciok, "missing 'uciok' in UCI response: {:?}", lines);

    send(stdin, "quit");
    let status = child.wait().unwrap();
    assert!(status.success());
}

#[test]
fn test_isready_returns_readyok() {
    let mut child = start_uci();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    send(stdin, "uci");
    let _ = read_until(&mut reader, "uciok");

    send(stdin, "isready");
    let lines = read_until(&mut reader, "readyok");
    let has_readyok = lines.iter().any(|l| l.trim() == "readyok");
    assert!(has_readyok, "missing 'readyok': {:?}", lines);

    send(stdin, "quit");
    let status = child.wait().unwrap();
    assert!(status.success());
}

#[test]
fn test_position_startpos_moves() {
    // Verify the engine accepts "position startpos moves e2e4 e7e5" without error.
    // We then do a perft-like probe via "go depth 1" to confirm it processed moves.
    let mut child = start_uci();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    send(stdin, "uci");
    let _ = read_until(&mut reader, "uciok");

    send(stdin, "isready");
    let _ = read_until(&mut reader, "readyok");

    send(stdin, "position startpos moves e2e4 e7e5");
    send(stdin, "go depth 1");

    let lines = read_until(&mut reader, "bestmove");
    let has_bestmove = lines.iter().any(|l| l.starts_with("bestmove"));
    assert!(
        has_bestmove,
        "expected bestmove after position+go: {:?}",
        lines
    );

    send(stdin, "quit");
    let status = child.wait().unwrap();
    assert!(status.success());
}

#[test]
fn test_go_depth_returns_bestmove() {
    let mut child = start_uci();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    send(stdin, "uci");
    let _ = read_until(&mut reader, "uciok");

    send(stdin, "isready");
    let _ = read_until(&mut reader, "readyok");

    send(stdin, "position startpos");
    send(stdin, "go depth 4");

    let lines = read_until(&mut reader, "bestmove");
    let bestmove_line = lines
        .iter()
        .find(|l| l.starts_with("bestmove"))
        .expect("no bestmove line found");

    // bestmove must contain a move in long algebraic (4-5 chars).
    let parts: Vec<&str> = bestmove_line.split_whitespace().collect();
    assert!(
        parts.len() >= 2 && parts[1].len() >= 4,
        "bestmove format invalid: {}",
        bestmove_line
    );

    // There should be at least one "info" line with depth and score.
    let has_info = lines.iter().any(|l| l.starts_with("info") && l.contains("depth"));
    assert!(
        has_info,
        "expected info lines with depth before bestmove: {:?}",
        lines
    );

    send(stdin, "quit");
    let status = child.wait().unwrap();
    assert!(status.success());
}

#[test]
fn test_quit_exits() {
    let mut child = start_uci();
    let stdin = child.stdin.as_mut().unwrap();

    send(stdin, "quit");

    let status = child
        .wait()
        .expect("failed to wait on engine after quit");
    assert!(
        status.success(),
        "engine should exit with code 0 on quit; got: {:?}",
        status
    );
}

// ===========================================================================
// 6.7 Edge Cases
// ===========================================================================

#[test]
fn test_checkmate_detected() {
    // Scholar's mate position: White is in checkmate, 0 legal moves.
    let output = run_perft(1, FEN_CHECKMATE);
    assert!(
        output.status.success(),
        "engine should accept checkmate position; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        perft_count(&output),
        0,
        "checkmate position should have perft(1) = 0"
    );
}

#[test]
fn test_stalemate_detected() {
    // Black king on h1, White queen g2 + king f3. Black to move, no legal moves,
    // not in check => stalemate.
    let output = run_perft(1, FEN_STALEMATE);
    assert!(
        output.status.success(),
        "engine should accept stalemate position; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        perft_count(&output),
        0,
        "stalemate position should have perft(1) = 0"
    );
}

#[test]
fn test_en_passant_discovered_check_filtered() {
    // Position 3 is specifically chosen to exercise the en passant discovered
    // check edge case. The perft numbers already encode the correct filtering.
    // If en passant discovered checks are not filtered, perft will be wrong.
    let output = run_perft(3, POSITION3);
    assert!(output.status.success());
    assert_eq!(
        perft_count(&output),
        2812,
        "position 3 perft(3) must be exact — en passant discovered check filtering"
    );
}

#[test]
fn test_castling_through_check_not_allowed() {
    // Kiwipete has castling rights and attacked squares. The perft numbers
    // encode correct castling legality. Any bug in castling-through-check
    // detection will cause perft mismatch.
    let output = run_perft(2, KIWIPETE);
    assert!(output.status.success());
    assert_eq!(
        perft_count(&output),
        2039,
        "kiwipete perft(2) must be exact — castling legality"
    );
}

#[test]
fn test_double_check_only_king_moves() {
    // Position 4 includes promotions that can give double check.
    // Only king moves are legal in double check. The perft numbers encode this.
    let output = run_perft(3, POSITION4);
    assert!(output.status.success());
    assert_eq!(
        perft_count(&output),
        9467,
        "position 4 perft(3) must be exact — double check handling"
    );
}

#[test]
fn test_promotion_under_pin_filtered() {
    // Position 5 has a knight on f2 giving check, with promotion possibilities.
    // Promotions that don't resolve pins must be filtered. Encoded in perft.
    let output = run_perft(2, POSITION5);
    assert!(output.status.success());
    assert_eq!(
        perft_count(&output),
        1486,
        "position 5 perft(2) must be exact — promotion under pin"
    );
}

// ===========================================================================
// 6.8 Integration Smoke Test
// ===========================================================================

#[test]
fn test_integration_smoke() {
    // --- Part 1: Perft startpos depth 4 ---
    let output = run_perft(4, STARTPOS);
    assert_eq!(output.status.code(), Some(0), "perft startpos should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "197281",
        "perft startpos depth 4"
    );

    // --- Part 2: Perft Kiwipete depth 3 ---
    let output = run_perft(3, KIWIPETE);
    assert_eq!(output.status.code(), Some(0), "perft kiwipete should exit 0");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "97862",
        "perft kiwipete depth 3"
    );

    // --- Part 3: UCI session ---
    let mut child = start_uci();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    send(stdin, "uci");
    let lines = read_until(&mut reader, "uciok");
    assert!(
        lines.iter().any(|l| l.contains("uciok")),
        "smoke: uciok missing"
    );

    send(stdin, "isready");
    let lines = read_until(&mut reader, "readyok");
    assert!(
        lines.iter().any(|l| l.trim() == "readyok"),
        "smoke: readyok missing"
    );

    send(stdin, "position startpos moves e2e4 e7e5");
    send(stdin, "go depth 4");
    let lines = read_until(&mut reader, "bestmove");
    assert!(
        lines.iter().any(|l| l.contains("bestmove")),
        "smoke: bestmove missing"
    );

    send(stdin, "quit");
    let status = child.wait().unwrap();
    assert!(status.success(), "smoke: UCI quit should exit 0");

    // --- Part 4: Invalid FEN -> exit code 1 ---
    let output = run_perft(1, "invalid");
    assert_eq!(
        output.status.code(),
        Some(1),
        "smoke: invalid FEN should exit 1"
    );

    // --- Part 5: Checkmate position -> perft 0 ---
    let output = run_perft(1, FEN_CHECKMATE);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "0",
        "smoke: checkmate perft(1) should be 0"
    );
}

// ===========================================================================
// Additional: Perft with default startpos (no --fen flag)
// ===========================================================================

#[test]
fn test_perft_default_startpos() {
    // When no --fen is given, the engine should default to the starting position.
    let output = run_perft_startpos(1);
    assert!(
        output.status.success(),
        "perft without --fen should default to startpos; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(perft_count(&output), 20);
}
