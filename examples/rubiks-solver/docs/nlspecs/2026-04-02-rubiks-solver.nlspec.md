---
date: 2026-04-02
topic: rubiks-solver
source_spec: docs/specs/2026-04-02-rubiks-solver-spec.md
status: reviewed
---

# Rubik's Cube Solver NLSpec

A command-line Rubik's cube solver that reads a 54-character facelet string, validates it, solves it using Kociemba's two-phase algorithm with IDA* search, and outputs a near-optimal move sequence. Intended for use as a Foundry adversarial workflow example.

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

Rubik's cube solving is a well-studied combinatorial problem with ~4.33x10^19 reachable states. We need a non-trivial example to demonstrate the Foundry adversarial red/green workflow on an algorithm involving precise state representation, group theory, coordinate encoding, and lookup-table-driven search -- where the red team writes tests from this NLSpec's Definition of Done and the green team implements from the How section, neither seeing the other's work.

### 1.2 Design Principles

**Zero non-essential dependencies.** Only `clap` for CLI argument parsing. The algorithm is self-contained: table generation and IDA* search require only arrays and arithmetic.

**Two-phase search.** Kociemba's algorithm splits the 4.33x10^19-state space into two manageable phases using group theory. Phase 1 reduces to a ~19.5-billion-element subgroup G1; phase 2 solves within G1. This yields near-optimal solutions (18-25 moves) in milliseconds.

**Split pruning tables.** ~2 MB total memory. Pair coordinates (CO x UDSlice, EO x UDSlice for phase 1; CP x USP, EP x USP for phase 2) and take the max of two heuristics. This avoids the ~67 MB cost of full symmetry-reduced tables while providing adequate pruning.

**Fail fast on bad input.** Invalid input is rejected before solving with a clear error message and appropriate exit code.

### 1.3 Layering and Scope

This spec covers: parsing a 54-character facelet string, validating solvability, generating move/pruning tables, solving via Kociemba two-phase with IDA*, and outputting the solution as a Singmaster move sequence via CLI. It does NOT cover: other cube sizes, GUI, optimal solving, scramble generation, or table compression beyond the split approach.

---

## 2. What

### 2.1 Data Model

```
ENUM Color: U=0, R=1, F=2, D=3, L=4, B=5

TYPE Facelet = [Color; 54]
-- Indices 0-8: U face, 9-17: R face, 18-26: F face,
--          27-35: D face, 36-44: L face, 45-53: B face
-- Per face layout (reading order):
--   0 1 2
--   3 4 5    (index 4 is center)
--   6 7 8
-- Center positions: U=4, R=13, F=22, D=31, L=40, B=49

RECORD CubieCube:
    corner_perm:   [u8; 8]   -- which corner (0-7) occupies each slot
    corner_orient: [u8; 8]   -- orientation of corner in each slot (0, 1, or 2)
    edge_perm:     [u8; 12]  -- which edge (0-11) occupies each slot
    edge_orient:   [u8; 12]  -- orientation of edge in each slot (0 or 1)

-- Corner numbering:
--   0=URF  1=UFL  2=ULB  3=UBR  4=DFR  5=DLF  6=DBL  7=DRB
-- Corner facelets (three facelets per corner, U/D sticker listed first):
--   URF: (U8, R9, F20)     UFL: (U6, F18, L44)
--   ULB: (U0, L36, B53)    UBR: (U2, B45, R11)
--   DFR: (D29, F26, R15)   DLF: (D27, L42, F24)
--   DBL: (D33, B51, L38)   DRB: (D35, R17, B47)
-- Orientation: 0 = U/D-color sticker is on U or D face
--              1 = U/D-color sticker is one twist clockwise from U/D face
--              2 = U/D-color sticker is two twists clockwise from U/D face

-- Edge numbering:
--   0=UR  1=UF  2=UL  3=UB  4=DR  5=DF  6=DL  7=DB
--   8=FR  9=FL  10=BL  11=BR
-- Edge facelets (two facelets per edge, U/D sticker listed first where applicable):
--   UR: (U5, R10)    UF: (U7, F19)    UL: (U3, L37)    UB: (U1, B46)
--   DR: (D32, R16)   DF: (D28, F25)   DL: (D30, L43)   DB: (D34, B52)
--   FR: (F23, R12)   FL: (F21, L41)   BL: (B50, L39)   BR: (B48, R14)
-- Orientation: 0 = correctly oriented, 1 = flipped
-- Edges 8-11 (FR, FL, BL, BR) are the "UD-slice" edges

RECORD Coordinates:
    -- Phase 1 coordinates (describe distance from G1)
    co:       u16  -- Corner Orientation, range 0-2186 (3^7 = 2187 values)
    eo:       u16  -- Edge Orientation, range 0-2047 (2^11 = 2048 values)
    ud_slice: u16  -- UD Slice combination, range 0-494 (C(12,4) = 495 values)

    -- Phase 2 coordinates (describe distance from solved within G1)
    cp:       u16  -- Corner Permutation, range 0-40319 (8! = 40320 values)
    ep:       u16  -- U/D Edge Permutation (8 non-slice edges), range 0-40319 (8! values)
    usp:      u8   -- UD Slice sorted Permutation, range 0-23 (4! = 24 values)

-- Move encoding: 18 moves total
--   move_id / 3 = face (0=U, 1=D, 2=R, 3=L, 4=F, 5=B)
--   move_id % 3 = variant (0=CW, 1=CCW, 2=double)
--   Names: U U' U2 D D' D2 R R' R2 L L' L2 F F' F2 B B' B2
--   IDs:    0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17
-- Phase 2 uses only 10 moves: U U' U2 D D' D2 R2 L2 F2 B2
--   (phase2 move indices 0-9 map to move_ids: 0,1,2,3,4,5,8,11,14,17)

ENUM SolveResult:
    SOLVED(Vec<u8>)          -- move sequence (each element is a move_id 0-17)
    ALREADY_SOLVED
    UNSOLVABLE(String)

ENUM InputError:
    WRONG_LENGTH { actual: usize }
    INVALID_CHARACTER { char: char, position: usize }

ENUM ValidationError:
    WRONG_STICKER_COUNT { color: Color, count: u8 }
    WRONG_CENTER { position: u8, expected: Color, found: Color }
    CORNER_ORIENTATION_PARITY { sum: u8 }
    EDGE_ORIENTATION_PARITY { sum: u8 }
    PERMUTATION_PARITY_MISMATCH
    DUPLICATE_CUBIE { kind: &str, index: u8 }
```

### 2.2 Architecture

Five modules:
- `cube::facelet` -- facelet representation, parsing, display
- `cube::cubie` -- CubieCube struct, move application, facelet-to-cubie conversion
- `cube::coord` -- coordinate encoding/decoding
- `cube::validate` -- solvability checks
- `solver::tables` -- move table and pruning table generation/caching
- `solver::search` -- IDA* search for both phases
- `solver::kociemba` -- two-phase coordination
- `main` -- CLI entry point, output formatting

### 2.3 Vocabulary

- **Facelet**: one of the 54 colored stickers on the cube surface
- **Cubie**: one of the 20 movable pieces (8 corners with 3 facelets, 12 edges with 2 facelets)
- **Coordinate**: an integer encoding of one aspect of the cube state, used as a table index
- **G1**: the subgroup <U, D, R2, L2, F2, B2> where all orientations are solved and UD-slice edges are in the middle layer
- **Move table**: precomputed array mapping (coordinate, move) to new coordinate
- **Pruning table**: precomputed array mapping coordinate combinations to minimum moves needed
- **IDA***: iterative deepening A* search -- depth-limited DFS with heuristic pruning
- **Lehmer code**: a bijection from permutations to integers in 0..n!-1
- **Combinadic**: a bijection from k-element subsets of {0..n-1} to integers in 0..C(n,k)-1

---

## 3. How

### 3.1 Parse Facelet String

```
FUNCTION parse(input: String) -> Result<Facelet, InputError>:
    cleaned = input.trim().to_uppercase()

    IF cleaned.length != 54:
        RETURN Err(WRONG_LENGTH { actual: cleaned.length })

    facelet = [0u8; 54]
    FOR i IN 0..54:
        char = cleaned[i]
        MATCH char:
            'U' => facelet[i] = 0
            'R' => facelet[i] = 1
            'F' => facelet[i] = 2
            'D' => facelet[i] = 3
            'L' => facelet[i] = 4
            'B' => facelet[i] = 5
            _   => RETURN Err(INVALID_CHARACTER { char, position: i })

    RETURN Ok(facelet)
```

### 3.2 Convert Facelets to Cubies

```
-- Corner facelet triples (U/D sticker first, then clockwise):
CONST CORNER_FACELETS: [[u8; 3]; 8] = [
    [8,  9,  20],  -- URF
    [6,  18, 44],  -- UFL
    [0,  36, 53],  -- ULB  (note: U0, L36, B53 — NOT U0, B53, L36)
    [2,  45, 11],  -- UBR
    [29, 26, 15],  -- DFR
    [27, 42, 24],  -- DLF  (note: D27, L42, F24)
    [33, 51, 38],  -- DBL
    [35, 17, 47],  -- DRB
]

-- Edge facelet pairs (U/D sticker first, or F/B sticker first for slice edges):
CONST EDGE_FACELETS: [[u8; 2]; 12] = [
    [5,  10],  -- UR
    [7,  19],  -- UF
    [3,  37],  -- UL
    [1,  46],  -- UB
    [32, 16],  -- DR
    [28, 25],  -- DF
    [30, 43],  -- DL
    [34, 52],  -- DB
    [23, 12],  -- FR
    [21, 41],  -- FL
    [50, 39],  -- BL
    [48, 14],  -- BR
]

-- The color that each corner position's first facelet should show when orientation=0:
CONST CORNER_COLORS: [[Color; 3]; 8] = [
    [U, R, F], [U, F, L], [U, L, B], [U, B, R],
    [D, F, R], [D, L, F], [D, B, L], [D, R, B],
]
CONST EDGE_COLORS: [[Color; 2]; 12] = [
    [U, R], [U, F], [U, L], [U, B],
    [D, R], [D, F], [D, L], [D, B],
    [F, R], [F, L], [B, L], [B, R],
]

FUNCTION facelets_to_cubies(facelet: Facelet) -> CubieCube:
    cube = CubieCube with all zeros

    -- Determine each corner
    FOR slot IN 0..8:
        f0 = facelet[CORNER_FACELETS[slot][0]]
        f1 = facelet[CORNER_FACELETS[slot][1]]
        f2 = facelet[CORNER_FACELETS[slot][2]]
        -- Find which corner cubie has these three colors
        FOR c IN 0..8:
            IF {f0, f1, f2} == set(CORNER_COLORS[c]):
                cube.corner_perm[slot] = c
                -- Orientation = rotation needed to bring U/D color to the U/D position
                IF f0 == CORNER_COLORS[c][0]:
                    cube.corner_orient[slot] = 0
                ELSE IF f1 == CORNER_COLORS[c][0]:
                    cube.corner_orient[slot] = 1
                ELSE:
                    cube.corner_orient[slot] = 2
                BREAK

    -- Determine each edge
    FOR slot IN 0..12:
        f0 = facelet[EDGE_FACELETS[slot][0]]
        f1 = facelet[EDGE_FACELETS[slot][1]]
        FOR e IN 0..12:
            IF {f0, f1} == set(EDGE_COLORS[e]):
                cube.edge_perm[slot] = e
                IF f0 == EDGE_COLORS[e][0]:
                    cube.edge_orient[slot] = 0
                ELSE:
                    cube.edge_orient[slot] = 1
                BREAK

    RETURN cube
```

### 3.3 Convert Cubies to Coordinates

```
FUNCTION encode_co(cube: &CubieCube) -> u16:
    -- Base-3 encoding of 7 independent corner orientations (8th is determined)
    val = 0
    FOR i IN 0..7:
        val = val * 3 + cube.corner_orient[i]
    RETURN val    -- range 0..2186

FUNCTION encode_eo(cube: &CubieCube) -> u16:
    -- Base-2 encoding of 11 independent edge orientations (12th is determined)
    val = 0
    FOR i IN 0..11:
        val = val * 2 + cube.edge_orient[i]
    RETURN val    -- range 0..2047

FUNCTION encode_ud_slice(cube: &CubieCube) -> u16:
    -- Which 4 of 12 edge positions contain a slice edge (edge_perm[pos] >= 8)
    -- Use combinadic encoding: sort the 4 positions ascending as p0<p1<p2<p3
    -- index = C(p0,1) + C(p1,2) + C(p2,3) + C(p3,4)
    positions = [pos for pos in 0..12 if cube.edge_perm[pos] >= 8], sorted ascending
    RETURN C(positions[0], 1) + C(positions[1], 2) + C(positions[2], 3) + C(positions[3], 4)
    -- range 0..494, goal state = 0 when slice edges at positions 8,9,10,11
    -- because C(8,1)+C(9,2)+C(10,3)+C(11,4) is NOT 0; see note below

-- NOTE on UDSlice goal state: the combinadic maps positions {8,9,10,11} to
-- C(8,1)+C(9,2)+C(10,3)+C(11,4) = 8+36+120+330 = 494. The goal is uds=494
-- if using this exact formula. Alternatively, renumber so goal = 0 by using
-- the complementary encoding: track which positions do NOT have slice edges.
-- Either convention works; be consistent between encoding and pruning table.

FUNCTION encode_cp(cube: &CubieCube) -> u16:
    -- Lehmer code of corner permutation
    val = 0
    FOR i IN 0..8:
        -- count how many values in positions i+1..7 are less than corner_perm[i]
        k = count(j in i+1..8 where cube.corner_perm[j] < cube.corner_perm[i])
        val = val * (8 - i) + k
    RETURN val    -- range 0..40319

FUNCTION encode_ep(cube: &CubieCube) -> u16:
    -- Lehmer code of the 8 non-slice edge permutation (edges 0-7)
    -- Extract positions of edges 0-7 in order
    perm8 = [cube.edge_perm[i] for i in 0..8]  -- only valid in phase 2 (slice edges at 8-11)
    val = 0
    FOR i IN 0..8:
        k = count(j in i+1..8 where perm8[j] < perm8[i])
        val = val * (8 - i) + k
    RETURN val    -- range 0..40319

FUNCTION encode_usp(cube: &CubieCube) -> u8:
    -- Lehmer code of the 4 slice edges' permutation within positions 8-11
    perm4 = [cube.edge_perm[i] - 8 for i in 8..12]  -- normalize to 0-3
    val = 0
    FOR i IN 0..4:
        k = count(j in i+1..4 where perm4[j] < perm4[i])
        val = val * (4 - i) + k
    RETURN val    -- range 0..23
```

### 3.4 Validate Cube

```
FUNCTION validate(facelet: &Facelet, cube: &CubieCube) -> Result<(), ValidationError>:
    -- Check 1: Sticker counts — each color must appear exactly 9 times
    counts = [0u8; 6]
    FOR f IN facelet:
        counts[f] += 1
    FOR c IN 0..6:
        IF counts[c] != 9:
            RETURN Err(WRONG_STICKER_COUNT { color: c, count: counts[c] })

    -- Check 2: Centers — positions 4,13,22,31,40,49 must be U,R,F,D,L,B respectively
    FOR (i, pos) IN [4,13,22,31,40,49].enumerate():
        IF facelet[pos] != i:
            RETURN Err(WRONG_CENTER { position: pos, expected: i, found: facelet[pos] })

    -- Check 3: Corner orientation parity — sum of 8 orientations must be 0 mod 3
    co_sum = sum(cube.corner_orient[0..8])
    IF co_sum % 3 != 0:
        RETURN Err(CORNER_ORIENTATION_PARITY { sum: co_sum })

    -- Check 4: Edge orientation parity — sum of 12 orientations must be 0 mod 2
    eo_sum = sum(cube.edge_orient[0..12])
    IF eo_sum % 2 != 0:
        RETURN Err(EDGE_ORIENTATION_PARITY { sum: eo_sum })

    -- Check 5: Permutation parity — corner parity must equal edge parity
    cp_parity = parity(cube.corner_perm)    -- true=even, false=odd
    ep_parity = parity(cube.edge_perm)
    IF cp_parity != ep_parity:
        RETURN Err(PERMUTATION_PARITY_MISMATCH)

    -- Check 6: Uniqueness — each corner 0-7 appears exactly once, each edge 0-11 once
    corner_seen = [false; 8]
    FOR i IN 0..8:
        IF corner_seen[cube.corner_perm[i]]:
            RETURN Err(DUPLICATE_CUBIE { kind: "corner", index: cube.corner_perm[i] })
        corner_seen[cube.corner_perm[i]] = true
    edge_seen = [false; 12]
    FOR i IN 0..12:
        IF edge_seen[cube.edge_perm[i]]:
            RETURN Err(DUPLICATE_CUBIE { kind: "edge", index: cube.edge_perm[i] })
        edge_seen[cube.edge_perm[i]] = true

    RETURN Ok(())

FUNCTION parity(perm: &[u8]) -> bool:
    -- Returns true if even permutation, false if odd
    visited = [false; perm.len()]
    even = true
    FOR i IN 0..perm.len():
        IF NOT visited[i]:
            j = i
            cycle_len = 0
            WHILE NOT visited[j]:
                visited[j] = true
                j = perm[j]
                cycle_len += 1
            IF cycle_len % 2 == 0:
                even = !even   -- even-length cycles contribute odd parity
    RETURN even
```

### 3.5 Generate Move Tables

```
-- Move definitions: each 90-degree CW face turn is a 4-cycle on corners and edges
-- with orientation deltas.

CONST CORNER_CYCLES: [[u8; 4]; 6] = [
    [0, 3, 2, 1],  -- U: URF->UBR->ULB->UFL
    [4, 5, 6, 7],  -- D: DFR->DLF->DBL->DRB
    [0, 4, 7, 3],  -- R: URF->DFR->DRB->UBR
    [1, 2, 6, 5],  -- L: UFL->ULB->DBL->DLF
    [0, 1, 5, 4],  -- F: URF->UFL->DLF->DFR
    [3, 7, 6, 2],  -- B: UBR->DRB->DBL->ULB
]
CONST CORNER_ORIENT_DELTAS: [[u8; 4]; 6] = [
    [0, 0, 0, 0],  -- U
    [0, 0, 0, 0],  -- D
    [1, 2, 1, 2],  -- R
    [1, 2, 1, 2],  -- L
    [1, 2, 1, 2],  -- F
    [1, 2, 1, 2],  -- B
]
CONST EDGE_CYCLES: [[u8; 4]; 6] = [
    [0, 3, 2, 1],  -- U: UR->UB->UL->UF
    [4, 7, 6, 5],  -- D: DR->DB->DL->DF
    [0, 8, 4, 11], -- R: UR->FR->DR->BR
    [2, 9, 6, 10], -- L: UL->FL->DL->BL
    [1, 9, 5, 8],  -- F: UF->FL->DF->FR
    [3, 10, 7, 11], -- B: UB->BL->DB->BR
]
CONST EDGE_ORIENT_DELTAS: [[u8; 4]; 6] = [
    [0, 0, 0, 0],  -- U
    [0, 0, 0, 0],  -- D
    [0, 0, 0, 0],  -- R
    [0, 0, 0, 0],  -- L
    [1, 1, 1, 1],  -- F  (F and B flip edges)
    [1, 1, 1, 1],  -- B
]

FUNCTION apply_move(cube: &CubieCube, move_id: u8) -> CubieCube:
    face = move_id / 3        -- 0-5
    variant = move_id % 3     -- 0=CW, 1=CCW, 2=double
    result = clone(cube)

    -- Apply corner 4-cycle with orientation
    cycle = CORNER_CYCLES[face]
    orient_d = CORNER_ORIENT_DELTAS[face]
    MATCH variant:
        0 (CW):   -- a->b->c->d->a  (d takes a's place, etc.)
            result.corner_perm[cycle[0]] = cube.corner_perm[cycle[3]]
            result.corner_perm[cycle[1]] = cube.corner_perm[cycle[0]]
            result.corner_perm[cycle[2]] = cube.corner_perm[cycle[1]]
            result.corner_perm[cycle[3]] = cube.corner_perm[cycle[2]]
            result.corner_orient[cycle[0]] = (cube.corner_orient[cycle[3]] + orient_d[0]) % 3
            result.corner_orient[cycle[1]] = (cube.corner_orient[cycle[0]] + orient_d[1]) % 3
            result.corner_orient[cycle[2]] = (cube.corner_orient[cycle[1]] + orient_d[2]) % 3
            result.corner_orient[cycle[3]] = (cube.corner_orient[cycle[2]] + orient_d[3]) % 3
        1 (CCW):  -- apply CW three times, or equivalently reverse the cycle
            apply CW cycle in reverse direction (a takes b's place, etc.)
        2 (double): -- apply CW twice: swap pairs (a<->c, b<->d)
            swap (cycle[0], cycle[2]) in perm and orient (orient deltas applied twice)

    -- Apply edge 4-cycle with orientation (same logic, mod 2 for orient)
    cycle = EDGE_CYCLES[face]
    orient_d = EDGE_ORIENT_DELTAS[face]
    -- same CW/CCW/double logic as corners, orientations mod 2

    RETURN result

FUNCTION generate_move_tables():
    -- Phase 1 move tables
    co_move  = array[2187][18] of u16
    eo_move  = array[2048][18] of u16
    uds_move = array[495][18]  of u16

    FOR coord_val IN 0..max:
        cube = decode_coordinate_to_cubie(coord_val)   -- reconstruct a cubie with this coordinate
        FOR move_id IN 0..18:
            new_cube = apply_move(cube, move_id)
            co_move[coord_val][move_id]  = encode_co(new_cube)   -- (for CO table)
            eo_move[coord_val][move_id]  = encode_eo(new_cube)   -- (for EO table)
            uds_move[coord_val][move_id] = encode_ud_slice(new_cube) -- (for UDS table)

    -- Phase 2 move tables (only 10 G1-preserving moves)
    cp_move  = array[40320][10] of u16
    ep_move  = array[40320][10] of u16
    usp_move = array[24][10]    of u8
    -- Same approach: decode, apply move, re-encode
    -- Phase 2 move indices map to move_ids: [0,1,2,3,4,5,8,11,14,17]
```

### 3.6 Generate Pruning Tables

```
FUNCTION generate_pruning_table(table_size: usize, coord_count: (usize, usize),
                                 move_table_a, move_table_b,
                                 num_moves: u8) -> Vec<u8>:
    -- table indexed by (coord_a * coord_count.1 + coord_b)
    -- each entry stores minimum moves to reach goal (both coords = 0), packed 4 bits
    prune = vec![0xFF; table_size]   -- 0xFF = unvisited
    set_entry(prune, goal_index, 0)
    filled = 1
    depth = 0

    WHILE filled < table_size:
        FOR idx IN 0..table_size:
            IF get_entry(prune, idx) == depth:
                a = idx / coord_count.1
                b = idx % coord_count.1
                FOR move_id IN 0..num_moves:
                    new_a = move_table_a[a][move_id]
                    new_b = move_table_b[b][move_id]
                    new_idx = new_a * coord_count.1 + new_b
                    IF get_entry(prune, new_idx) == 0xFF:
                        set_entry(prune, new_idx, depth + 1)
                        filled += 1
        depth += 1

    RETURN prune

-- Phase 1 pruning tables:
--   co_uds_prune:  2187 x 495  = 1,082,565 entries (goal: co=0, uds=goal)
--   eo_uds_prune:  2048 x 495  = 1,013,760 entries (goal: eo=0, uds=goal)
-- Phase 2 pruning tables:
--   cp_usp_prune:  40320 x 24  = 967,680 entries (goal: cp=0, usp=0)
--   ep_usp_prune:  40320 x 24  = 967,680 entries (goal: ep=0, usp=0)
-- Each entry is 4 bits; total ~2 MB
```

### 3.7 IDA* Search

```
FUNCTION ida_star(initial_coords, move_table, prune_tables, allowed_moves,
                  max_depth: u8) -> Option<Vec<u8>>:
    h = heuristic(initial_coords, prune_tables)
    FOR depth_limit IN h..=max_depth:
        path = []
        result = dfs(initial_coords, 0, depth_limit, path, move_table,
                      prune_tables, allowed_moves, NO_LAST_FACE)
        IF result IS Some(solution):
            RETURN Some(solution)
    RETURN None

FUNCTION dfs(coords, depth, limit, path, move_table, prune_tables,
             allowed_moves, last_face) -> Option<Vec<u8>>:
    h = heuristic(coords, prune_tables)
    IF depth + h > limit:
        RETURN None                       -- prune: cannot reach goal in time

    IF is_goal(coords):
        RETURN Some(path)

    FOR move_id IN allowed_moves:
        face = move_id / 3
        -- Move redundancy pruning:
        IF face == last_face:
            CONTINUE                      -- never same face consecutively
        IF are_opposite(face, last_face) AND face > last_face:
            CONTINUE                      -- canonical order for commuting faces

        new_coords = apply_move_tables(coords, move_id, move_table)
        path.push(move_id)
        result = dfs(new_coords, depth + 1, limit, path, move_table,
                      prune_tables, allowed_moves, face)
        IF result IS Some:
            RETURN result
        path.pop()

    RETURN None

FUNCTION heuristic(coords, prune_tables) -> u8:
    -- max of two sub-table lookups
    RETURN max(prune_tables[0].lookup(coords), prune_tables[1].lookup(coords))

FUNCTION are_opposite(f1, f2) -> bool:
    -- U(0)/D(1), R(2)/L(3), F(4)/B(5)
    RETURN (f1 / 2 == f2 / 2) AND (f1 != f2)
```

### 3.8 Two-Phase Solve

```
FUNCTION solve(cube: CubieCube, tables: &Tables) -> SolveResult:
    co  = encode_co(cube)
    eo  = encode_eo(cube)
    uds = encode_ud_slice(cube)

    -- Check already solved
    IF co == 0 AND eo == 0 AND uds == GOAL_UDS
       AND encode_cp(cube) == 0 AND encode_ep(cube) == 0 AND encode_usp(cube) == 0:
        RETURN ALREADY_SOLVED

    best_solution = None
    best_length = 25                -- maximum acceptable total length

    -- Phase 1: find moves to reach G1 (co=0, eo=0, uds=goal)
    p1_coords = (co, eo, uds)
    h1 = phase1_heuristic(p1_coords, tables)

    FOR p1_limit IN h1..min(12, best_length):
        -- Find ALL phase-1 solutions of exactly p1_limit moves via IDA*
        -- For each phase-1 solution:
        FOR p1_solution IN phase1_search(p1_coords, p1_limit, tables):
            -- Apply phase-1 moves to get the G1-state cube
            cube_g1 = apply_moves(cube, p1_solution)
            cp  = encode_cp(cube_g1)
            ep  = encode_ep(cube_g1)
            usp = encode_usp(cube_g1)

            -- Phase 2: solve within G1 using only G1-preserving moves
            p2_limit = best_length - p1_solution.len() - 1
            p2_coords = (cp, ep, usp)
            p2_solution = ida_star(p2_coords, tables.phase2_move, tables.phase2_prune,
                                    PHASE2_MOVES, p2_limit)

            IF p2_solution IS Some(moves):
                total = p1_solution + moves
                IF total.len() < best_length:
                    best_solution = Some(total)
                    best_length = total.len()

    IF best_solution IS Some(s):
        RETURN SOLVED(s)
    RETURN UNSOLVABLE("No solution found within move limit")
```

### 3.9 Output Formatting

```
CONST MOVE_NAMES: [&str; 18] = [
    "U", "U'", "U2", "D", "D'", "D2",
    "R", "R'", "R2", "L", "L'", "L2",
    "F", "F'", "F2", "B", "B'", "B2",
]

FUNCTION format_solution(moves: &[u8]) -> String:
    RETURN moves.iter().map(|&m| MOVE_NAMES[m]).collect::<Vec<_>>().join(" ")
```

### 3.10 CLI Entry Point

```
FUNCTION main():
    -- Read input: first CLI argument, or stdin
    input = IF args.len() > 1: args[1] ELSE: read_stdin_line()

    -- Parse facelet string
    facelet = parse(input)
    IF facelet IS Err(e):
        eprintln("Error: {e}")
        exit(1)

    -- Convert to cubie representation
    cube = facelets_to_cubies(facelet)

    -- Validate solvability
    validation = validate(facelet, cube)
    IF validation IS Err(e):
        eprintln("Error: {e}")
        exit(1)

    -- Load or generate tables
    tables = load_tables_from_cache()
    IF tables IS None:
        tables = generate_all_tables()
        save_tables_to_cache(tables)     -- warn on failure, do not exit

    -- Solve
    result = solve(cube, tables)
    MATCH result:
        ALREADY_SOLVED:
            println("Already solved")
            exit(0)
        SOLVED(moves):
            println(format_solution(moves))
            exit(0)
        UNSOLVABLE(msg):
            eprintln("Error: {msg}")
            exit(2)
```

---

## 4. Out of Scope

- **Other cube sizes.** 2x2, 4x4, or NxNxN cubes. Extension point: parameterize cubie counts and coordinate dimensions.
- **Optimal solver.** Finding provably shortest solutions requires Korf's IDA* with >1 GB pattern databases. Extension point: add a `--optimal` flag that uses larger tables.
- **Symmetry-reduced tables.** The ~67 MB FlipUDSlice table gives stronger pruning. Extension point: add symmetry conjugation logic and larger tables behind a `--full-tables` flag.
- **GUI, visualization, or interactive mode.** Extension point: expose the solver as a library crate.
- **Scramble generation.** Generating random scrambles or converting scramble notation to facelet strings. Extension point: add a `scramble` subcommand.
- **Multi-threaded search.** Extension point: partition phase-1 solutions across threads.

---

## 5. Design Decision Rationale

**Why Kociemba two-phase instead of Korf's optimal solver?** Kociemba produces near-optimal solutions (18-25 moves) in milliseconds with ~2 MB tables. Korf's solver guarantees optimality but needs >1 GB tables and seconds-to-minutes per solve. For a CLI tool, near-optimal in milliseconds is the pragmatic choice.

**Why split pruning tables instead of symmetry-reduced FlipUDSlice?** Split tables (CO x UDSlice + EO x UDSlice for phase 1, ~1 MB each) total ~2 MB and generate in under a second. The symmetry-reduced approach yields ~67 MB tables needing 5-30 seconds to generate and complex symmetry conjugation code. Split tables are simpler and sufficient for <=25-move solutions.

**Why combinadic for UD-slice encoding?** The UD-slice coordinate tracks which 4 of 12 positions contain slice edges, ignoring their order. The combinatorial number system gives a bijection to 0..494, which is the densest encoding. Lehmer code would encode a full permutation (12!), which is wasteful when only the combination matters.

**Why Lehmer code for permutation coordinates?** It gives a bijection from permutations of n elements to integers in 0..n!-1. This is the standard encoding for permutation coordinates in the Kociemba algorithm. Computing it is O(n^2) which is trivial for n<=12.

**Why 4-bit pruning entries?** Phase 1 pruning depths max out around 12; phase 2 around 18. Both fit in 4 bits (0-15). Packing two entries per byte halves memory usage with minimal access overhead.

**Why `clap` and no other dependencies?** The algorithm needs only arrays, integer arithmetic, and file I/O. `clap` handles CLI ergonomics (help text, argument parsing) better than manual parsing. No other crate adds value.

---

## 6. Definition of Done

### 6.1 Input Parsing (mirrors 3.1)
- [ ] Accepts a 54-character string of face letters U, R, F, D, L, B
- [ ] Handles both uppercase and lowercase input
- [ ] Rejects input with fewer than 54 characters (exit 1)
- [ ] Rejects input with more than 54 characters (exit 1)
- [ ] Rejects input containing characters other than U/R/F/D/L/B (exit 1)
- [ ] Reports the invalid character and its position in the error message
- [ ] Strips leading/trailing whitespace before parsing
- [ ] The solved-state string `UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB` parses successfully

### 6.2 Facelet-to-Cubie Conversion (mirrors 3.2)
- [ ] Solved facelet string produces identity CubieCube: corner_perm=[0,1,2,3,4,5,6,7], corner_orient=[0;8], edge_perm=[0,1,..11], edge_orient=[0;12]
- [ ] A single U move applied to solved state produces corner_perm=[3,0,1,2,4,5,6,7] with corner_orient=[0;8]
- [ ] A single F move produces edge_orient values of 1 for the four affected edges (UF, FL, DF, FR)
- [ ] A single R move produces corner_orient [1,0,0,2,2,0,0,1] (orientations of the 4 affected corners change by +1,+2,+1,+2)
- [ ] Round-trip: facelet -> cubie -> apply 18 identity moves (each face CW then CCW) -> same cubie

### 6.3 Cubie-to-Coordinate Conversion (mirrors 3.3)
- [ ] Solved cube encodes to: co=0, eo=0, cp=0, ep=0, usp=0
- [ ] UDSlice coordinate for solved cube equals the goal value (slice edges at positions 8-11)
- [ ] A single F move from solved changes eo (4 edges flipped) and co (4 corners twisted) to non-zero values
- [ ] Encoding then decoding (round-trip) reproduces the original cubie state for CO, EO, CP, EP, USP
- [ ] CO encoding of corner_orient=[1,0,0,0,0,0,0,2] equals 1*3^6 = 729
- [ ] EO encoding of edge_orient=[1,0,0,0,0,0,0,0,0,0,0,1] equals 1*2^10 = 1024

### 6.4 Validation (mirrors 3.4)
- [ ] Rejects cube with wrong sticker count (e.g., 10 U's and 8 R's) — reports which color
- [ ] Rejects cube with wrong center (e.g., position 4 is not U) — reports position and colors
- [ ] Rejects cube with corner orientation parity violation (sum not divisible by 3) — reports sum
- [ ] Rejects cube with edge orientation parity violation (sum not even) — reports sum
- [ ] Rejects cube with permutation parity mismatch (corner parity != edge parity)
- [ ] Rejects cube with duplicate cubies (same corner or edge in two positions)
- [ ] Accepts the solved cube
- [ ] Accepts any cube generated by applying random legal moves to solved state

### 6.5 Move Tables (mirrors 3.5)
- [ ] Phase 1 CO move table: applying U to co=0 returns co=0 (U does not change corner orientation)
- [ ] Phase 1 EO move table: applying F to eo=0 returns eo != 0 (F flips 4 edges)
- [ ] Phase 1 UDS move table: applying R to uds=goal returns uds != goal (R moves a slice edge)
- [ ] Phase 2 CP move table: applying U to cp=0 returns cp != 0 (U permutes corners)
- [ ] For every coordinate and move, applying the move then its inverse returns the original coordinate
- [ ] Move table dimensions: co_move is 2187x18, eo_move is 2048x18, uds_move is 495x18, cp_move is 40320x10, ep_move is 40320x10, usp_move is 24x10

### 6.6 Pruning Tables (mirrors 3.6)
- [ ] Goal state entry is 0 in all four pruning tables
- [ ] No entry exceeds 15 (fits in 4 bits)
- [ ] All entries are filled (no 0xFF remaining)
- [ ] States one move from goal have entry value 1
- [ ] The heuristic is admissible: for any state, h(state) <= actual minimum moves
- [ ] Phase 1 tables: co_uds_prune has 2187*495 entries, eo_uds_prune has 2048*495 entries
- [ ] Phase 2 tables: cp_usp_prune has 40320*24 entries, ep_usp_prune has 40320*24 entries

### 6.7 Phase 1 Search (mirrors 3.7, 3.8)
- [ ] Solved cube: phase 1 finds empty move sequence (already in G1)
- [ ] After phase 1 solution is applied, cube has co=0, eo=0, and slice edges in positions 8-11
- [ ] Single F move scramble: phase 1 finds F' (or equivalent) to restore orientations
- [ ] Move redundancy pruning: solution never has same face consecutively, opposite faces are in canonical order

### 6.8 Phase 2 Search (mirrors 3.7, 3.8)
- [ ] From a G1-state cube, phase 2 finds a solution using only G1-preserving moves: U, U', U2, D, D', D2, R2, L2, F2, B2
- [ ] After phase 2 solution is applied, cube is fully solved (identity permutation, all orientations 0)
- [ ] Phase 2 solution never uses quarter turns of R, L, F, or B

### 6.9 Full Solve (mirrors 3.8)
- [ ] Solves the superflip (all edges flipped, all else correct) — known to require 20 moves in HTM
- [ ] Solves a known 4-move scramble (e.g., R U R' U') and finds a solution of <= 25 moves
- [ ] Solves a known 20-move scramble and finds a solution of <= 25 moves
- [ ] Applying the returned solution moves to the scrambled cube produces the solved state
- [ ] Multiple solves of the same scramble produce valid (though possibly different) solutions

### 6.10 Solution Quality
- [ ] No solution exceeds 25 moves on any valid input
- [ ] Average solution length over 100 random scrambles is <= 22 moves

### 6.11 CLI (mirrors 3.10)
- [ ] Accepts facelet string as first CLI argument
- [ ] Accepts facelet string from stdin if no argument provided
- [ ] Exits with code 0 on successful solve (including already-solved)
- [ ] Exits with code 1 on invalid input (parse error or validation error)
- [ ] Exits with code 2 on unsolvable state
- [ ] Prints solution to stdout as space-separated Singmaster notation (e.g., `R U R' U2 F'`)
- [ ] Prints error messages to stderr
- [ ] Already-solved cube prints "Already solved" to stdout and exits 0

### 6.12 Edge Cases
- [ ] Solved cube: outputs "Already solved", exit 0
- [ ] One-move scramble (e.g., R applied to solved): solution applied to scrambled cube yields solved
- [ ] Superflip: solves correctly (valid solution that restores solved state)

### 6.13 Integration Smoke Test

```
FUNCTION integration_smoke_test():
    -- Solved cube
    solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
    result = run_solver(solved)
    ASSERT result.exit_code == 0
    ASSERT result.stdout.trim() == "Already solved"

    -- Invalid input (too short)
    result = run_solver("UUUUU")
    ASSERT result.exit_code == 1
    ASSERT result.stderr.contains("length")

    -- Invalid character
    result = run_solver("XUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBB")
    ASSERT result.exit_code == 1
    ASSERT result.stderr.contains("character")

    -- Wrong sticker count (10 U's, 8 R's)
    bad_counts = "UUUUUUUUURRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBUBBBBBBBB"
    result = run_solver(bad_counts)
    ASSERT result.exit_code == 1

    -- Known scramble: apply R U R' U' to solved cube, then solve
    -- Facelet string for R U R' U' applied to solved:
    scrambled = "UUFUURUURFFRRRRRRRFFDFFDFFDDDBDDDDDDLLLLLLLLLUBBBBLBBB"
    result = run_solver(scrambled)
    ASSERT result.exit_code == 0
    ASSERT result.stdout.trim().split_whitespace().count() <= 25
    -- Verify: applying the output moves to scrambled state yields solved

    -- 20-move scramble (hard case)
    hard = "DRLUUBFBLDRUFRRFUUBRDFFBDLLUFDRDDBLFLBRLLFRUDRBBUUBRLUD"
    result = run_solver(hard)
    ASSERT result.exit_code == 0
    ASSERT result.stdout.trim().split_whitespace().count() <= 25
```
