# Example: Chess Engine — Golden Test Vectors Fix the Convention Problem

This directory contains a chess engine built using the Foundry adversarial workflow. It demonstrates how **golden test vectors** (perft numbers) prevent the convention mismatch that broke the Rubik's cube example.

## What Happened

```
research → spec → NLSpec (with perft golden vectors) → red team (44 tests) → green team implements → 36/44 → fix NLSpec bugs → 44/44
```

The golden test vectors (perft numbers from chessprogramming.org) worked exactly as intended for 4 of 5 test positions. The 5th position had a **spec derivation bug** — the NLSpec agent used the wrong FEN for Position 4 but kept the perft numbers from a different position. This was caught immediately because the numbers didn't match.

The remaining failures were red team test bugs: an impossible mate-in-one position, a non-stalemate claimed as stalemate, and a wrong move count. All were fixable without breaking the information barrier.

## The Lesson: Golden Vectors Work

| Example | Golden Vectors? | Convention Mismatch? | Final Result |
|---------|----------------|---------------------|-------------|
| Sudoku | N/A (no convention ambiguity) | No | 30/30 |
| Rubik's cube | Missing | Yes — fatal, 15 tests unfixable | 31/46 |
| Chess | Present (perft numbers) | Caught and fixed by vectors | 44/44 |

Perft numbers are the ideal golden test vector: they're exact, well-established, independently verifiable, and catch ANY move generation bug. When the NLSpec had the wrong FEN for Position 4, the mismatch between the solver's perft(1)=6 and the test's expected perft(1)=26 was immediately obvious.

## What Was Built

A complete chess engine in Rust (~1800 lines) with:
- Bitboard board representation (12 × u64)
- FEN parsing and output
- Full legal move generation (all piece types, castling, en passant, promotion)
- Perft for move generation validation
- Alpha-beta search with iterative deepening and quiescence search
- PeSTO tapered evaluation (middlegame/endgame piece-square tables)
- UCI protocol (uci, isready, position, go depth/movetime, bestmove, quit)
- Transposition table with Zobrist hashing

## Artifacts

| Phase | Artifact | Description |
|-------|----------|-------------|
| **Research** | [`docs/research/`](docs/research/) | Bitboards, FEN, perft numbers (verified from 3+ sources), UCI, search, evaluation |
| **Spec** | [`docs/specs/`](docs/specs/) | 8 requirements, 5 behaviors |
| **NLSpec** | [`docs/nlspecs/`](docs/nlspecs/) | With golden perft vectors in the DoD |
| **Red Team Tests** | [`red/tests/`](red/tests/) | 44 integration tests (22 perft, 7 FEN, 5 UCI, 6 edge cases, 2 search, 1 smoke) |
| **Green Team** | [`green/src/`](green/src/) | Full chess engine, single file |

## Running It

```bash
# Perft (move generation validation)
cargo run -- perft 5
# → 4865609

# Perft with custom position (Kiwipete)
cargo run -- perft 4 --fen "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"
# → 4085603

# UCI mode
cargo run -- uci
# Then: position startpos moves e2e4 e7e5
#        go depth 6
#        quit

# Run tests
cargo test
```

## The Three Examples Together

| Example | Difficulty | Adversarial Result | Key Lesson |
|---------|-----------|-------------------|------------|
| **Sudoku** | Easy | 30/30 clean pass | The workflow works for well-defined constraint problems |
| **Rubik's cube** | Hard | 31/46 (convention mismatch) | NLSpecs MUST include golden test vectors for geometric conventions |
| **Chess** | Hard | 44/44 (after NLSpec fix) | Golden vectors catch both convention errors and spec derivation bugs |
