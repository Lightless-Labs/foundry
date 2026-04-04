---
date: 2026-04-03
topic: chess-engine
source_spec: docs/specs/2026-04-03-chess-engine-spec.md
status: reviewed
---

# Chess Engine NLSpec

A command-line chess engine in Rust: bitboard representation, full legal move generation, alpha-beta search with iterative deepening, tapered evaluation (PeSTO tables), and minimal UCI protocol. Correctness is proven by matching published perft numbers exactly.

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

Chess engines are the canonical benchmark for game-tree search. This implementation serves as a Foundry adversarial red/green workflow example -- the red team writes tests from the Definition of Done, the green team implements from the How section, with perft golden vectors as the shared contract.

### 1.2 Design Principles

**Bitboards for speed and correctness.** 64-bit integers map naturally to the 64 squares. Set operations compile to single CPU instructions.

**Perft as the truth.** Move generation is validated exclusively against published perft numbers. If perft matches, the move generator is correct.

**Minimal viable engine.** Material + piece-square tables + alpha-beta. No null-move pruning, no LMR, no opening book.

**Single external dependency.** Only `clap` for CLI parsing. All chess logic uses `std` only.

### 1.3 Layering and Scope

Covers: board representation, FEN parsing/output, legal move generation, perft, evaluation, search, UCI (minimal subset), CLI binary. Does NOT cover: opening books, endgame tablebases, null-move pruning, multi-threading, or clock-based time control.

---

## 2. What

### 2.1 Data Model

```
RECORD Board:
    pieces: [[Bitboard; 6]; 2]     -- [Color][PieceType] = 12 bitboards
    occupancy: [Bitboard; 2]       -- [Color] aggregate
    all_occupancy: Bitboard        -- union of both colors
    mailbox: [Option<(Color, PieceType)>; 64]  -- O(1) square lookup
    side_to_move: Color
    castling_rights: u8            -- bits: K=1, Q=2, k=4, q=8
    en_passant_square: Option<u8>  -- target square (0-63) or None
    halfmove_clock: u16
    fullmove_number: u16
    zobrist_hash: u64

ENUM Color: White = 0, Black = 1
ENUM PieceType: Pawn = 0, Knight = 1, Bishop = 2, Rook = 3, Queen = 4, King = 5

RECORD Move:
    from: u8, to: u8, promotion: Option<PieceType>

Square mapping (LERF): a1=0, b1=1, ..., h1=7, a2=8, ..., h8=63
    rank(sq) = sq / 8, file(sq) = sq % 8
```

### 2.2 Architecture

Five modules:
- `board` -- Board, bitboard ops, FEN parsing/output, make/unmake, Zobrist
- `movegen` -- pseudo-legal/legal move generation, attack tables, perft
- `eval` -- material values, piece-square tables, tapered evaluation
- `search` -- alpha-beta, quiescence, iterative deepening, transposition table
- `uci` -- UCI protocol loop, CLI entry point

### 2.3 Vocabulary

- **LERF**: Little-Endian Rank-File. a1=bit 0, h8=bit 63.
- **Pseudo-legal move**: respects piece movement but may leave own king in check.
- **Legal move**: pseudo-legal move verified not to leave own king in check.
- **Perft(depth)**: leaf-node count at exactly `depth` plies (not cumulative).
- **Centipawn (cp)**: 1/100th of a pawn.
- **Phase**: 24 (all pieces, middlegame) down to 0 (endgame). Interpolates eval.
- **MVV-LVA**: Most Valuable Victim, Least Valuable Attacker. Capture ordering.
- **PV**: Principal Variation. Best line found by search.

---

## 3. How

### 3.1 FEN Parsing

```
FUNCTION parse_fen(fen: &str) -> Result<Board, Error>:
    fields = fen.split_whitespace()   -- expect 4-6 fields

    -- Field 1: piece placement (ranks 8..1 separated by '/')
    FOR each rank string (8 ranks, top to bottom):
        FOR each char:
            digit 1-8 -> skip that many files
            letter in "pnbrqkPNBRQK" -> place piece at sq = (7 - rank_idx)*8 + file
        file must equal 8 after processing each rank

    -- Field 2: side_to_move = 'w' -> White, 'b' -> Black
    -- Field 3: castling = "KQkq" -> bits, "-" -> 0
    -- Field 4: en_passant = "-" -> None, "e3" -> Some(algebraic_to_sq)
    -- Field 5-6: halfmove_clock, fullmove_number (default 0, 1 if absent)
    -- Compute zobrist hash from final board state

FUNCTION board_to_fen(board) -> String:
    -- Reverse of parse_fen: emit ranks 8..1, collapse consecutive empties to digits
```

### 3.2 Board Representation

```
-- Pre-computed attack tables (init once at startup)
KNIGHT_ATTACKS: [Bitboard; 64]
KING_ATTACKS: [Bitboard; 64]
PAWN_ATTACKS: [[Bitboard; 64]; 2]   -- [Color][Square], diagonal attacks only

-- Sliding attacks via magic bitboards
FUNCTION bishop_attacks(sq, occupancy) -> Bitboard:
    index = ((occupancy & BISHOP_MASK[sq]) * BISHOP_MAGIC[sq]) >> BISHOP_SHIFT[sq]
    RETURN BISHOP_TABLE[sq][index]

FUNCTION rook_attacks(sq, occupancy) -> Bitboard:
    -- same pattern with ROOK_MASK/MAGIC/SHIFT/TABLE

FUNCTION queen_attacks(sq, occ) = bishop_attacks(sq,occ) | rook_attacks(sq,occ)

FUNCTION make_move(board, mv) -> UndoInfo:
    -- Remove piece from source (bitboards + mailbox + zobrist)
    -- Remove captured piece if any (en passant: captured pawn at different sq)
    -- Place piece (or promoted piece) at destination
    -- Castling: also move the rook (h1<->f1, a1<->d1, etc.)
    -- Update castling rights, en passant sq, halfmove clock
    -- Toggle side_to_move, increment fullmove after Black
    -- Incrementally update zobrist hash

FUNCTION unmake_move(board, mv, undo):
    -- Reverse of make_move using saved UndoInfo
```

### 3.3 Move Generation

```
FUNCTION generate_pseudo_legal_moves(board) -> Vec<Move>:
    us = board.side_to_move; them = opposite(us)

    -- Pawns: single push, double push (from start rank), diagonal captures,
    --   en passant, promotion (all 4 pieces) on reaching back rank
    -- Knights: KNIGHT_ATTACKS[sq] & ~own_pieces
    -- Bishops: bishop_attacks(sq, all_occ) & ~own_pieces
    -- Rooks: rook_attacks(sq, all_occ) & ~own_pieces
    -- Queens: queen_attacks(sq, all_occ) & ~own_pieces
    -- King: KING_ATTACKS[sq] & ~own_pieces

    -- Castling (White kingside example):
    --   castling_rights & K set, f1+g1 empty,
    --   e1+f1+g1 not attacked by them -> push Move(e1, g1)
    -- Queenside: b1+c1+d1 empty, c1+d1+e1 not attacked
    -- Mirror for Black (e8/f8/g8, e8/d8/c8/b8)
```

### 3.4 Legal Move Filtering

```
FUNCTION generate_legal_moves(board) -> Vec<Move>:
    FOR mv IN generate_pseudo_legal_moves(board):
        make_move(board, mv)
        IF NOT is_king_in_check(board, side_that_just_moved):
            keep mv
        unmake_move(board, mv)

FUNCTION is_attacked(board, sq, by_color) -> bool:
    -- "Super-piece" check from sq:
    KNIGHT_ATTACKS[sq] & enemy_knights != 0 OR
    PAWN_ATTACKS[opposite][sq] & enemy_pawns != 0 OR
    KING_ATTACKS[sq] & enemy_king != 0 OR
    bishop_attacks(sq, occ) & (enemy_bishops | enemy_queens) != 0 OR
    rook_attacks(sq, occ) & (enemy_rooks | enemy_queens) != 0
```

### 3.5 Perft

```
FUNCTION perft(board, depth) -> u64:
    IF depth == 0: RETURN 1
    moves = generate_legal_moves(board)
    IF depth == 1: RETURN moves.len()   -- bulk counting
    sum = 0
    FOR mv IN moves:
        make_move(board, mv); sum += perft(board, depth-1); unmake_move(board, mv)
    RETURN sum

FUNCTION perft_divide(board, depth):
    -- Per-move breakdown: for each legal move, print "mv: count"
```

### 3.6 Evaluation

```
-- PeSTO material values (centipawns)
MG_VALUE = [82, 337, 365, 477, 1025, 0]   -- P, N, B, R, Q, K
EG_VALUE = [94, 281, 297, 512,  936, 0]

-- PeSTO piece-square tables: MG_PST[6][64] and EG_PST[6][64]
-- White perspective, a8=index 0, h1=index 63. For Black: sq ^ 56.
-- (Full table values in research doc)

PHASE_WEIGHT = [0, 1, 1, 2, 4, 0]   -- P, N, B, R, Q, K
TOTAL_PHASE = 24

FUNCTION evaluate(board) -> i32:
    mg = [0, 0]; eg = [0, 0]; phase = 0
    FOR sq IN 0..64:
        IF piece at sq -> (color, piece_type):
            tbl_sq = IF White: sq ELSE: sq ^ 56
            mg[color] += MG_VALUE[pt] + MG_PST[pt][tbl_sq]
            eg[color] += EG_VALUE[pt] + EG_PST[pt][tbl_sq]
            phase += PHASE_WEIGHT[pt]
    -- Tapered: (mg_diff * clamped_phase + eg_diff * (24 - clamped_phase)) / 24
    -- Score relative to side_to_move (positive = good)
```

### 3.7 Search

```
FUNCTION iterative_deepening(board, limits) -> Move:
    FOR depth IN 1..=max_depth:
        score = alpha_beta(board, depth, -INF, +INF, 0)
        print info(depth, score, nodes, time, pv)
        IF time_exceeded: BREAK
    RETURN pv[0]

FUNCTION alpha_beta(board, depth, alpha, beta, ply) -> i32:
    -- TT probe: if valid entry with depth >= current, use/adjust bounds
    IF depth <= 0: RETURN quiescence(board, alpha, beta)

    moves = generate_legal_moves(board)
    IF moves.empty():
        RETURN IF in_check: -MATE_SCORE + ply ELSE: 0

    order_moves(moves, tt_best_move)
    -- Priority: TT move > captures (MVV-LVA) > killer moves > quiet moves

    best_score = -INF
    FOR mv IN moves:
        make; score = -alpha_beta(board, depth-1, -beta, -alpha, ply+1); unmake
        best_score = max(best_score, score)
        IF score > alpha: alpha = score; update PV
        IF alpha >= beta: BREAK   -- cutoff

    tt_store(hash, depth, best_score, flag, best_move)
    RETURN best_score

FUNCTION quiescence(board, alpha, beta) -> i32:
    stand_pat = evaluate(board)
    IF stand_pat >= beta: RETURN beta
    alpha = max(alpha, stand_pat)
    FOR capture IN ordered_legal_captures(board):
        make; score = -quiescence(board, -beta, -alpha); unmake
        IF score >= beta: RETURN beta
        alpha = max(alpha, score)
    RETURN alpha
```

### 3.8 UCI Protocol Loop

```
FUNCTION uci_loop():
    board = startpos

    LOOP on stdin lines:
        "uci"      -> print "id name Foundry Chess 0.1.0\nid author Lightless Labs\nuciok"
        "isready"  -> print "readyok"
        "position startpos [moves m1 m2 ...]" -> set startpos, apply moves
        "position fen <fen> [moves m1 m2 ...]" -> parse FEN, apply moves
        "go depth N"      -> search to depth N, print "bestmove <mv>"
        "go movetime N"   -> search for N ms, print "bestmove <mv>"
        "quit"            -> exit(0)
        _                 -> ignore (per UCI spec)

-- Move format: "e2e4" (normal), "e7e8q" (promotion), "e1g1" (castling)
```

### 3.9 CLI Entry Point

```
FUNCTION main():   -- uses clap
    MATCH subcommand:
        None | "uci" -> uci_loop()
        "perft"      -> parse_fen(--fen or startpos), print perft(depth), exit(0)
        "divide"     -> parse_fen(--fen or startpos), perft_divide(depth), exit(0)
    On FEN error: eprintln error, exit(1)
```

---

## 4. Out of Scope

- **Opening book.** Extension: load polyglot `.bin`.
- **Endgame tablebases.** Extension: Syzygy WDL/DTZ probing.
- **Null-move pruning, LMR, futility pruning.** Extension points in search.
- **Multi-threaded search.** Extension: Lazy SMP with shared TT.
- **Pondering, full UCI options, clock-based time management.**

---

## 5. Design Decision Rationale

**Bitboards over 8x8 array:** bulk operations ("all squares attacked by white knights") reduce to a single OR over lookups. Mailbox kept alongside for O(1) piece-on-square queries.

**Magic bitboards for sliders:** O(1) attack generation vs O(n) ray scanning. Magic numbers can be hardcoded from known values.

**Pseudo-legal + legality filter:** simpler than generating only legal moves (which requires explicit pin tracking). Correctness validated by perft.

**PeSTO tables:** Texel-tuned against millions of positions, de facto standard for simple eval. Outperforms hand-tuned tables with zero extra complexity.

**Tapered eval:** king centralization is bad in middlegame, good in endgame. Smooth interpolation based on remaining material handles the transition.

**Transposition table:** many positions reached via different move orders. Caching avoids redundant search, critical for depth 6+.

---

## 6. Definition of Done

### 6.1 FEN Parsing

- [ ] Parses standard starting position FEN correctly
- [ ] Parses all 6 fields; defaults halfmove=0, fullmove=1 if absent
- [ ] Rejects: wrong rank count, invalid chars, rank not summing to 8, invalid side
- [ ] Round-trips: `board_to_fen(parse_fen(fen)) == fen`
- [ ] Exits code 1 with error on stderr for invalid FEN

### 6.2 Move Generation

- [ ] Pawn: single push, double push, diagonal capture, en passant, promotion (4 types)
- [ ] En passant rejected when it exposes king to horizontal discovered check
- [ ] Knight, bishop, rook, queen: standard movement, blocked by own pieces
- [ ] King: one square any direction, cannot move into check
- [ ] Kingside castling: king+rook unmoved, f/g empty, e/f/g not attacked
- [ ] Queenside castling: king+rook unmoved, b/c/d empty, c/d/e not attacked (b may be attacked)
- [ ] Castling rights cleared on king move, rook move, or rook capture
- [ ] No legal moves + in check = checkmate; no legal moves + not in check = stalemate

### 6.3 Perft Golden Vectors (P0 -- must all pass exactly)

**Position 1 (startpos):** `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`
- [ ] perft(1) = 20
- [ ] perft(2) = 400
- [ ] perft(3) = 8,902
- [ ] perft(4) = 197,281
- [ ] perft(5) = 4,865,609

**Position 2 (Kiwipete):** `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -`
- [ ] perft(1) = 48
- [ ] perft(2) = 2,039
- [ ] perft(3) = 97,862
- [ ] perft(4) = 4,085,603
- [ ] perft(5) = 193,690,690

**Position 3:** `8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -`
- [ ] perft(1) = 14
- [ ] perft(2) = 191
- [ ] perft(3) = 2,812
- [ ] perft(4) = 43,238
- [ ] perft(5) = 674,624

**Position 4:** `r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1`
- [ ] perft(1) = 6
- [ ] perft(2) = 264
- [ ] perft(3) = 9,467
- [ ] perft(4) = 422,333

**Position 5:** `rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ -`
- [ ] perft(1) = 44
- [ ] perft(2) = 1,486
- [ ] perft(3) = 62,379
- [ ] perft(4) = 2,103,487
- [ ] perft(5) = 89,941,194

These are EXACT leaf-node counts. Any deviation indicates a move generation bug.

### 6.4 Search

- [ ] Returns a legal move for any non-terminal position
- [ ] Iterative deepening depth 1..N, reporting info at each depth
- [ ] Quiescence search extends captures at leaf nodes
- [ ] Checkmate: returns mate score; stalemate: returns 0
- [ ] Transposition table avoids redundant work
- [ ] Depth 6 from startpos completes in under 5 seconds
- [ ] No crash or hang on any valid position

### 6.5 Evaluation

- [ ] PeSTO material: P=82/94, N=337/281, B=365/297, R=477/512, Q=1025/936 (mg/eg cp)
- [ ] PeSTO piece-square tables applied; Black uses sq ^ 56
- [ ] Tapered eval: phase 24 (middlegame) to 0 (endgame)
- [ ] Score relative to side-to-move; startpos evaluates to ~0

### 6.6 UCI Protocol

- [ ] `uci` -> `id name`, `id author`, `uciok`
- [ ] `isready` -> `readyok`
- [ ] `position startpos [moves ...]` sets position
- [ ] `position fen <fen> [moves ...]` sets position
- [ ] `go depth N` -> `info` lines + `bestmove` in UCI long algebraic
- [ ] `go movetime N` -> searches for at most N ms
- [ ] `quit` -> exit code 0
- [ ] Unknown commands silently ignored

### 6.7 Edge Cases

- [ ] Checkmate: empty legal move list, mate score returned
- [ ] Stalemate: empty legal move list, score 0
- [ ] Kings only: eval 0, search returns 0
- [ ] En passant discovered check: illegal EP not in legal moves
- [ ] Castling out of / through check: not generated
- [ ] Promotion under pin: filtered by legality check
- [ ] Double check: only king moves are legal

### 6.8 Integration Smoke Test

```
FUNCTION integration_smoke_test():
    -- Perft startpos depth 4
    result = run_engine("perft", "--fen", STARTPOS, "--depth", "4")
    ASSERT result.exit_code == 0
    ASSERT result.stdout.trim() == "197281"

    -- Perft Kiwipete depth 3
    result = run_engine("perft", "--fen", KIWIPETE, "--depth", "3")
    ASSERT result.exit_code == 0
    ASSERT result.stdout.trim() == "97862"

    -- UCI session
    session = start_engine("uci")
    send("uci");       ASSERT receive().contains("uciok")
    send("isready");   ASSERT receive() == "readyok"
    send("position startpos moves e2e4 e7e5")
    send("go depth 4")
    ASSERT receive_until("bestmove").contains("bestmove")
    send("quit");      ASSERT session.exit_code == 0

    -- Invalid FEN -> exit code 1
    result = run_engine("perft", "--fen", "invalid", "--depth", "1")
    ASSERT result.exit_code == 1

    -- Checkmate position (scholar's mate): 0 legal moves
    result = run_engine("perft", "--fen",
        "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
        "--depth", "1")
    ASSERT result.exit_code == 0
    ASSERT result.stdout.trim() == "0"
```
