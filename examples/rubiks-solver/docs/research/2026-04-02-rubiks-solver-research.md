# Rubik's Cube Solver Research

**Date:** 2026-04-02
**Purpose:** Technical research for implementing a CLI Rubik's cube solver in Rust
**Target algorithm:** Kociemba two-phase

---

## Table of Contents

1. [Cube Representation](#1-cube-representation)
2. [Move Notation](#2-move-notation)
3. [Algorithm Comparison](#3-algorithm-comparison)
4. [Kociemba Two-Phase Algorithm](#4-kociemba-two-phase-algorithm)
5. [Input/Output Conventions](#5-inputoutput-conventions)
6. [Validation](#6-validation)
7. [Existing Rust Implementations](#7-existing-rust-implementations)
8. [Implementation Recommendations](#8-implementation-recommendations)

---

## 1. Cube Representation

Three main representations exist, each suited for a different purpose. A practical solver uses all three: facelets for I/O, cubies for move application, and coordinates for the search algorithm.

### 1.1 Facelet Representation (54 stickers)

The cube has 6 faces with 9 facelets each = 54 stickers. Each sticker stores a color (or face identity, 0-5).

```
Face indexing (standard Kociemba ordering):
  U: facelets  0- 8   (Up)
  R: facelets  9-17   (Right)
  F: facelets 18-26   (Front)
  D: facelets 27-35   (Down)
  L: facelets 36-44   (Left)
  B: facelets 45-53   (Back)

Per-face layout (reading order, top-left to bottom-right):
  +---+---+---+
  | 0 | 1 | 2 |
  +---+---+---+
  | 3 | 4 | 5 |    (facelet 4 is the center)
  +---+---+---+
  | 6 | 7 | 8 |
  +---+---+---+

Center facelets (fixed, define face identity):
  U=4, R=13, F=22, D=31, L=40, B=49
```

**Use case:** User input/output. The solved cube as a facelet string is:
```
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

**Rust representation:**
```rust
type Facelet = [u8; 54]; // each element 0-5 representing U/R/F/D/L/B
```

### 1.2 Cubie Representation (8 corners + 12 edges)

The cube has 20 movable cubies: 8 corners (3 stickers each) and 12 edges (2 stickers each). 6 centers are fixed.

Each cubie has:
- **Position:** which slot it occupies (corner: 0-7, edge: 0-11)
- **Orientation:** rotational state relative to solved

#### Corner numbering

| Index | Name | Position |
|-------|------|----------|
| 0 | URF | Up-Right-Front |
| 1 | UFL | Up-Front-Left |
| 2 | ULB | Up-Left-Back |
| 3 | UBR | Up-Back-Right |
| 4 | DFR | Down-Front-Right |
| 5 | DLF | Down-Left-Front |
| 6 | DBL | Down-Back-Left |
| 7 | DRB | Down-Right-Back |

**Orientation:** 0 = correct, 1 = clockwise twist, 2 = counterclockwise twist (relative to the U/D sticker position).

#### Edge numbering

| Index | Name | Position |
|-------|------|----------|
| 0 | UR | Up-Right |
| 1 | UF | Up-Front |
| 2 | UL | Up-Left |
| 3 | UB | Up-Back |
| 4 | DR | Down-Right |
| 5 | DF | Down-Front |
| 6 | DL | Down-Left |
| 7 | DB | Down-Back |
| 8 | FR | Front-Right |
| 9 | FL | Front-Left |
| 10 | BL | Back-Left |
| 11 | BR | Back-Right |

**Orientation:** 0 = correct, 1 = flipped.

**Note:** Edges 8-11 (FR, FL, BL, BR) are the "UD-slice" edges -- the 4 edges in the middle layer between U and D. This is critical for the Kociemba algorithm.

**Rust representation:**
```rust
struct CubieCube {
    corner_perm: [u8; 8],    // which corner is in each slot (0-7)
    corner_orient: [u8; 8],  // orientation of corner in each slot (0-2)
    edge_perm: [u8; 12],     // which edge is in each slot (0-11)
    edge_orient: [u8; 12],   // orientation of edge in each slot (0-1)
}
```

### 1.3 Coordinate Representation

The coordinate representation maps the cubie state to small integers suitable for indexing into lookup tables. This is what the Kociemba search algorithm operates on.

| Coordinate | Phase | Range | Count | Encoding |
|------------|-------|-------|-------|----------|
| Corner Orientation (CO) | 1 | 0-2,186 | 2,187 = 3^7 | Base-3, 7 independent corners |
| Edge Orientation (EO) | 1 | 0-2,047 | 2,048 = 2^11 | Base-2, 11 independent edges |
| UD Slice (combination) | 1 | 0-494 | 495 = C(12,4) | Combinadic of 4 slice edge positions |
| Corner Permutation (CP) | 2 | 0-40,319 | 40,320 = 8! | Lehmer code |
| U/D Edge Permutation (EP) | 2 | 0-40,319 | 40,320 = 8! | Lehmer code of 8 non-slice edges |
| UD Slice Permutation (USP) | 2 | 0-23 | 24 = 4! | Lehmer code of 4 slice edges |

---

## 2. Move Notation

### 2.1 Singmaster Notation

Standard notation uses 6 face letters with optional modifiers:

| Face | Letter | Direction |
|------|--------|-----------|
| Up | U | Clockwise when looking at U face |
| Down | D | Clockwise when looking at D face |
| Right | R | Clockwise when looking at R face |
| Left | L | Clockwise when looking at L face |
| Front | F | Clockwise when looking at F face |
| Back | B | Clockwise when looking at B face |

Modifiers:
- (none) = 90 degrees clockwise: `U`
- `'` (prime) = 90 degrees counterclockwise (= 270 degrees clockwise): `U'`
- `2` = 180 degrees: `U2`

This gives **18 moves total** (6 faces x 3 variants).

### 2.2 Internal Move Encoding

```rust
// 18 moves, grouped by face (3 per face)
// move_id / 3 = face index (0=U, 1=D, 2=R, 3=L, 4=F, 5=B)
// move_id % 3 = variant (0=CW, 1=CCW, 2=double)
enum Face { U=0, D=1, R=2, L=3, F=4, B=5 }

const MOVE_NAMES: [&str; 18] = [
    "U", "U'", "U2",   // 0, 1, 2
    "D", "D'", "D2",   // 3, 4, 5
    "R", "R'", "R2",   // 6, 7, 8
    "L", "L'", "L2",   // 9, 10, 11
    "F", "F'", "F2",   // 12, 13, 14
    "B", "B'", "B2",   // 15, 16, 17
];
```

### 2.3 Move Effects on Cubies

Each 90-degree clockwise face turn produces a 4-cycle on corners and a 4-cycle on edges. Below are the permutation cycles and orientation changes for each basic move.

**Corner cycles** (cycle notation: a -> b -> c -> d -> a):

| Move | Corner Cycle | Orientation Deltas |
|------|-------------|-------------------|
| U | (0 3 2 1) = URF->UBR->ULB->UFL | all 0 (no twist) |
| D | (4 5 6 7) = DFR->DLF->DBL->DRB | all 0 (no twist) |
| R | (0 4 7 3) = URF->DFR->DRB->UBR | +1, +2, +1, +2 |
| L | (1 2 6 5) = UFL->ULB->DBL->DLF | +1, +2, +1, +2 |
| F | (0 1 5 4) = URF->UFL->DLF->DFR | +1, +2, +1, +2 |
| B | (3 7 6 2) = UBR->DRB->DBL->ULB | +1, +2, +1, +2 |

**Note:** U and D moves do not change corner orientation. R, L, F, B each twist corners by alternating +1/+2 (mod 3) around the cycle.

**Edge cycles:**

| Move | Edge Cycle | Orientation Deltas |
|------|-----------|-------------------|
| U | (0 3 2 1) = UR->UB->UL->UF | all 0 (no flip) |
| D | (4 7 6 5) = DR->DB->DL->DF | all 0 (no flip) |
| R | (0 8 4 11) = UR->FR->DR->BR | all 0 (no flip) |
| L | (2 9 6 10) = UL->FL->DL->BL | all 0 (no flip) |
| F | (1 9 5 8) = UF->FL->DF->FR | all +1 (flip) |
| B | (3 10 7 11) = UB->BL->DB->BR | all +1 (flip) |

**Note:** Only F and B moves flip edge orientations. This is a key property: edge orientation is defined relative to the "F/B axis."

---

## 3. Algorithm Comparison

### 3.1 Overview

| Algorithm | Phases | Typical Moves | Optimal? | Table Size | Solve Time |
|-----------|--------|---------------|----------|------------|------------|
| Beginner's layer-by-layer | 5-7 | 100-150 | No | None | Instant |
| Thistlethwaite 4-phase | 4 | ~45 (QTM) | No | Small | Fast |
| **Kociemba two-phase** | **2** | **18-25** | **Near-optimal** | **10-200 MB** | **ms** |
| Korf IDA* | 1 | <=20 (optimal) | Yes | >1 GB | Seconds-minutes |

### 3.2 Beginner's Layer-by-Layer

Solves in 5 stages:
1. **White cross** (4 edges on bottom)
2. **First-layer corners** (4 corners)
3. **Second layer** (4 middle edges)
4. **OLL** (Orient Last Layer -- 57 cases, or 2-look with ~10 algorithms)
5. **PLL** (Permute Last Layer -- 21 cases, or 2-look with ~6 algorithms)

Typical: 100-150 moves. Simple to implement but far from optimal. Not suitable for a competitive solver.

### 3.3 Thistlethwaite's 4-Phase Algorithm

Reduces the cube through 4 nested subgroups:

| Phase | Group Transition | Max Moves | Allowed Moves |
|-------|-----------------|-----------|---------------|
| 1 | G0 -> G1 | 7 | All 18 |
| 2 | G1 -> G2 | 10 | U,D,R2,L2,F2,B2 (10) |
| 3 | G2 -> G3 | 13 | U2,D2,R2,L2,F2,B2 (6) |
| 4 | G3 -> Solved | 15 | U2,D2,R2,L2,F2,B2 (6) |

Total: <= 45 moves (QTM). Historically significant (1981) but superseded by Kociemba.

### 3.4 Kociemba Two-Phase (Target Algorithm)

See Section 4 for full details. Key advantages:
- 2 phases instead of 4 (simpler, shorter solutions)
- Typically 18-25 moves (vs Thistlethwaite's ~45)
- Near-optimal with solution improvement loop
- Practical table sizes (~10-200 MB)
- Millisecond solve times

### 3.5 Korf's IDA* Optimal Solver

Finds provably optimal solutions (minimum moves = God's number for that state).

**Pattern databases used:**
- Corner pattern database: all 8 corners (88,179,840 states, ~80 MB)
- Edge pattern databases: typically split into two groups of 6 edges
  - Each: C(12,6) x 6! x 2^6 states (~42 MB each)
- Total: ~160 MB - 1+ GB depending on implementation

**Comparison with Kociemba:**
- Always finds <=20-move solutions (God's number proven in 2010)
- Much slower: seconds to minutes per solve vs milliseconds
- Larger tables
- For a CLI tool, Kociemba is the pragmatic choice

### 3.6 God's Number

- **HTM (Half-Turn Metric):** 20 moves (proven 2010 by Rokicki et al.)
  - Every cube position can be solved in at most 20 moves
  - 90-degree and 180-degree turns each count as 1 move
- **QTM (Quarter-Turn Metric):** 26 moves (proven 2014)
  - Only 90-degree turns count; 180-degree = 2 moves

---

## 4. Kociemba Two-Phase Algorithm

### 4.1 Core Idea

The full Rubik's cube group **G0** = <U, D, R, L, F, B> has ~4.33 x 10^19 elements.

**Phase 1** reduces the cube to the subgroup **G1** = <U, D, R2, L2, F2, B2>, which has ~19.5 billion elements. In G1:
- All corner orientations are 0 (solved)
- All edge orientations are 0 (solved)
- The 4 UD-slice edges (FR, FL, BL, BR) are in the middle layer (though not necessarily in the right order)

**Phase 2** solves within G1 to reach the identity (solved cube). Only G1-preserving moves are used: U, U', U2, D, D', D2, R2, L2, F2, B2 (10 moves).

### 4.2 Phase 1 Coordinates

Three coordinates describe the distance from G1:

#### Corner Orientation (CO): 0-2,186

Encodes the twist state of all 8 corners. Only 7 are independent (the 8th is determined by the constraint that the sum of all corner orientations must be divisible by 3).

**Encoding:** Base-3 number from 7 corner orientations.
```
co_index = co[0]*3^6 + co[1]*3^5 + co[2]*3^4 + co[3]*3^3
         + co[4]*3^2 + co[5]*3^1 + co[6]*3^0
```
Range: 0 to 3^7 - 1 = 2,186.

**Goal state:** co_index = 0 (all orientations are 0).

#### Edge Orientation (EO): 0-2,047

Encodes the flip state of all 12 edges. Only 11 are independent (the 12th is determined by the constraint that the sum of all edge orientations must be even).

**Encoding:** Base-2 number from 11 edge orientations.
```
eo_index = eo[0]*2^10 + eo[1]*2^9 + ... + eo[10]*2^0
```
Range: 0 to 2^11 - 1 = 2,047.

**Goal state:** eo_index = 0 (all orientations are 0).

#### UD Slice (UDS): 0-494

Tracks which 4 of the 12 edge positions contain UD-slice edges (FR, FL, BL, BR). Does NOT track their order within the slice, only their positions.

**Encoding:** Combinatorial number system (combinadic). Choose 4 positions from 12: C(12,4) = 495 combinations.

```
// Given 4 positions p0 < p1 < p2 < p3 (sorted):
uds_index = C(p0, 1) + C(p1, 2) + C(p2, 3) + C(p3, 4)
// where C(n,k) = binomial coefficient
```
Range: 0 to 494.

**Goal state:** uds_index = 0 (slice edges in positions 8, 9, 10, 11).

#### Phase 1 State Space

Total: 2,187 x 2,048 x 495 = **2,217,093,120** states (~2.2 billion).

#### Phase 1 Allowed Moves

All 18 moves (U, U', U2, D, D', D2, R, R', R2, L, L', L2, F, F', F2, B, B', B2).

### 4.3 Phase 2 Coordinates

Three coordinates describe the distance from the solved state within G1:

#### Corner Permutation (CP): 0-40,319

Encodes the permutation of all 8 corners using the Lehmer code (factorial number system).

```
// Lehmer code: for each position i, count how many values
// after position i are smaller than the value at position i.
// Then encode as: d[0]*7! + d[1]*6! + ... + d[6]*1! + d[7]*0!
```
Range: 0 to 8! - 1 = 40,319.

#### U/D Edge Permutation (EP): 0-40,319

Encodes the permutation of the 8 non-slice edges (UR, UF, UL, UB, DR, DF, DL, DB) using the Lehmer code.

Range: 0 to 8! - 1 = 40,319.

**Note:** In phase 2, the 4 slice edges are already in the middle layer, so the 8 U/D edges form an independent permutation.

#### UD Slice Sorted Permutation (USP): 0-23

Encodes the permutation of the 4 UD-slice edges within their 4 positions.

Range: 0 to 4! - 1 = 23.

#### Phase 2 State Space

Total: 40,320 x 40,320 x 24 = **39,030,374,400** states (~39 billion).

However, permutation parity constraints reduce this: corner parity must equal edge parity, effectively halving the reachable space.

#### Phase 2 Allowed Moves

Only G1-preserving moves: **U, U', U2, D, D', D2, R2, L2, F2, B2** (10 moves).

Quarter turns of R, L, F, B are forbidden because they would change corner/edge orientations and move edges out of the UD slice.

### 4.4 Move Tables

Move tables precompute the effect of each move on each coordinate value, enabling O(1) state transitions during search.

**Structure:** For each coordinate and each move, store the resulting coordinate value.

```rust
// Phase 1 move tables
co_move: [[u16; 18]; 2187]    // co_move[co_idx][move_id] = new_co_idx
eo_move: [[u16; 18]; 2048]    // eo_move[eo_idx][move_id] = new_eo_idx
uds_move: [[u16; 18]; 495]    // uds_move[uds_idx][move_id] = new_uds_idx

// Phase 2 move tables
cp_move: [[u16; 10]; 40320]   // cp_move[cp_idx][move_id] = new_cp_idx
ep_move: [[u16; 10]; 40320]   // ep_move[ep_idx][move_id] = new_ep_idx
usp_move: [[u8; 10]; 24]      // usp_move[usp_idx][move_id] = new_usp_idx
```

**Generation:** For each coordinate value, decode to cubie state, apply the move, re-encode. This is done once at startup or precomputed to disk.

**Sizes:**
- Phase 1: 2,187 x 18 x 2 + 2,048 x 18 x 2 + 495 x 18 x 2 = ~170 KB
- Phase 2: 40,320 x 10 x 2 + 40,320 x 10 x 2 + 24 x 10 x 1 = ~1.6 MB
- Total move tables: ~1.8 MB

### 4.5 Pruning Tables

Pruning tables store the minimum number of moves needed to reach the goal state from each coordinate combination. They provide the heuristic `h(state)` for IDA*.

#### Practical Approach: Split Pruning Tables

Storing the full combined coordinate space is impractical. Instead, use **pairs of coordinates** and take the maximum of two heuristics.

**Phase 1 pruning tables (two sub-tables):**

| Table | Dimensions | Entries | Size (4 bits each) |
|-------|-----------|---------|---------------------|
| CO x UDSlice | 2,187 x 495 | 1,082,565 | ~528 KB |
| EO x UDSlice | 2,048 x 495 | 1,013,760 | ~494 KB |

Heuristic: `h1 = max(co_uds_prune[co][uds], eo_uds_prune[eo][uds])`

**Phase 2 pruning tables (two sub-tables):**

| Table | Dimensions | Entries | Size (4 bits each) |
|-------|-----------|---------|---------------------|
| CP x USP | 40,320 x 24 | 967,680 | ~472 KB |
| EP x USP | 40,320 x 24 | 967,680 | ~472 KB |

Heuristic: `h2 = max(cp_usp_prune[cp][usp], ep_usp_prune[ep][usp])`

**Total pruning table memory: ~2 MB** (without symmetry reduction).

#### Advanced: Symmetry-Reduced FlipUDSlice Coordinate

For better pruning, combine EO (2,048) and UDSlice (495) into a single "FlipUDSlice" coordinate:
- Raw values: 2,048 x 495 = 1,013,760
- The cube has 16 symmetries that preserve the UD axis
- Symmetry reduction: 1,013,760 / 16 ~ **64,430 equivalence classes**
- Paired with CO (2,187): 2,187 x 64,430 = ~140 million entries
- At 4 bits each: ~67 MB (or ~1.6 bits/entry with mod-3 compression: ~27 MB)

This gives stronger pruning but requires more memory and symmetry conjugation logic.

#### Generation via BFS

Pruning tables are generated by backward BFS from the goal state:

```
1. Initialize all entries to "unvisited" (e.g., 0xFF)
2. Set goal state entry to 0
3. For depth d = 0, 1, 2, ...:
   a. For each state with value d:
      - Apply all inverse moves
      - For each predecessor state that is unvisited:
        - Set its value to d + 1
   b. Stop when all states are filled
```

**Generation time (approximate):**
- Phase 1 split tables (~1M entries each): < 1 second
- Phase 2 split tables (~1M entries each): < 1 second
- Symmetry-reduced FlipUDSlice table (~140M entries): 5-30 seconds

### 4.6 IDA* Search

IDA* (Iterative Deepening A*) is a depth-first search with iteratively increasing depth limits, pruned by the heuristic from the pruning tables.

#### Algorithm

```
function ida_star(initial_state):
    for depth_limit = heuristic(initial_state) to MAX_DEPTH:
        result = dfs(initial_state, 0, depth_limit, [])
        if result is Some(solution):
            return solution
    return None

function dfs(state, depth, limit, path):
    h = heuristic(state)
    if depth + h > limit:
        return None          // prune: can't reach goal within limit
    if state == goal:
        return Some(path)    // found solution
    for each move m in allowed_moves:
        if is_redundant(path, m):
            continue         // skip redundant moves
        new_state = apply_move(state, m)
        result = dfs(new_state, depth + 1, limit, path + [m])
        if result is Some:
            return result
    return None
```

#### Move Redundancy Pruning

To avoid exploring redundant move sequences:

1. **Same face:** Never apply the same face twice consecutively (e.g., U U -> U2 or identity).
   ```
   if last_face == current_face: skip
   ```

2. **Opposite faces:** For commuting pairs (U/D, R/L, F/B), enforce a canonical order. If we just did D and now want U, skip (we would have done U first).
   ```
   if are_opposite(last_face, current_face) and last_face > current_face: skip
   ```

This reduces the branching factor from 18 to ~15 on average.

### 4.7 Two-Phase Coordination

The phases are run sequentially, but with an important optimization: the algorithm searches for **multiple phase-1 solutions** of increasing length and picks the one that yields the shortest total (phase 1 + phase 2) solution.

```
function solve(cube):
    best_solution = None
    best_length = MAX_TOTAL

    for phase1_limit = h1(cube) to min(12, best_length - 1):
        for each phase1_solution of length phase1_limit:
            cube_after_p1 = apply_moves(cube, phase1_solution)
            phase2_state = extract_phase2_coords(cube_after_p1)
            phase2_limit = best_length - phase1_limit - 1

            phase2_solution = ida_star_phase2(phase2_state, phase2_limit)
            if phase2_solution is Some:
                total = phase1_solution + phase2_solution
                if total.len() < best_length:
                    best_solution = total
                    best_length = total.len()

    return best_solution
```

### 4.8 Typical Performance

| Metric | Value |
|--------|-------|
| Average solution length | 18-22 moves (HTM) |
| Phase 1 average | 7-10 moves |
| Phase 2 average | 10-13 moves |
| Worst case (with optimization loop) | ~25 moves |
| Solve time (with precomputed tables) | 1-50 ms |
| Table generation time | 1-30 seconds |
| Total memory | 2-200 MB (depending on table strategy) |

---

## 5. Input/Output Conventions

### 5.1 Facelet String (Kociemba format)

The standard input format is a **54-character string** using the letters U, R, F, D, L, B. Each letter represents the color of the center it matches (not the literal color, but the face identity).

**Ordering:** U face (9 chars) + R face (9) + F face (9) + D face (9) + L face (9) + B face (9).

```
Solved: UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB

Example scrambled:
DRLUUBFBL DRUFRRFUU BRDFFBDLL UFDRDDBLF LBRLLFRUDR BBUUBRLUD
(spaces added for readability; actual string has no spaces)
```

**Facelet map (unfolded cube):**

```
             +---+---+---+
             | U0| U1| U2|
             +---+---+---+
             | U3| U4| U5|
             +---+---+---+
             | U6| U7| U8|
  +---+---+---+---+---+---+---+---+---+---+---+---+
  |L36|L37|L38|F18|F19|F20|R9 |R10|R11|B45|B46|B47|
  +---+---+---+---+---+---+---+---+---+---+---+---+
  |L39|L40|L41|F21|F22|F23|R12|R13|R14|B48|B49|B50|
  +---+---+---+---+---+---+---+---+---+---+---+---+
  |L42|L43|L44|F24|F25|F26|R15|R16|R17|B51|B52|B53|
  +---+---+---+---+---+---+---+---+---+---+---+---+
             |D27|D28|D29|
             +---+---+---+
             |D30|D31|D32|
             +---+---+---+
             |D33|D34|D35|
             +---+---+---+
```

### 5.2 Scramble Notation (Move Sequence)

A scramble is a sequence of moves applied to a solved cube:
```
R U R' U' F2 D L2 B' R2 U F' D2 L B2 R' F U2 D' B L2
```

Moves are space-separated. This is the standard format used by:
- WCA (World Cube Association) competitions
- Online timers and scramblers
- Most solver tools

### 5.3 Color String

Some tools accept a string of actual color characters:
```
W = White, Y = Yellow, R = Red, O = Orange, G = Green, B = Blue
```

The standard Western/BOY color scheme maps:
| Face | Color |
|------|-------|
| U | White |
| D | Yellow |
| F | Green |
| B | Blue |
| R | Red |
| L | Orange |

### 5.4 Output Format

Solutions are output as space-separated move sequences:
```
R2 U F' D2 B L' R U2 D B2 F R' L D2 U' F2 B' R L2 D
```

For CLI output, additional useful information:
- Move count
- Phase 1/Phase 2 split point
- Solve time

---

## 6. Validation

A cube state must pass these checks to be solvable. Of the 12^12 x 8^8 possible sticker arrangements, only 1/12 are reachable from the solved state.

### 6.1 Sticker Count

Each of the 6 colors must appear exactly 9 times.

```rust
fn validate_sticker_counts(facelets: &[u8; 54]) -> Result<(), String> {
    let mut counts = [0u8; 6];
    for &f in facelets {
        if f >= 6 {
            return Err(format!("Invalid facelet color: {}", f));
        }
        counts[f as usize] += 1;
    }
    for (i, &count) in counts.iter().enumerate() {
        if count != 9 {
            return Err(format!("Color {} appears {} times, expected 9", i, count));
        }
    }
    Ok(())
}
```

### 6.2 Center Facelets

The 6 center facelets (positions 4, 13, 22, 31, 40, 49) must each show a different color. Centers are fixed and define face identity.

```rust
fn validate_centers(facelets: &[u8; 54]) -> Result<(), String> {
    let centers = [facelets[4], facelets[13], facelets[22],
                   facelets[31], facelets[40], facelets[49]];
    let mut seen = [false; 6];
    for &c in &centers {
        if seen[c as usize] {
            return Err(format!("Duplicate center color: {}", c));
        }
        seen[c as usize] = true;
    }
    Ok(())
}
```

### 6.3 Corner Orientation Parity

The sum of all 8 corner orientations must be divisible by 3.

**Why:** Each face turn changes 4 corner orientations such that the sum change is always 0 (mod 3). A single twisted corner is physically impossible.

```rust
fn validate_corner_orientation(cube: &CubieCube) -> Result<(), String> {
    let sum: u32 = cube.corner_orient.iter().map(|&o| o as u32).sum();
    if sum % 3 != 0 {
        return Err(format!(
            "Corner orientation sum {} is not divisible by 3", sum
        ));
    }
    Ok(())
}
```

### 6.4 Edge Orientation Parity

The sum of all 12 edge orientations must be even (divisible by 2).

**Why:** Each face turn flips either 0 or 4 edges (F and B flip 4; U, D, R, L flip 0). The parity is always preserved.

```rust
fn validate_edge_orientation(cube: &CubieCube) -> Result<(), String> {
    let sum: u32 = cube.edge_orient.iter().map(|&o| o as u32).sum();
    if sum % 2 != 0 {
        return Err(format!(
            "Edge orientation sum {} is not even", sum
        ));
    }
    Ok(())
}
```

### 6.5 Permutation Parity

The permutation parity (even/odd) of the corners must equal the permutation parity of the edges.

**Why:** Each face turn is an even permutation on corners AND an even permutation on edges. So both parities must always match.

```rust
fn permutation_parity(perm: &[u8]) -> bool {
    // Returns true if even permutation, false if odd
    let n = perm.len();
    let mut visited = vec![false; n];
    let mut even = true;

    for i in 0..n {
        if !visited[i] {
            let mut j = i;
            let mut cycle_len = 0;
            while !visited[j] {
                visited[j] = true;
                j = perm[j] as usize;
                cycle_len += 1;
            }
            if cycle_len % 2 == 0 {
                even = !even; // even-length cycles are odd permutations
            }
        }
    }
    even
}

fn validate_permutation_parity(cube: &CubieCube) -> Result<(), String> {
    let cp = permutation_parity(&cube.corner_perm);
    let ep = permutation_parity(&cube.edge_perm);
    if cp != ep {
        return Err(
            "Corner and edge permutation parities do not match".to_string()
        );
    }
    Ok(())
}
```

### 6.6 Uniqueness

Each corner must appear exactly once (positions 0-7), and each edge must appear exactly once (positions 0-11).

### 6.7 Summary of Solvability Constraints

| Constraint | Mathematical Rule | Factor |
|-----------|-------------------|--------|
| Corner orientation | Sum of 8 orientations = 0 (mod 3) | x3 |
| Edge orientation | Sum of 12 orientations = 0 (mod 2) | x2 |
| Permutation parity | sign(corner_perm) = sign(edge_perm) | x2 |

These three constraints together mean that only 1 in 12 of all possible cubie configurations are actually reachable from the solved state by legal moves. Total reachable positions: 8! x 3^7 x 12! x 2^11 / 12 = 43,252,003,274,489,856,000 (~4.33 x 10^19).

---

## 7. Existing Rust Implementations

### 7.1 kewb

- **Repository:** https://github.com/luckasRanarison/kewb
- **Crate:** https://crates.io/crates/kewb (v0.4.2, May 2024)
- **Algorithm:** Kociemba two-phase
- **Features:** Library + CLI, table generation, cube manipulation
- **Notes:** Work-in-progress for efficiency. Good reference for data structures and API design.

### 7.2 kociema

- **Repository:** https://github.com/adungaos/kociema
- **Algorithm:** Kociemba two-phase (port of Kociemba's reference implementation)
- **Features:** Averages under 19 moves for 3x3 solves.

### 7.3 TwoPhaseSolver

- **Repository:** https://github.com/tremwil/TwoPhaseSolver
- **Algorithm:** Two-phase with IDA*-like search
- **Features:** Pruning tables up to depth 12 (phase 1) and 12/18 (phase 2).

### 7.4 cubesim

- **Repository:** https://lib.rs/crates/cubesim
- **Algorithm:** Thistlethwaite's (planned: Kociemba)
- **Features:** NxNxN cube simulation.

### 7.5 Rusty-Rubik

- **Repository:** https://github.com/esqu1/Rusty-Rubik
- **Algorithm:** Optimal solver
- **Notes:** Pruning table generation takes ~10-20 min on modern CPUs.

### 7.6 Reference Implementations (Non-Rust)

- **Herbert Kociemba's Python solver:** https://github.com/hkociemba/RubiksCube-TwophaseSolver
  - The canonical reference implementation by the algorithm's author.
  - Excellent for understanding the algorithm; Python code is readable.
- **Kociemba's website:** https://kociemba.org/math/twophase.htm
  - Detailed mathematical explanations of all coordinates, move tables, and pruning tables.

---

## 8. Implementation Recommendations

### 8.1 Architecture

```
rubiks-solver/
  src/
    main.rs            -- CLI entry point (clap)
    cube/
      facelet.rs       -- Facelet representation, parsing, display
      cubie.rs         -- CubieCube struct, move application
      coord.rs         -- Coordinate encoding/decoding
      moves.rs         -- Move definitions (permutation cycles, orientations)
      validate.rs      -- Solvability validation
    solver/
      tables.rs        -- Move table and pruning table generation/loading
      phase1.rs        -- Phase 1 IDA* search
      phase2.rs        -- Phase 2 IDA* search
      kociemba.rs      -- Two-phase coordination
    io/
      parse.rs         -- Input parsing (facelet string, scramble notation)
      format.rs        -- Output formatting
```

### 8.2 Development Phases

1. **Cube representation:** Implement facelet and cubie representations with conversion between them. Write thorough tests.
2. **Move system:** Define all 18 moves as corner/edge permutation cycles with orientation changes. Verify with known move sequences.
3. **Validation:** Implement all solvability checks. Test with known solvable and unsolvable states.
4. **Coordinate system:** Implement coordinate encoding/decoding for all 6 coordinates. Test round-trip conversion.
5. **Move tables:** Generate move tables for all coordinates. Verify by comparing with move application on cubie representation.
6. **Pruning tables:** Generate pruning tables via BFS. Start with the simple split approach (~2 MB total). Verify a few known distances.
7. **IDA* search:** Implement the search for both phases. Test on known scrambles.
8. **Two-phase coordination:** Implement the solution improvement loop. Benchmark on random scrambles.
9. **CLI:** Add clap-based argument parsing, input/output formatting.

### 8.3 Key Design Decisions

| Decision | Recommendation | Rationale |
|----------|---------------|-----------|
| Pruning strategy | Start with split tables (~2 MB) | Simple, fast to generate, sufficient quality |
| Table storage | Generate at startup, cache to disk | ~1-5 seconds startup, then instant |
| Serialization | bincode or raw bytes | Fast load/save for precomputed tables |
| Move encoding | u8 (0-17) | Simple, compact |
| Error handling | thiserror for library, anyhow for CLI | Standard Rust practice |
| CLI framework | clap (derive) | Standard, well-documented |

### 8.4 Testing Strategy

- **Unit tests:** Coordinate encoding round-trips, move application, validation
- **Property tests:** Random scrambles always produce solvable states; solving then applying the solution yields the identity
- **Known solutions:** Superflip (requires 20 moves in HTM), simple cases (single move, two moves)
- **Benchmark:** Solve 1000 random scrambles, measure time distribution and solution length distribution

### 8.5 Performance Targets

| Metric | Target |
|--------|--------|
| Average solve time | < 50 ms |
| Worst case solve time | < 500 ms |
| Average solution length | <= 22 moves |
| Table generation | < 10 seconds |
| Binary size | < 5 MB |
| Runtime memory | < 50 MB |

---

## Sources

- Kociemba, H. "Two-Phase Algorithm." https://kociemba.org/math/twophase.htm
- Kociemba, H. "Coordinate Level." https://kociemba.org/math/coordlevel.htm
- Kociemba, H. "Implementation of the Two-Phase Algorithm." https://kociemba.org/math/imptwophase.htm
- Rokicki, T. et al. "God's Number is 20." https://cube20.org/
- kewb crate: https://crates.io/crates/kewb
- kociema: https://github.com/adungaos/kociema
- Kociemba's Python solver: https://github.com/hkociemba/RubiksCube-TwophaseSolver
- Wikipedia, "Optimal solutions for the Rubik's Cube"
- Jaap's Puzzle Page, "Cube Theory": https://www.jaapsch.net/puzzles/theory.htm
