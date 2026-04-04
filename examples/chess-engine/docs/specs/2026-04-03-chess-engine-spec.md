---
date: 2026-04-03
topic: chess-engine
status: reviewed
---

# Chess Engine Specification

## Overview

A command-line chess engine in Rust that implements bitboard-based board representation, full legal move generation, alpha-beta search with iterative deepening, tapered evaluation with piece-square tables, and a minimal UCI protocol interface. The engine validates correctness through perft testing against published reference numbers.

## Scope

- Board representation using bitboards (12 x `u64`)
- FEN parsing and output
- Legal move generation (all piece types, castling, en passant, promotion)
- Perft command for move generation validation
- Alpha-beta search with iterative deepening and quiescence search
- Evaluation: material values + piece-square tables, tapered between middlegame and endgame
- UCI protocol (minimal subset: `uci`, `isready`, `position`, `go depth`/`go movetime`, `bestmove`, `quit`)
- CLI binary with `perft` subcommand

## Requirements

### R1. FEN Parsing and Output

The engine shall parse and output Forsyth-Edwards Notation strings containing all six fields: piece placement, active color, castling availability, en passant target, halfmove clock, and fullmove number.

- Piece placement: ranks 8 to 1 separated by `/`, uppercase = white, lowercase = black, digits 1-8 = consecutive empty squares.
- Active color: `w` or `b`.
- Castling availability: combination of `K`, `Q`, `k`, `q`, or `-` if none.
- En passant target: algebraic square or `-`.
- Halfmove clock: non-negative integer.
- Fullmove number: positive integer starting at 1.
- Invalid FEN (wrong field count, illegal characters, impossible board state) shall produce an error message on stderr and exit code 1.

### R2. Legal Move Generation

The engine shall generate all legal moves for any valid chess position, including:

- Pawn: single push, double push from starting rank, diagonal captures, en passant captures, promotion (queen, rook, bishop, knight) on reaching the back rank.
- Knight: L-shaped jumps to unoccupied or enemy-occupied squares.
- Bishop: diagonal sliding until blocked.
- Rook: horizontal/vertical sliding until blocked.
- Queen: combination of rook and bishop movement.
- King: one square in any direction, plus castling.
- Castling: kingside and queenside for both colors, subject to all castling conditions (king/rook unmoved, intervening squares empty, king not in check, king does not pass through or land on attacked square).

A move is legal if and only if, after making it, the moving side's king is not in check.

### R3. Perft Correctness

The engine shall provide a `perft` command that counts leaf nodes at a given depth. The counts shall match the following published reference numbers exactly:

**Position 1 (startpos):** `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`
| Depth | Nodes |
|-------|-------|
| 1 | 20 |
| 2 | 400 |
| 3 | 8,902 |
| 4 | 197,281 |
| 5 | 4,865,609 |

**Position 2 (Kiwipete):** `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -`
| Depth | Nodes |
|-------|-------|
| 1 | 48 |
| 2 | 2,039 |
| 3 | 97,862 |
| 4 | 4,085,603 |
| 5 | 193,690,690 |

**Position 3:** `8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -`
| Depth | Nodes |
|-------|-------|
| 1 | 14 |
| 2 | 191 |
| 3 | 2,812 |
| 4 | 43,238 |
| 5 | 674,624 |

**Position 4:** `r3k2r/Pppppppp/8/8/8/8/pPPPPPPP/R3K2R w KQkq - 0 1`
| Depth | Nodes |
|-------|-------|
| 1 | 26 |
| 2 | 568 |
| 3 | 13,744 |
| 4 | 314,346 |
| 5 | 8,153,719 |

**Position 5:** `rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -`
| Depth | Nodes |
|-------|-------|
| 1 | 44 |
| 2 | 1,486 |
| 3 | 62,379 |
| 4 | 2,103,487 |
| 5 | 89,941,194 |

These are leaf-node counts (positions reachable in exactly N moves), not cumulative.

### R4. Search Performance

The engine shall find reasonable moves at depth 6 or greater within 5 seconds on a modern machine. "Reasonable" means the engine does not hang a piece or miss a one-move tactic in standard middlegame positions.

### R5. UCI Protocol Compliance

The engine shall implement the following UCI commands:

- `uci` -- respond with `id name`, `id author`, and `uciok`.
- `isready` -- respond with `readyok`.
- `position startpos [moves ...]` -- set position from starting position, optionally applying a sequence of moves.
- `position fen <fen> [moves ...]` -- set position from FEN string, optionally applying moves.
- `go depth <n>` -- search to depth N, respond with `bestmove`.
- `go movetime <ms>` -- search for at most the given milliseconds, respond with `bestmove`.
- `quit` -- exit the engine.

Moves use UCI long algebraic notation: `e2e4`, `e7e8q` (promotion), `e1g1` (castling).

The engine should output `info` lines during search with at least: `depth`, `score cp`, `nodes`, `time`, `pv`.

### R6. Exit Codes

- `0` on normal quit (UCI `quit` command or successful perft).
- `1` on invalid input (bad FEN, unknown command, malformed arguments).

### R7. Edge Cases

The engine shall correctly handle:

- **Checkmate:** no legal moves and king is in check. Search returns mate score.
- **Stalemate:** no legal moves and king is not in check. Search returns 0 (draw).
- **Insufficient material:** recognized but not required to force draw (the engine may continue playing).
- **En passant discovered check:** en passant capture that would expose the king to a horizontal attack by removing both pawns from the rank is correctly detected as illegal.
- **Castling through/out-of check:** all castling preconditions enforced.
- **Promotion under check:** pawn promotion moves that leave the king in check are filtered.

### R8. Dependencies

No external dependencies except `clap` for CLI argument parsing. All chess logic uses only the Rust standard library.
