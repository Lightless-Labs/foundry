# Chess Engine Research: Rust CLI Engine with UCI Protocol

**Date:** 2026-04-03
**Purpose:** Verified technical reference for implementing a chess engine in Rust
**Sources:** Chess Programming Wiki (chessprogramming.org), Stockfish UCI docs, python-chess test vectors, Fairy-Stockfish perft scripts, TalkChess forums

---

## 1. Board Representation: Bitboards

### Core Structure: 12 Bitboards

Each bitboard is a `u64` where each bit represents one square. Bit 0 = a1, bit 63 = h8 (Little-Endian Rank-File mapping, LERF).

```
Square index layout (LERF):
  a  b  c  d  e  f  g  h
8: 56 57 58 59 60 61 62 63
7: 48 49 50 51 52 53 54 55
6: 40 41 42 43 44 45 46 47
5: 32 33 34 35 36 37 38 39
4: 24 25 26 27 28 29 30 31
3: 16 17 18 19 20 21 22 23
2:  8  9 10 11 12 13 14 15
1:  0  1  2  3  4  5  6  7
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bitboard(pub u64);

pub struct Board {
    // One bitboard per piece type per color = 12 total
    pieces: [[Bitboard; 6]; 2],  // [Color][PieceType]
    // Aggregate bitboards for fast queries
    occupancy: [Bitboard; 2],    // [Color] - all pieces of that color
    all_occupancy: Bitboard,     // all pieces on board
    // Mailbox for O(1) "what piece is on square X?"
    mailbox: [Option<(Color, PieceType)>; 64],
}

#[repr(u8)]
pub enum PieceType { Pawn = 0, Knight = 1, Bishop = 2, Rook = 3, Queen = 4, King = 5 }

#[repr(u8)]
pub enum Color { White = 0, Black = 1 }
```

### Attack Generation

**Leaper pieces (knights, kings):** Pre-computed lookup tables of 64 entries each. At engine init, compute all possible attack squares for each position.

```rust
// Knight attack offsets: +/- (1,2) and (2,1) in all combinations
static KNIGHT_ATTACKS: [Bitboard; 64] = /* pre-computed at build time or init */;
static KING_ATTACKS: [Bitboard; 64] = /* pre-computed */;
// Pawn attacks are direction-dependent (color-specific)
static PAWN_ATTACKS: [[Bitboard; 64]; 2] = /* [Color][Square] */;
```

**Sliding pieces (bishops, rooks, queens):** Use magic bitboards.

The magic bitboard technique:
1. For each square, define an **attack mask** (relevant squares the piece can attack, excluding edges)
2. Given an occupancy pattern (pieces blocking the rays), compute the actual attacks
3. Use a "magic number" to hash the occupancy into a lookup table index:
   ```
   index = ((occupancy & mask) * magic_number) >> shift
   attacks = ATTACK_TABLE[square][index]
   ```
4. Queens combine rook attacks and bishop attacks: `queen_attacks = rook_attacks | bishop_attacks`

**Mailbox hybrid:** The `mailbox: [Option<(Color, PieceType)>; 64]` array provides O(1) lookup for "what piece is on this square?" -- useful during move generation (determining capture victim), FEN parsing, and evaluation. It is kept in sync with the bitboards on every make/unmake.

### Zobrist Hashing

Pre-compute random `u64` values for each combination:
- 12 x 64 = 768 values for piece-on-square
- 1 value for side-to-move
- 4 values for castling rights (K, Q, k, q)
- 8 values for en passant file (a-h)

```rust
struct ZobristKeys {
    pieces: [[[u64; 64]; 6]; 2],  // [color][piece_type][square]
    side_to_move: u64,
    castling: [u64; 4],           // KQkq
    en_passant_file: [u64; 8],    // files a-h
}
```

Hash is computed incrementally via XOR:
- Initial: XOR all piece-square keys for the starting position
- On move: XOR out old piece position, XOR in new position
- On capture: also XOR out captured piece
- Toggle side-to-move key each half-move
- Update castling/en-passant keys as needed

Self-inverse property: `hash ^ key ^ key == hash`, making undo trivial.

### Transposition Table

```rust
struct TTEntry {
    hash: u64,         // Full Zobrist hash (for collision detection)
    depth: u8,         // Search depth this entry was computed at
    score: i16,        // Evaluation score
    flag: TTFlag,      // Exact, LowerBound (beta cutoff), UpperBound (alpha not raised)
    best_move: Move,   // Best move found (for move ordering)
}

enum TTFlag { Exact, LowerBound, UpperBound }
```

Table size typically power-of-2 entries. Index = `hash % table_size`. Replace on deeper search or same depth.

---

## 2. FEN (Forsyth-Edwards Notation)

**Source:** https://kirill-kryukov.com/chess/doc/fen.html, https://www.chessprogramming.org/Forsyth-Edwards_Notation

Six space-separated fields:

| Field | Description | Example |
|-------|-------------|---------|
| 1. Piece placement | Ranks 8 to 1 (top to bottom), separated by `/`. Uppercase = white, lowercase = black. Digits 1-8 = consecutive empty squares. | `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR` |
| 2. Active color | `w` = white to move, `b` = black to move | `w` |
| 3. Castling availability | `K` = white kingside, `Q` = white queenside, `k` = black kingside, `q` = black queenside. `-` if none available. | `KQkq` |
| 4. En passant target | Algebraic notation of the square "behind" the pawn that just advanced two squares, or `-` if none. | `-` or `e3` |
| 5. Halfmove clock | Number of half-moves since last pawn advance or capture (for 50-move rule). Non-negative integer. | `0` |
| 6. Fullmove number | Starts at 1, incremented after black's move. | `1` |

**Starting position FEN:**
```
rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
```

**Piece letters:** K=King, Q=Queen, R=Rook, B=Bishop, N=Knight, P=Pawn

**Parsing notes:**
- Piece placement is read left-to-right within each rank (a-file to h-file)
- Rank 8 (black's back rank) is listed first, rank 1 (white's back rank) last
- En passant target square is recorded even if no legal en passant capture exists (some implementations omit it in this case)

---

## 3. Move Generation

### Pseudo-Legal vs Legal

1. Generate **pseudo-legal** moves: respect piece movement rules but ignore whether the king is left in check
2. For each pseudo-legal move, make the move on the board, then check if the own king is attacked
3. If king is attacked, the move is illegal -- unmake and discard

### Castling Rules

A castling move is legal if and only if ALL of these conditions hold:

| Condition | Kingside (e.g., White O-O) | Queenside (e.g., White O-O-O) |
|-----------|---------------------------|-------------------------------|
| King has not moved | e1 for white | e1 for white |
| Rook has not moved | h1 for white | a1 for white |
| Squares between are empty | f1, g1 | b1, c1, d1 |
| King is NOT currently in check | e1 not attacked | e1 not attacked |
| King does not pass through attacked square | f1 not attacked | d1 not attacked |
| King does not land on attacked square | g1 not attacked | c1 not attacked |

**Note:** The rook MAY pass through an attacked square (relevant for queenside castling where b1 can be attacked). Only the king's path matters.

**UCI notation:** Castling is encoded as king move: `e1g1` (white kingside), `e1c1` (white queenside), `e8g8` (black kingside), `e8c8` (black queenside).

### En Passant Rules

1. An enemy pawn must have just advanced two squares from its starting rank on the immediately preceding move
2. The capturing pawn must be on its 5th rank (rank 5 for white, rank 4 for black)
3. The capturing pawn must be adjacent (on a neighboring file) to the enemy pawn
4. The capture is made diagonally to the square the enemy pawn "passed through"
5. The enemy pawn is removed from the board (it is NOT on the destination square)
6. The right expires immediately -- it is only available on the very next move

**Special pin case:** En passant can expose the king to a horizontal check. Example: white king on e5, white pawn on d5, black pawn on e5, black rook on a5. If black plays d7-d5, white's en passant dxe6 would remove BOTH the white pawn from d5 AND the black pawn from d5, exposing the king to the rook. This must be checked by full make/unmake legality testing.

### Pawn Promotion

When a pawn reaches the opponent's back rank (rank 8 for white, rank 1 for black), it MUST be promoted. Four possible promotions per advance = 4 distinct moves:
- Queen promotion (most common)
- Rook promotion (rare, avoids stalemate in edge cases)
- Bishop promotion (very rare)
- Knight promotion (rare but tactically important for fork patterns)

**UCI notation:** Promotion piece appended lowercase: `e7e8q`, `e7e8r`, `e7e8n`, `e7e8b`

### Check Detection

After making a move, determine if the opponent can attack the moving side's king square:
1. From king square, generate attack patterns for each piece type in reverse
2. If a "reverse knight attack" from the king square hits an enemy knight -> check
3. If a "reverse bishop attack" hits an enemy bishop or queen -> check
4. If a "reverse rook attack" hits an enemy rook or queen -> check
5. If a "reverse pawn attack" hits an enemy pawn -> check

This is efficient with bitboards: `(knight_attacks[king_sq] & enemy_knights) != 0`, etc.

### Pin Detection

Pins are handled implicitly by the pseudo-legal + legality check approach: any move that leaves the king in check (including moving a pinned piece off the pin line) is filtered out during the legality check.

For more advanced engines, pins can be explicitly detected for move ordering or pruning:
1. From the king square, cast rays in all 8 directions
2. If a ray hits exactly one friendly piece, then hits an enemy sliding piece that attacks along that ray, the friendly piece is pinned
3. Pinned pieces can only move along the pin ray

---

## 4. Perft Numbers (Golden Test Vectors)

**Source:** https://www.chessprogramming.org/Perft_Results (verified against Fairy-Stockfish test scripts, python-chess test vectors, Stockfish, Quick Perft by H.G. Muller)

**CRITICAL:** These numbers are leaf node counts only. Each depth counts positions reachable in exactly that many moves, NOT cumulative.

### Position 1: Starting Position

FEN: `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`

| Depth | Nodes |
|-------|-------|
| 1 | 20 |
| 2 | 400 |
| 3 | 8,902 |
| 4 | 197,281 |
| 5 | 4,865,609 |
| 6 | 119,060,324 |
| 7 | 3,195,901,860 |

### Position 2: "Kiwipete" (by Peter McKenzie)

FEN: `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -`

| Depth | Nodes |
|-------|-------|
| 1 | 48 |
| 2 | 2,039 |
| 3 | 97,862 |
| 4 | 4,085,603 |
| 5 | 193,690,690 |
| 6 | 8,031,647,685 |

### Position 3

FEN: `8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -`

| Depth | Nodes |
|-------|-------|
| 1 | 14 |
| 2 | 191 |
| 3 | 2,812 |
| 4 | 43,238 |
| 5 | 674,624 |
| 6 | 11,030,083 |
| 7 | 178,633,661 |

### Position 4

FEN: `r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1`

Mirrored (same perft results): `r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1`

| Depth | Nodes |
|-------|-------|
| 1 | 6 |
| 2 | 264 |
| 3 | 9,467 |
| 4 | 422,333 |
| 5 | 15,833,292 |
| 6 | 706,045,033 |

### Position 5

FEN: `rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8`

| Depth | Nodes |
|-------|-------|
| 1 | 44 |
| 2 | 1,486 |
| 3 | 62,379 |
| 4 | 2,103,487 |
| 5 | 89,941,194 |

### Position 6 (by Steven Edwards)

FEN: `r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10`

| Depth | Nodes |
|-------|-------|
| 1 | 46 |
| 2 | 2,079 |
| 3 | 89,890 |
| 4 | 3,894,594 |
| 5 | 164,075,551 |
| 6 | 6,923,051,137 |

### Verification Notes

- Position 4 perft(6) = 706,045,033 has been specifically verified by multiple engines including Stockfish and DiscoCheck. Off-by-one errors (706,045,032) indicate move generation bugs, commonly in pawn attack generation.
- Position 2 (Kiwipete) perft(5) = 193,690,690 is confirmed by Quick Perft, Fairy-Stockfish test script, and the wiki table. Do NOT confuse with Position 6's perft(5) = 164,075,551.
- Position 5 perft(5) = 89,941,194 is confirmed by the wiki and Fairy-Stockfish. Earlier versions of this wiki page had incorrect values; the current values were corrected by Steven Edwards (July 18, 2015).
- All numbers verified against: Fairy-Stockfish perft.sh script, python-chess tricky.perft test file, H.G. Muller's Quick Perft, and the chessprogramming.org wiki table.

### Additional Perft Test Positions (from TalkChess/Martin Sedlak)

These are useful for catching specific edge cases:

| FEN | Depth | Nodes | Tests |
|-----|-------|-------|-------|
| `3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1` | 6 | 1,134,888 | En passant + discovered check |
| `5k2/8/8/8/8/8/8/4K2R w K - 0 1` | 6 | 661,072 | Castling rights |
| `r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1` | 4 | 1,274,206 | Castle rights |
| `r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1` | 4 | 1,720,476 | Castling prevented |
| `2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1` | 6 | 3,821,001 | Promote out of check |
| `4k3/1P6/8/8/8/8/K7/8 w - - 0 1` | 6 | 217,342 | Promotion |
| `8/P1k5/K7/8/8/8/8/8 w - - 0 1` | 6 | 92,683 | Promotion edge |
| `K1k5/8/P7/8/8/8/8/8 w - - 0 1` | 6 | 2,217 | Promotion stalemate |
| `8/k1P5/8/1K6/8/8/8/8 w - - 0 1` | 7 | 567,584 | Promotion + check |

---

## 5. UCI Protocol (Minimal Subset)

**Source:** https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html, https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf

Communication is via stdin/stdout, one command per line, newline-terminated.

### GUI -> Engine Commands

```
uci
```
Engine must respond with identification and `uciok`:
```
id name MyEngine 1.0
id author My Name
uciok
```

```
isready
```
Engine responds when ready:
```
readyok
```

```
position startpos
position startpos moves e2e4 e7e5 g1f3
position fen <fenstring>
position fen <fenstring> moves <move1> <move2> ...
```
Sets the internal board position. No response required.

```
go depth 6
go movetime 1000
go wtime 300000 btime 300000 winc 2000 binc 2000
go wtime 60000 btime 60000 movestogo 40
go infinite
```
Starts searching. Engine must eventually respond with:
```
bestmove e2e4
```
Optionally before `bestmove`, engine can output info lines:
```
info depth 5 score cp 30 nodes 12345 time 150 pv e2e4 e7e5 g1f3
```

```
stop
```
Stop searching immediately and output `bestmove`.

```
quit
```
Exit the engine process.

### Move Format (Long Algebraic)

- Normal moves: `<from><to>` e.g., `e2e4`, `g1f3`, `b8c6`
- Captures: same format (no 'x'), e.g., `d4e5`
- Castling: king movement, e.g., `e1g1` (white O-O), `e1c1` (white O-O-O), `e8g8` (black O-O), `e8c8` (black O-O-O)
- Promotion: `<from><to><piece>` where piece is lowercase: `e7e8q`, `a2a1n`
- En passant: normal capture notation, e.g., `e5d6`
- Null move: `0000` (used in some analysis modes)

### Example Session

```
GUI:    uci
ENGINE: id name Foundry Chess 0.1.0
ENGINE: id author Lightless Labs
ENGINE: uciok
GUI:    isready
ENGINE: readyok
GUI:    position startpos moves e2e4 e7e5
GUI:    go depth 6
ENGINE: info depth 1 score cp 10 pv d2d4
ENGINE: info depth 2 score cp 15 pv d2d4 d7d5
ENGINE: info depth 3 score cp 20 pv g1f3 b8c6 d2d4
ENGINE: info depth 4 score cp 12 pv g1f3 b8c6 f1b5 a7a6
ENGINE: info depth 5 score cp 18 pv g1f3 b8c6 f1b5 a7a6 b5a4
ENGINE: info depth 6 score cp 22 pv g1f3 b8c6 f1b5 a7a6 b5a4 g8f6
ENGINE: bestmove g1f3
GUI:    quit
```

### Time Management

When receiving `go wtime W btime B`:
- If `movestogo` is given: allocate `time_left / movestogo` per move (with some buffer)
- If no `movestogo` (sudden death): estimate ~30 moves remaining, allocate `time_left / 30`
- Add increment (`winc`/`binc`) to available time
- Always keep a safety margin (e.g., 50ms) to avoid flagging
- Use iterative deepening: complete each depth iteration, check time, stop if insufficient for next depth

---

## 6. Search

### Alpha-Beta with Iterative Deepening

```
function iterative_deepening(position, time_limit):
    best_move = None
    for depth in 1..MAX_DEPTH:
        score = alpha_beta(position, depth, -INF, +INF, true)
        best_move = pv[0]
        print info line
        if time_exceeded:
            break
    return best_move

function alpha_beta(position, depth, alpha, beta, is_pv):
    // Transposition table lookup
    tt_entry = tt_probe(position.hash)
    if tt_entry and tt_entry.depth >= depth:
        if tt_entry.flag == Exact: return tt_entry.score
        if tt_entry.flag == LowerBound: alpha = max(alpha, tt_entry.score)
        if tt_entry.flag == UpperBound: beta = min(beta, tt_entry.score)
        if alpha >= beta: return tt_entry.score

    if depth == 0:
        return quiescence(position, alpha, beta)

    moves = generate_legal_moves(position)
    if moves.is_empty():
        if in_check: return -MATE + ply  // Checkmate
        else: return 0                   // Stalemate

    order_moves(moves)  // Critical for pruning efficiency

    best_score = -INF
    for move in moves:
        make_move(position, move)
        score = -alpha_beta(position, depth - 1, -beta, -alpha, is_pv)
        unmake_move(position, move)

        if score > best_score:
            best_score = score
            if score > alpha:
                alpha = score
                // Update PV
            if alpha >= beta:
                break  // Beta cutoff (pruning)

    // Store in transposition table
    tt_store(position.hash, depth, best_score, flag, best_move)
    return best_score
```

### Quiescence Search

Extends search at leaf nodes by examining only capture moves (and possibly checks) to avoid the "horizon effect" where a position appears calm but has pending captures.

```
function quiescence(position, alpha, beta):
    stand_pat = evaluate(position)
    if stand_pat >= beta: return beta
    if stand_pat > alpha: alpha = stand_pat

    captures = generate_captures(position)
    order_captures(captures)  // MVV-LVA ordering

    for capture in captures:
        make_move(position, capture)
        score = -quiescence(position, -beta, -alpha)
        unmake_move(position, capture)

        if score >= beta: return beta
        if score > alpha: alpha = score

    return alpha
```

### Move Ordering: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)

Score captures by: `victim_value - attacker_value / 100`

Priority table (higher = search first):

| Attacker \ Victim | Pawn | Knight | Bishop | Rook | Queen |
|-------------------|------|--------|--------|------|-------|
| Pawn              | 105  | 205    | 305    | 405  | 505   |
| Knight            | 104  | 204    | 304    | 404  | 504   |
| Bishop            | 103  | 203    | 303    | 403  | 503   |
| Rook              | 102  | 202    | 302    | 402  | 502   |
| Queen             | 101  | 201    | 301    | 401  | 501   |
| King              | 100  | 200    | 300    | 400  | 500   |

Full move ordering priority:
1. TT move (from transposition table) -- highest priority
2. Winning captures (MVV-LVA, where victim > attacker)
3. Equal captures
4. Killer moves (quiet moves that caused beta cutoffs at same depth)
5. History heuristic (quiet moves that caused cutoffs in other positions)
6. Losing captures (where attacker > victim)

---

## 7. Evaluation

### Material Values

**Simplified Evaluation Function** (Tomasz Michniewski, from Chess Programming Wiki):

| Piece | Value (centipawns) |
|-------|--------------------|
| Pawn | 100 |
| Knight | 320 |
| Bishop | 330 |
| Rook | 500 |
| Queen | 900 |
| King | 20000 (sentinel, not used in material balance) |

**PeSTO Tuned Values** (Texel-tuned, from chessprogramming.org):

| Piece | Middlegame | Endgame |
|-------|-----------|---------|
| Pawn | 82 | 94 |
| Knight | 337 | 281 |
| Bishop | 365 | 297 |
| Rook | 477 | 512 |
| Queen | 1025 | 936 |
| King | 0 | 0 |

### Piece-Square Tables: Simplified Evaluation Function (Michniewski)

These are the well-known tables from the Chess Programming Wiki. Values are from WHITE's perspective, with rank 8 at top (index 0-7) and rank 1 at bottom (index 56-63). For black, flip vertically (use index `sq ^ 56` or `63 - sq` depending on mapping).

**Pawn PST:**
```
  0,   0,   0,   0,   0,   0,   0,   0,
 50,  50,  50,  50,  50,  50,  50,  50,
 10,  10,  20,  30,  30,  20,  10,  10,
  5,   5,  10,  25,  25,  10,   5,   5,
  0,   0,   0,  20,  20,   0,   0,   0,
  5,  -5, -10,   0,   0, -10,  -5,   5,
  5,  10,  10, -20, -20,  10,  10,   5,
  0,   0,   0,   0,   0,   0,   0,   0,
```

**Knight PST:**
```
-50, -40, -30, -30, -30, -30, -40, -50,
-40, -20,   0,   0,   0,   0, -20, -40,
-30,   0,  10,  15,  15,  10,   0, -30,
-30,   5,  15,  20,  20,  15,   5, -30,
-30,   0,  15,  20,  20,  15,   0, -30,
-30,   5,  10,  15,  15,  10,   5, -30,
-40, -20,   0,   5,   5,   0, -20, -40,
-50, -40, -30, -30, -30, -30, -40, -50,
```

**Bishop PST:**
```
-20, -10, -10, -10, -10, -10, -10, -20,
-10,   0,   0,   0,   0,   0,   0, -10,
-10,   0,   5,  10,  10,   5,   0, -10,
-10,   5,   5,  10,  10,   5,   5, -10,
-10,   0,  10,  10,  10,  10,   0, -10,
-10,  10,  10,  10,  10,  10,  10, -10,
-10,   5,   0,   0,   0,   0,   5, -10,
-20, -10, -10, -10, -10, -10, -10, -20,
```

**Rook PST:**
```
  0,   0,   0,   0,   0,   0,   0,   0,
  5,  10,  10,  10,  10,  10,  10,   5,
 -5,   0,   0,   0,   0,   0,   0,  -5,
 -5,   0,   0,   0,   0,   0,   0,  -5,
 -5,   0,   0,   0,   0,   0,   0,  -5,
 -5,   0,   0,   0,   0,   0,   0,  -5,
 -5,   0,   0,   0,   0,   0,   0,  -5,
  0,   0,   0,   5,   5,   0,   0,   0,
```

**Queen PST:**
```
-20, -10, -10,  -5,  -5, -10, -10, -20,
-10,   0,   0,   0,   0,   0,   0, -10,
-10,   0,   5,   5,   5,   5,   0, -10,
 -5,   0,   5,   5,   5,   5,   0,  -5,
  0,   0,   5,   5,   5,   5,   0,  -5,
-10,   5,   5,   5,   5,   5,   0, -10,
-10,   0,   5,   0,   0,   0,   0, -10,
-20, -10, -10,  -5,  -5, -10, -10, -20,
```

**King Middlegame PST:**
```
-30, -40, -40, -50, -50, -40, -40, -30,
-30, -40, -40, -50, -50, -40, -40, -30,
-30, -40, -40, -50, -50, -40, -40, -30,
-30, -40, -40, -50, -50, -40, -40, -30,
-20, -30, -30, -40, -40, -30, -30, -20,
-10, -20, -20, -20, -20, -20, -20, -10,
 20,  20,   0,   0,   0,   0,  20,  20,
 20,  30,  10,   0,   0,  10,  30,  20,
```

**King Endgame PST:**
```
-50, -40, -30, -20, -20, -30, -40, -50,
-30, -20, -10,   0,   0, -10, -20, -30,
-30, -10,  20,  30,  30,  20, -10, -30,
-30, -10,  30,  40,  40,  30, -10, -30,
-30, -10,  30,  40,  40,  30, -10, -30,
-30, -10,  20,  30,  30,  20, -10, -30,
-30, -30,   0,   0,   0,   0, -30, -30,
-50, -30, -30, -30, -30, -30, -30, -50,
```

### PeSTO Piece-Square Tables (Texel-Tuned)

These are the empirically tuned tables from the PeSTO evaluation function (Ronald Friederich). Source: chessprogramming.org/PeSTO's_Evaluation_Function. These represent positional BONUS values only (material values are added separately via `mg_value`/`eg_value`).

Orientation: White's perspective, a8=index 0, h1=index 63 (rank 8 first row, rank 1 last row).

**mg_pawn_table:**
```
  0,   0,   0,   0,   0,   0,   0,   0,
 98, 134,  61,  95,  68, 126,  34, -11,
 -6,   7,  26,  31,  65,  56,  25, -20,
-14,  13,   6,  21,  23,  12,  17, -23,
-27,  -2,  -5,  12,  17,   6,  10, -25,
-26,  -4,  -4, -10,   3,   3,  33, -12,
-35,  -1, -20, -23, -15,  24,  38, -22,
  0,   0,   0,   0,   0,   0,   0,   0,
```

**eg_pawn_table:**
```
  0,   0,   0,   0,   0,   0,   0,   0,
178, 173, 158, 134, 147, 132, 165, 187,
 94, 100,  85,  67,  56,  53,  82,  84,
 32,  24,  13,   5,  -2,   4,  17,  17,
 13,   9,  -3,  -7,  -7,  -8,   3,  -1,
  4,   7,  -6,   1,   0,  -5,  -1,  -8,
 13,   8,   8,  10,  13,   0,   2,  -7,
  0,   0,   0,   0,   0,   0,   0,   0,
```

**mg_knight_table:**
```
-167, -89, -34, -49,  61, -97, -15,-107,
 -73, -41,  72,  36,  23,  62,   7, -17,
 -47,  60,  37,  65,  84, 129,  73,  44,
  -9,  17,  19,  53,  37,  69,  18,  22,
 -13,   4,  16,  13,  28,  19,  21,  -8,
 -23,  -9,  12,  10,  19,  17,  25, -16,
 -29, -53, -12,  -3,  -1,  18, -14, -19,
-105, -21, -58, -33, -17, -28, -19, -23,
```

**eg_knight_table:**
```
-58, -38, -13, -28, -31, -27, -63, -99,
-25,  -8, -25,  -2,  -9, -25, -24, -52,
-24, -20,  10,   9,  -1,  -9, -19, -41,
-17,   3,  22,  22,  22,  11,   8, -18,
-18,  -6, -5,  11,   8,  -3,  -6, -22,
-23,  -3,  -1,  15,  10,  -3, -20, -22,
-42, -20, -10,  -5,  -2, -20, -23, -44,
-29, -51, -23, -15,  -22, -18, -50, -64,
```

**mg_bishop_table:**
```
-29,   4, -82, -37, -25,  -42,   7,  -8,
-26,  16, -18, -13,  30,  59,  18, -47,
-16,  37,  43,  40,  35,  50,  37,  -2,
 -4,   5,  19,  50,  37,  37,   7,  -2,
 -6,  13,  13,  26,  34,  12,  10,   4,
  0,  15,  15,  15,  14,  27,  18,  10,
  4,  15,  16,   0,   7,  21,  33,   1,
-33,  -3, -14, -21, -13, -12, -39, -21,
```

**eg_bishop_table:**
```
-14, -21, -11,  -8,  -7,  -9, -17, -24,
 -8,  -4,   7, -12,  -3, -13,  -4, -14,
  2,  -8,   0,  -1,  -2,   6,   0,   4,
 -3,   9,  12,   9,  14,  10,   3,   2,
 -6,   3,  13,  19,   7,  10,  -3,  -9,
-12,  -3,   8,  10,  13,   3,  -7, -15,
-14, -18,  -7,  -1,   4,  -9, -15, -27,
-23,  -9, -23,  -5,  -9, -16,  -5, -17,
```

**mg_rook_table:**
```
 32,  42,  32,  51,  63,   9,  31,  43,
 27,  32,  58,  62,  80,  67,  26,  44,
 -5,  19,  26,  36,  17,  45,  61,  16,
-24, -11,   7,  26,  24,  35,  -8, -20,
-36, -26, -12,  -1,   9,  -7,   6, -23,
-45, -25, -16, -17,   3,   0,  -5, -33,
-44, -16, -20,  -9,  -1,  11,  -6, -71,
-19, -13,   1,  17,  16,   7, -37, -26,
```

**eg_rook_table:**
```
 13,  10,  18,  15,  12,  12,   8,   5,
 11,  13,  13,  11,  -3,   3,   8,   3,
  7,   7,   7,   5,   4,  -3,  -5,  -3,
  4,   3,  13,   1,   2,   1,  -1,   2,
  3,   5,   8,   4,  -5,  -6,  -8, -11,
 -4,   0,  -5,  -1,  -7, -12,  -8, -16,
 -6,  -6,   0,   2,  -9,  -9, -11,  -3,
 -9,   2,   3,  -1,  -5, -13,   4, -20,
```

**mg_queen_table:**
```
-28,   0,  29,  12,  59,  44,  43,  45,
-24, -39,  -5,   1, -16,  57,  28,  54,
-13, -17,   7,   8,  29,  56,  47,  57,
-27, -27, -16, -16,  -1,  17,  -2,   1,
 -9, -26,  -9, -10,  -2,  -4,   3,  -3,
-14,   2, -11,  -2,  -5,   2,  14,   5,
-35,  -8,  11,   2,   8,  15,  -3,   1,
 -1, -18,  -9,  10, -15, -25, -31, -50,
```

**eg_queen_table:**
```
 -9,  22,  22,  27,  27,  19,  10,  20,
-17,  20,  32,  41,  58,  25,  30,   0,
-20,   6,   9,  49,  47,  35,  19,   9,
  3,  22,  24,  45,  57,  40,  57,  36,
-18,  28,  19,  47,  31,  34,  39,  23,
-16, -27,  15,   6,   9,  17,  10,   5,
-22, -23, -30, -16, -16, -23, -36, -32,
-33, -28, -22, -43,  -5, -32, -20, -41,
```

**mg_king_table:**
```
-65,  23,  16, -15, -56, -34,   2,  13,
 29,  -1, -20,  -7,  -8,  -4, -38, -29,
 -9,  24,   2, -16, -20,   6,  22, -22,
-17, -20, -12, -27, -30, -25, -14, -36,
-49,  -1, -27, -39, -46, -44, -33, -51,
-14, -14, -22, -46, -44, -30, -15, -27,
  1,   7,  -8, -64, -43, -16,   9,   8,
-15,  36,  12, -54,   8, -28,  24,  14,
```

**eg_king_table:**
```
-74, -35, -18, -18, -11,  15,   4, -17,
-12,  17,  14,  17,  17,  38,  23,  11,
 10,  17,  23,  15,  20,  45,  44,  13,
 -8,  22,  24,  27,  26,  33,  26,   3,
-18,  -4,  21,  24,  27,  23,   9, -11,
-19,  -3,  11,  21,  23,  16,   7,  -9,
-27, -11,   4,  13,  14,   4,  -5, -17,
-53, -34, -21, -11, -28, -14, -24, -43,
```

### Tapered Evaluation

Interpolate between middlegame and endgame scores based on game phase:

```rust
// Phase weights per piece type (pawns=0, knight=1, bishop=1, rook=2, queen=4)
const PHASE_WEIGHTS: [i32; 6] = [0, 1, 1, 2, 4, 0]; // P, N, B, R, Q, K
const TOTAL_PHASE: i32 = 24; // 4*1 + 4*1 + 4*2 + 2*4 = 24 (all minor/major pieces)

fn tapered_eval(mg_score: i32, eg_score: i32, phase: i32) -> i32 {
    let mg_phase = phase.min(TOTAL_PHASE); // clamp
    let eg_phase = TOTAL_PHASE - mg_phase;
    (mg_score * mg_phase + eg_score * eg_phase) / TOTAL_PHASE
}
```

Phase calculation: sum `PHASE_WEIGHTS[piece_type]` for every piece on the board (both colors). Starting position phase = 24 (pure middlegame). As pieces are captured, phase decreases toward 0 (pure endgame).

### Evaluation Function Structure

```rust
fn evaluate(board: &Board) -> i32 {
    let mut mg_score = [0i32; 2]; // [white, black]
    let mut eg_score = [0i32; 2];
    let mut phase = 0i32;

    for sq in 0..64 {
        if let Some((color, piece)) = board.mailbox[sq] {
            let c = color as usize;
            let p = piece as usize;
            let table_sq = if color == White { sq } else { sq ^ 56 }; // flip for black

            mg_score[c] += MG_VALUE[p] + MG_TABLE[p][table_sq];
            eg_score[c] += EG_VALUE[p] + EG_TABLE[p][table_sq];
            phase += PHASE_WEIGHTS[p];
        }
    }

    let mg = mg_score[side_to_move] - mg_score[opponent];
    let eg = eg_score[side_to_move] - eg_score[opponent];

    tapered_eval(mg, eg, phase)
}
```

---

## Sources

- [Chess Programming Wiki: Perft Results](https://www.chessprogramming.org/Perft_Results)
- [Chess Programming Wiki: PeSTO's Evaluation Function](https://www.chessprogramming.org/PeSTO's_Evaluation_Function)
- [Chess Programming Wiki: Simplified Evaluation Function](https://www.chessprogramming.org/Simplified_Evaluation_Function)
- [Chess Programming Wiki: Forsyth-Edwards Notation](https://www.chessprogramming.org/Forsyth-Edwards_Notation)
- [Chess Programming Wiki: Piece-Square Tables](https://www.chessprogramming.org/Piece-Square_Tables)
- [Chess Programming Wiki: Tapered Eval](https://www.chessprogramming.org/Tapered_Eval)
- [Stockfish UCI Documentation](https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html)
- [UCI Protocol Specification (Gist)](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf)
- [FEN Specification](https://kirill-kryukov.com/chess/doc/fen.html)
- [Rustic Chess Engine (Rust tutorial)](https://rustic-chess.org/evaluation/psqt.html)
- [Fairy-Stockfish Perft Test Script](https://github.com/fairy-stockfish/Fairy-Stockfish/blob/master/tests/perft.sh)
- [python-chess Perft Test Vectors](https://github.com/niklasf/python-chess/blob/master/examples/perft/tricky.perft)
- [Perfect Perft (chessprogramming.net)](https://www.chessprogramming.net/perfect-perft/)
- [TalkChess: Perft Statistics Discussion](https://talkchess.com/viewtopic.php?t=78402)
- [Open-Chess: Perft off-by-one debugging](https://open-chess.org/viewtopic.php?t=2201)
- [Mediocre Chess: Tapered Eval Guide](http://mediocrechess.blogspot.com/2011/10/guide-tapered-eval.html)
- [PeSTO / RofChade (Ronald Friederich)](https://rofchade.nl/?p=307)
