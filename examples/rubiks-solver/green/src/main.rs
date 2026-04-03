use std::env;
use std::io::{self, BufRead};
use std::process;

// ============================================================================
// Section 2.1 / 3.1: Data model and constants
// ============================================================================

const COLOR_U: u8 = 0;
const COLOR_R: u8 = 1;
const COLOR_F: u8 = 2;
const COLOR_D: u8 = 3;
const COLOR_L: u8 = 4;
const COLOR_B: u8 = 5;

const COLOR_NAMES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

// Corner facelets (U/D sticker first, then clockwise)
const CORNER_FACELETS: [[usize; 3]; 8] = [
    [8, 9, 20],   // URF
    [6, 18, 38],  // UFL
    [0, 36, 47],  // ULB
    [2, 45, 11],  // UBR
    [29, 26, 15], // DFR
    [27, 44, 24], // DLF
    [33, 53, 42], // DBL
    [35, 17, 51], // DRB
];

// Edge facelets (U/D sticker first, or F/B first for slice edges)
const EDGE_FACELETS: [[usize; 2]; 12] = [
    [5, 10],  // UR
    [7, 19],  // UF
    [3, 37],  // UL
    [1, 46],  // UB
    [32, 16], // DR
    [28, 25], // DF
    [30, 43], // DL
    [34, 52], // DB
    [23, 12], // FR
    [21, 41], // FL
    [50, 39], // BL
    [48, 14], // BR
];

const CORNER_COLORS: [[u8; 3]; 8] = [
    [COLOR_U, COLOR_R, COLOR_F], // URF
    [COLOR_U, COLOR_F, COLOR_L], // UFL
    [COLOR_U, COLOR_L, COLOR_B], // ULB
    [COLOR_U, COLOR_B, COLOR_R], // UBR
    [COLOR_D, COLOR_F, COLOR_R], // DFR
    [COLOR_D, COLOR_L, COLOR_F], // DLF
    [COLOR_D, COLOR_B, COLOR_L], // DBL
    [COLOR_D, COLOR_R, COLOR_B], // DRB
];

const EDGE_COLORS: [[u8; 2]; 12] = [
    [COLOR_U, COLOR_R], // UR
    [COLOR_U, COLOR_F], // UF
    [COLOR_U, COLOR_L], // UL
    [COLOR_U, COLOR_B], // UB
    [COLOR_D, COLOR_R], // DR
    [COLOR_D, COLOR_F], // DF
    [COLOR_D, COLOR_L], // DL
    [COLOR_D, COLOR_B], // DB
    [COLOR_F, COLOR_R], // FR
    [COLOR_F, COLOR_L], // FL
    [COLOR_B, COLOR_L], // BL
    [COLOR_B, COLOR_R], // BR
];

// Move definitions — standard Singmaster CW cycles
const CORNER_CYCLES: [[usize; 4]; 6] = [
    [0, 3, 2, 1], // U: URF->UBR->ULB->UFL
    [4, 5, 6, 7], // D: DFR->DLF->DBL->DRB
    [0, 4, 7, 3], // R: URF->DFR->DRB->UBR
    [2, 1, 5, 6], // L: ULB->UFL->DLF->DBL
    [1, 0, 4, 5], // F: UFL->URF->DFR->DLF
    [3, 2, 6, 7], // B: UBR->ULB->DBL->DRB
];

const CORNER_ORIENT_DELTAS: [[u8; 4]; 6] = [
    [0, 0, 0, 0], // U
    [0, 0, 0, 0], // D
    [2, 1, 2, 1], // R
    [2, 1, 2, 1], // L
    [2, 1, 2, 1], // F
    [2, 1, 2, 1], // B
];

const EDGE_CYCLES: [[usize; 4]; 6] = [
    [0, 3, 2, 1],   // U: UR->UB->UL->UF
    [4, 5, 6, 7],   // D: DR->DF->DL->DB
    [0, 8, 4, 11],  // R: UR->FR->DR->BR
    [2, 9, 6, 10],  // L: UL->FL->DL->BL
    [1, 8, 5, 9],   // F: UF->FR->DF->FL
    [3, 10, 7, 11], // B: UB->BL->DB->BR
];

const EDGE_ORIENT_DELTAS: [[u8; 4]; 6] = [
    [0, 0, 0, 0], // U
    [0, 0, 0, 0], // D
    [0, 0, 0, 0], // R
    [0, 0, 0, 0], // L
    [1, 1, 1, 1], // F
    [1, 1, 1, 1], // B
];

const MOVE_NAMES: [&str; 18] = [
    "U", "U'", "U2", "D", "D'", "D2", "R", "R'", "R2", "L", "L'", "L2", "F", "F'", "F2", "B",
    "B'", "B2",
];

// Phase 2 move IDs (indices into the 18-move array)
const PHASE2_MOVE_IDS: [u8; 10] = [0, 1, 2, 3, 4, 5, 8, 11, 14, 17];

// UDSlice goal: positions {8,9,10,11} => C(8,1)+C(9,2)+C(10,3)+C(11,4) = 494
const UDS_GOAL: u16 = 494;

// ============================================================================
// CubieCube
// ============================================================================

#[derive(Clone)]
struct CubieCube {
    corner_perm: [u8; 8],
    corner_orient: [u8; 8],
    edge_perm: [u8; 12],
    edge_orient: [u8; 12],
}

impl CubieCube {
    fn solved() -> Self {
        CubieCube {
            corner_perm: [0, 1, 2, 3, 4, 5, 6, 7],
            corner_orient: [0; 8],
            edge_perm: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            edge_orient: [0; 12],
        }
    }
}

// ============================================================================
// 3.1 Parse facelet string
// ============================================================================

fn parse(input: &str) -> Result<[u8; 54], String> {
    // Strip all whitespace (not just trim)
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let cleaned = cleaned.to_uppercase();

    if cleaned.len() != 54 {
        return Err(format!(
            "Invalid input: expected 54 characters, got {}",
            cleaned.len()
        ));
    }

    let mut facelet = [0u8; 54];
    for (i, ch) in cleaned.chars().enumerate() {
        match ch {
            'U' => facelet[i] = 0,
            'R' => facelet[i] = 1,
            'F' => facelet[i] = 2,
            'D' => facelet[i] = 3,
            'L' => facelet[i] = 4,
            'B' => facelet[i] = 5,
            _ => {
                return Err(format!(
                    "Invalid character '{}' at position {}",
                    ch, i
                ));
            }
        }
    }

    Ok(facelet)
}

// ============================================================================
// 3.2 Facelets to cubies
// ============================================================================

fn facelets_to_cubies(facelet: &[u8; 54]) -> CubieCube {
    let mut cube = CubieCube {
        corner_perm: [0; 8],
        corner_orient: [0; 8],
        edge_perm: [0; 12],
        edge_orient: [0; 12],
    };

    // Determine each corner
    for slot in 0..8 {
        let f0 = facelet[CORNER_FACELETS[slot][0]];
        let f1 = facelet[CORNER_FACELETS[slot][1]];
        let f2 = facelet[CORNER_FACELETS[slot][2]];

        for c in 0..8 {
            let cc = CORNER_COLORS[c];
            // Check if {f0, f1, f2} == set(cc) — same three colors in any order
            let mut match_found = true;
            let fs = [f0, f1, f2];
            let mut used = [false; 3];
            for &color in &cc {
                let mut found = false;
                for j in 0..3 {
                    if !used[j] && fs[j] == color {
                        used[j] = true;
                        found = true;
                        break;
                    }
                }
                if !found {
                    match_found = false;
                    break;
                }
            }
            if match_found {
                cube.corner_perm[slot] = c as u8;
                // Orientation: which facelet position has the U/D color
                if f0 == cc[0] {
                    cube.corner_orient[slot] = 0;
                } else if f1 == cc[0] {
                    cube.corner_orient[slot] = 1;
                } else {
                    cube.corner_orient[slot] = 2;
                }
                break;
            }
        }
    }

    // Determine each edge
    for slot in 0..12 {
        let f0 = facelet[EDGE_FACELETS[slot][0]];
        let f1 = facelet[EDGE_FACELETS[slot][1]];

        for e in 0..12 {
            let ec = EDGE_COLORS[e];
            if (f0 == ec[0] && f1 == ec[1]) || (f0 == ec[1] && f1 == ec[0]) {
                cube.edge_perm[slot] = e as u8;
                if f0 == ec[0] {
                    cube.edge_orient[slot] = 0;
                } else {
                    cube.edge_orient[slot] = 1;
                }
                break;
            }
        }
    }

    cube
}

// ============================================================================
// 3.3 Cubies to coordinates
// ============================================================================

fn encode_co(cube: &CubieCube) -> u16 {
    let mut val: u16 = 0;
    for i in 0..7 {
        val = val * 3 + cube.corner_orient[i] as u16;
    }
    val
}

fn encode_eo(cube: &CubieCube) -> u16 {
    let mut val: u16 = 0;
    for i in 0..11 {
        val = val * 2 + cube.edge_orient[i] as u16;
    }
    val
}

fn c_n_k(n: usize, k: usize) -> u16 {
    if k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    let k = k.min(n - k);
    let mut result: u64 = 1;
    for i in 0..k {
        result = result * (n - i) as u64 / (i + 1) as u64;
    }
    result as u16
}

fn encode_ud_slice(cube: &CubieCube) -> u16 {
    // Which 4 of 12 edge positions contain a slice edge (edge_perm[pos] >= 8)
    let mut positions = Vec::new();
    for pos in 0..12 {
        if cube.edge_perm[pos] >= 8 {
            positions.push(pos);
        }
    }
    positions.sort();
    c_n_k(positions[0], 1) + c_n_k(positions[1], 2) + c_n_k(positions[2], 3) + c_n_k(positions[3], 4)
}

fn encode_cp(cube: &CubieCube) -> u16 {
    let mut val: u16 = 0;
    for i in 0..8 {
        let mut k = 0u16;
        for j in (i + 1)..8 {
            if cube.corner_perm[j] < cube.corner_perm[i] {
                k += 1;
            }
        }
        val = val * (8 - i) as u16 + k;
    }
    val
}

fn encode_ep(cube: &CubieCube) -> u16 {
    // Lehmer code of the 8 non-slice edge permutation (edges in positions 0-7)
    let perm8: Vec<u8> = (0..8).map(|i| cube.edge_perm[i]).collect();
    let mut val: u16 = 0;
    for i in 0..8 {
        let mut k = 0u16;
        for j in (i + 1)..8 {
            if perm8[j] < perm8[i] {
                k += 1;
            }
        }
        val = val * (8 - i) as u16 + k;
    }
    val
}

fn encode_usp(cube: &CubieCube) -> u8 {
    // Lehmer code of the 4 slice edges' permutation within positions 8-11
    let perm4: Vec<u8> = (8..12).map(|i| cube.edge_perm[i] - 8).collect();
    let mut val: u8 = 0;
    for i in 0..4 {
        let mut k = 0u8;
        for j in (i + 1)..4 {
            if perm4[j] < perm4[i] {
                k += 1;
            }
        }
        val = val * (4 - i) as u8 + k;
    }
    val
}

// Decode functions for move table generation

fn decode_co(val: u16) -> [u8; 8] {
    let mut orient = [0u8; 8];
    let mut v = val;
    let mut sum = 0u8;
    for i in (0..7).rev() {
        orient[i] = (v % 3) as u8;
        sum += orient[i];
        v /= 3;
    }
    orient[7] = (3 - sum % 3) % 3;
    orient
}

fn decode_eo(val: u16) -> [u8; 12] {
    let mut orient = [0u8; 12];
    let mut v = val;
    let mut sum = 0u8;
    for i in (0..11).rev() {
        orient[i] = (v % 2) as u8;
        sum += orient[i];
        v /= 2;
    }
    orient[11] = (2 - sum % 2) % 2;
    orient
}

fn decode_ud_slice(val: u16) -> [u8; 12] {
    // Decode combinadic to get which 4 positions have slice edges
    let mut positions = [0usize; 4];
    let mut v = val;

    // Decode in reverse: find p3 first (the largest), then p2, p1, p0
    let mut k = 3;
    let mut n = 11;
    loop {
        let c = c_n_k(n, k + 1);
        if c <= v {
            positions[k] = n;
            v -= c;
            if k == 0 {
                break;
            }
            k -= 1;
        }
        if n == 0 {
            break;
        }
        n -= 1;
    }

    // Build edge_perm: slice edges (8-11) at decoded positions, non-slice (0-7) elsewhere
    let mut perm = [0u8; 12];
    let mut slice_idx = 8u8;
    let mut non_slice_idx = 0u8;
    for i in 0..12 {
        if positions.contains(&i) {
            perm[i] = slice_idx;
            slice_idx += 1;
        } else {
            perm[i] = non_slice_idx;
            non_slice_idx += 1;
        }
    }
    perm
}

fn decode_cp(val: u16) -> [u8; 8] {
    let mut perm = [0u8; 8];
    let mut v = val;
    let mut used = [false; 8];

    for i in 0..8 {
        let factorial = factorial_u16(7 - i as u32);
        let idx = v / factorial;
        v %= factorial;

        // Find the idx-th unused value
        let mut count = 0u16;
        for j in 0..8 {
            if !used[j] {
                if count == idx {
                    perm[i] = j as u8;
                    used[j] = true;
                    break;
                }
                count += 1;
            }
        }
    }
    perm
}

fn decode_ep(val: u16) -> [u8; 8] {
    let mut perm = [0u8; 8];
    let mut v = val;
    let mut used = [false; 8];

    for i in 0..8 {
        let factorial = factorial_u16(7 - i as u32);
        let idx = v / factorial;
        v %= factorial;

        let mut count = 0u16;
        for j in 0..8 {
            if !used[j] {
                if count == idx {
                    perm[i] = j as u8;
                    used[j] = true;
                    break;
                }
                count += 1;
            }
        }
    }
    perm
}

fn decode_usp(val: u8) -> [u8; 4] {
    let mut perm = [0u8; 4];
    let mut v = val;
    let mut used = [false; 4];

    for i in 0..4 {
        let factorial = factorial_u8(3 - i as u32);
        let idx = v / factorial;
        v %= factorial;

        let mut count = 0u8;
        for j in 0..4 {
            if !used[j] {
                if count == idx {
                    perm[i] = j as u8;
                    used[j] = true;
                    break;
                }
                count += 1;
            }
        }
    }
    perm
}

fn factorial_u16(n: u32) -> u16 {
    match n {
        0 => 1,
        1 => 1,
        2 => 2,
        3 => 6,
        4 => 24,
        5 => 120,
        6 => 720,
        7 => 5040,
        _ => panic!("factorial too large for u16"),
    }
}

fn factorial_u8(n: u32) -> u8 {
    match n {
        0 => 1,
        1 => 1,
        2 => 2,
        3 => 6,
        _ => panic!("factorial too large for u8"),
    }
}

// ============================================================================
// 3.4 Validate
// ============================================================================

fn validate(facelet: &[u8; 54], cube: &CubieCube) -> Result<(), String> {
    // Check 1: Sticker counts
    let mut counts = [0u8; 6];
    for &f in facelet.iter() {
        counts[f as usize] += 1;
    }
    for c in 0..6 {
        if counts[c] != 9 {
            return Err(format!(
                "Invalid sticker count for {}: found {} (expected 9)",
                COLOR_NAMES[c], counts[c]
            ));
        }
    }

    // Check 2: Centers
    let center_positions = [4, 13, 22, 31, 40, 49];
    for (i, &pos) in center_positions.iter().enumerate() {
        if facelet[pos] != i as u8 {
            return Err(format!(
                "Invalid center at position {}: expected {} but found {}",
                pos, COLOR_NAMES[i], COLOR_NAMES[facelet[pos] as usize]
            ));
        }
    }

    // Check 3: Corner orientation parity
    let co_sum: u8 = cube.corner_orient.iter().sum();
    if co_sum % 3 != 0 {
        return Err(format!(
            "Corner orientation parity error (sum={})",
            co_sum
        ));
    }

    // Check 4: Edge orientation parity
    let eo_sum: u8 = cube.edge_orient.iter().sum();
    if eo_sum % 2 != 0 {
        return Err(format!(
            "Edge orientation parity error (sum={})",
            eo_sum
        ));
    }

    // Check 5: Permutation parity
    let cp_parity = parity(&cube.corner_perm);
    let ep_parity = parity(&cube.edge_perm);
    if cp_parity != ep_parity {
        return Err("Permutation parity mismatch".to_string());
    }

    // Check 6: Uniqueness
    let mut corner_seen = [false; 8];
    for i in 0..8 {
        let c = cube.corner_perm[i] as usize;
        if corner_seen[c] {
            return Err(format!("Duplicate corner cubie {}", c));
        }
        corner_seen[c] = true;
    }
    let mut edge_seen = [false; 12];
    for i in 0..12 {
        let e = cube.edge_perm[i] as usize;
        if edge_seen[e] {
            return Err(format!("Duplicate edge cubie {}", e));
        }
        edge_seen[e] = true;
    }

    Ok(())
}

fn parity(perm: &[u8]) -> bool {
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
                even = !even;
            }
        }
    }
    even
}

// ============================================================================
// 3.5 Apply move to CubieCube
// ============================================================================

fn apply_cw(cube: &CubieCube, face: usize) -> CubieCube {
    let mut result = cube.clone();
    let cycle = CORNER_CYCLES[face];
    let orient_d = CORNER_ORIENT_DELTAS[face];

    result.corner_perm[cycle[0]] = cube.corner_perm[cycle[3]];
    result.corner_perm[cycle[1]] = cube.corner_perm[cycle[0]];
    result.corner_perm[cycle[2]] = cube.corner_perm[cycle[1]];
    result.corner_perm[cycle[3]] = cube.corner_perm[cycle[2]];
    result.corner_orient[cycle[0]] = (cube.corner_orient[cycle[3]] + orient_d[0]) % 3;
    result.corner_orient[cycle[1]] = (cube.corner_orient[cycle[0]] + orient_d[1]) % 3;
    result.corner_orient[cycle[2]] = (cube.corner_orient[cycle[1]] + orient_d[2]) % 3;
    result.corner_orient[cycle[3]] = (cube.corner_orient[cycle[2]] + orient_d[3]) % 3;

    let cycle = EDGE_CYCLES[face];
    let orient_d = EDGE_ORIENT_DELTAS[face];

    result.edge_perm[cycle[0]] = cube.edge_perm[cycle[3]];
    result.edge_perm[cycle[1]] = cube.edge_perm[cycle[0]];
    result.edge_perm[cycle[2]] = cube.edge_perm[cycle[1]];
    result.edge_perm[cycle[3]] = cube.edge_perm[cycle[2]];
    result.edge_orient[cycle[0]] = (cube.edge_orient[cycle[3]] + orient_d[0]) % 2;
    result.edge_orient[cycle[1]] = (cube.edge_orient[cycle[0]] + orient_d[1]) % 2;
    result.edge_orient[cycle[2]] = (cube.edge_orient[cycle[1]] + orient_d[2]) % 2;
    result.edge_orient[cycle[3]] = (cube.edge_orient[cycle[2]] + orient_d[3]) % 2;

    result
}

fn apply_move(cube: &CubieCube, move_id: u8) -> CubieCube {
    let face = (move_id / 3) as usize;
    let variant = move_id % 3;
    match variant {
        0 => apply_cw(cube, face),
        1 => apply_cw(&apply_cw(&apply_cw(cube, face), face), face), // CCW = CW^3
        2 => apply_cw(&apply_cw(cube, face), face),                  // double = CW^2
        _ => unreachable!(),
    }
}

// ============================================================================
// 3.5 Move tables
// ============================================================================

struct Tables {
    co_move: Vec<[u16; 18]>,   // [2187][18]
    eo_move: Vec<[u16; 18]>,   // [2048][18]
    uds_move: Vec<[u16; 18]>,  // [495][18]
    cp_move: Vec<[u16; 10]>,   // [40320][10]
    ep_move: Vec<[u16; 10]>,   // [40320][10]
    usp_move: Vec<[u8; 10]>,   // [24][10]
    // Pruning tables
    co_uds_prune: Vec<u8>,  // 2187*495 entries, 4 bits each
    eo_uds_prune: Vec<u8>,  // 2048*495 entries, 4 bits each
    cp_usp_prune: Vec<u8>,  // 40320*24 entries, 4 bits each
    ep_usp_prune: Vec<u8>,  // 40320*24 entries, 4 bits each
}

fn generate_tables() -> Tables {
    // Phase 1 move tables
    let co_move = generate_co_move_table();
    let eo_move = generate_eo_move_table();
    let uds_move = generate_uds_move_table();

    // Phase 2 move tables
    let cp_move = generate_cp_move_table();
    let ep_move = generate_ep_move_table();
    let usp_move = generate_usp_move_table();

    // Pruning tables
    let co_uds_prune = generate_pruning_table_phase1(&co_move, &uds_move, 2187, 495);
    let eo_uds_prune = generate_pruning_table_phase1(&eo_move, &uds_move, 2048, 495);
    let cp_usp_prune = generate_pruning_table_phase2(&cp_move, &usp_move, 40320, 24);
    let ep_usp_prune = generate_pruning_table_phase2(&ep_move, &usp_move, 40320, 24);

    Tables {
        co_move,
        eo_move,
        uds_move,
        cp_move,
        ep_move,
        usp_move,
        co_uds_prune,
        eo_uds_prune,
        cp_usp_prune,
        ep_usp_prune,
    }
}

fn generate_co_move_table() -> Vec<[u16; 18]> {
    let mut table = vec![[0u16; 18]; 2187];
    for co_val in 0..2187u16 {
        let orient = decode_co(co_val);
        let mut cube = CubieCube::solved();
        cube.corner_orient = orient;

        for move_id in 0..18u8 {
            let new_cube = apply_move(&cube, move_id);
            table[co_val as usize][move_id as usize] = encode_co(&new_cube);
        }
    }
    table
}

fn generate_eo_move_table() -> Vec<[u16; 18]> {
    let mut table = vec![[0u16; 18]; 2048];
    for eo_val in 0..2048u16 {
        let orient = decode_eo(eo_val);
        let mut cube = CubieCube::solved();
        cube.edge_orient = orient;

        for move_id in 0..18u8 {
            let new_cube = apply_move(&cube, move_id);
            table[eo_val as usize][move_id as usize] = encode_eo(&new_cube);
        }
    }
    table
}

fn generate_uds_move_table() -> Vec<[u16; 18]> {
    let mut table = vec![[0u16; 18]; 495];
    for uds_val in 0..495u16 {
        let edge_perm = decode_ud_slice(uds_val);
        let mut cube = CubieCube::solved();
        cube.edge_perm = edge_perm;

        for move_id in 0..18u8 {
            let new_cube = apply_move(&cube, move_id);
            table[uds_val as usize][move_id as usize] = encode_ud_slice(&new_cube);
        }
    }
    table
}

fn generate_cp_move_table() -> Vec<[u16; 10]> {
    let mut table = vec![[0u16; 10]; 40320];
    for cp_val in 0..40320u16 {
        let perm = decode_cp(cp_val);
        let mut cube = CubieCube::solved();
        cube.corner_perm = perm;

        for (p2_idx, &move_id) in PHASE2_MOVE_IDS.iter().enumerate() {
            let new_cube = apply_move(&cube, move_id);
            table[cp_val as usize][p2_idx] = encode_cp(&new_cube);
        }
    }
    table
}

fn generate_ep_move_table() -> Vec<[u16; 10]> {
    let mut table = vec![[0u16; 10]; 40320];
    for ep_val in 0..40320u16 {
        let perm8 = decode_ep(ep_val);
        let mut cube = CubieCube::solved();
        for i in 0..8 {
            cube.edge_perm[i] = perm8[i];
        }
        // Slice edges stay at 8-11 in identity order
        for i in 8..12 {
            cube.edge_perm[i] = i as u8;
        }

        for (p2_idx, &move_id) in PHASE2_MOVE_IDS.iter().enumerate() {
            let new_cube = apply_move(&cube, move_id);
            table[ep_val as usize][p2_idx] = encode_ep(&new_cube);
        }
    }
    table
}

fn generate_usp_move_table() -> Vec<[u8; 10]> {
    let mut table = vec![[0u8; 10]; 24];
    for usp_val in 0..24u8 {
        let perm4 = decode_usp(usp_val);
        let mut cube = CubieCube::solved();
        for i in 0..4 {
            cube.edge_perm[8 + i] = perm4[i] + 8;
        }

        for (p2_idx, &move_id) in PHASE2_MOVE_IDS.iter().enumerate() {
            let new_cube = apply_move(&cube, move_id);
            table[usp_val as usize][p2_idx] = encode_usp(&new_cube);
        }
    }
    table
}

// ============================================================================
// 3.6 Pruning tables
// ============================================================================

fn get_prune_entry(table: &[u8], idx: usize) -> u8 {
    let byte = table[idx / 2];
    if idx % 2 == 0 {
        byte & 0x0F
    } else {
        (byte >> 4) & 0x0F
    }
}

fn set_prune_entry(table: &mut [u8], idx: usize, val: u8) {
    let byte_idx = idx / 2;
    if idx % 2 == 0 {
        table[byte_idx] = (table[byte_idx] & 0xF0) | (val & 0x0F);
    } else {
        table[byte_idx] = (table[byte_idx] & 0x0F) | ((val & 0x0F) << 4);
    }
}

fn generate_pruning_table_phase1(
    move_table_a: &[[u16; 18]],
    move_table_b: &[[u16; 18]],
    size_a: usize,
    size_b: usize,
) -> Vec<u8> {
    let total = size_a * size_b;
    let byte_size = (total + 1) / 2;
    let mut prune = vec![0xFFu8; byte_size];

    // Goal: a=0, b=UDS_GOAL (494)
    let goal_idx = 0 * size_b + UDS_GOAL as usize;
    set_prune_entry(&mut prune, goal_idx, 0);
    let mut filled = 1usize;
    let mut depth = 0u8;

    while filled < total && depth < 20 {
        for idx in 0..total {
            if get_prune_entry(&prune, idx) == depth {
                let a = idx / size_b;
                let b = idx % size_b;
                for move_id in 0..18u8 {
                    let new_a = move_table_a[a][move_id as usize] as usize;
                    let new_b = move_table_b[b][move_id as usize] as usize;
                    let new_idx = new_a * size_b + new_b;
                    if get_prune_entry(&prune, new_idx) == 0x0F {
                        set_prune_entry(&mut prune, new_idx, depth + 1);
                        filled += 1;
                    }
                }
            }
        }
        depth += 1;
    }

    prune
}

fn generate_pruning_table_phase2(
    move_table_a: &[[u16; 10]],
    move_table_b: &[[u8; 10]],
    size_a: usize,
    size_b: usize,
) -> Vec<u8> {
    let total = size_a * size_b;
    let byte_size = (total + 1) / 2;
    let mut prune = vec![0xFFu8; byte_size];

    // Goal: a=0, b=0
    set_prune_entry(&mut prune, 0, 0);
    let mut filled = 1usize;
    let mut depth = 0u8;

    while filled < total && depth < 20 {
        for idx in 0..total {
            if get_prune_entry(&prune, idx) == depth {
                let a = idx / size_b;
                let b = idx % size_b;
                for p2_move in 0..10 {
                    let new_a = move_table_a[a][p2_move] as usize;
                    let new_b = move_table_b[b][p2_move] as usize;
                    let new_idx = new_a * size_b + new_b;
                    if get_prune_entry(&prune, new_idx) == 0x0F {
                        set_prune_entry(&mut prune, new_idx, depth + 1);
                        filled += 1;
                    }
                }
            }
        }
        depth += 1;
    }

    prune
}

// ============================================================================
// 3.7 IDA* Search
// ============================================================================

fn are_opposite(f1: u8, f2: u8) -> bool {
    // U(0)/D(1), R(2)/L(3), F(4)/B(5)
    (f1 / 2 == f2 / 2) && (f1 != f2)
}

// Phase 1 search: find ONE solution at exactly the given depth
fn phase1_search_at_depth(
    co: u16,
    eo: u16,
    uds: u16,
    tables: &Tables,
    depth_limit: u8,
) -> Vec<Vec<u8>> {
    let mut solutions = Vec::new();
    let h = phase1_heuristic(co, eo, uds, tables);
    if h > depth_limit {
        return solutions;
    }

    let mut path = Vec::new();
    phase1_dfs(
        co,
        eo,
        uds,
        0,
        depth_limit,
        &mut path,
        tables,
        255, // no last face
        &mut solutions,
    );
    solutions
}

fn phase1_heuristic(co: u16, eo: u16, uds: u16, tables: &Tables) -> u8 {
    let h1 = get_prune_entry(
        &tables.co_uds_prune,
        co as usize * 495 + uds as usize,
    );
    let h2 = get_prune_entry(
        &tables.eo_uds_prune,
        eo as usize * 495 + uds as usize,
    );
    h1.max(h2)
}

fn phase1_dfs(
    co: u16,
    eo: u16,
    uds: u16,
    depth: u8,
    limit: u8,
    path: &mut Vec<u8>,
    tables: &Tables,
    last_face: u8,
    solutions: &mut Vec<Vec<u8>>,
) {
    let h = phase1_heuristic(co, eo, uds, tables);
    if depth + h > limit {
        return;
    }

    if co == 0 && eo == 0 && uds == UDS_GOAL {
        solutions.push(path.clone());
        return;
    }

    for move_id in 0..18u8 {
        let face = move_id / 3;
        if face == last_face {
            continue;
        }
        if last_face != 255 && are_opposite(face, last_face) && face > last_face {
            continue;
        }

        let new_co = tables.co_move[co as usize][move_id as usize];
        let new_eo = tables.eo_move[eo as usize][move_id as usize];
        let new_uds = tables.uds_move[uds as usize][move_id as usize];

        path.push(move_id);
        phase1_dfs(new_co, new_eo, new_uds, depth + 1, limit, path, tables, face, solutions);
        path.pop();

        // Only find one solution per depth limit for efficiency
        if !solutions.is_empty() {
            return;
        }
    }
}

// Phase 2 search
fn phase2_search(
    cp: u16,
    ep: u16,
    usp: u8,
    tables: &Tables,
    max_depth: u8,
    last_face_from_p1: u8,
) -> Option<Vec<u8>> {
    if cp == 0 && ep == 0 && usp == 0 {
        return Some(Vec::new());
    }

    let h = phase2_heuristic(cp, ep, usp, tables);
    if h > max_depth {
        return None;
    }

    for depth_limit in h..=max_depth {
        let mut path = Vec::new();
        if let Some(solution) = phase2_dfs(cp, ep, usp, 0, depth_limit, &mut path, tables, last_face_from_p1) {
            return Some(solution);
        }
    }
    None
}

fn phase2_heuristic(cp: u16, ep: u16, usp: u8, tables: &Tables) -> u8 {
    let h1 = get_prune_entry(
        &tables.cp_usp_prune,
        cp as usize * 24 + usp as usize,
    );
    let h2 = get_prune_entry(
        &tables.ep_usp_prune,
        ep as usize * 24 + usp as usize,
    );
    h1.max(h2)
}

fn phase2_dfs(
    cp: u16,
    ep: u16,
    usp: u8,
    depth: u8,
    limit: u8,
    path: &mut Vec<u8>,
    tables: &Tables,
    last_face: u8,
) -> Option<Vec<u8>> {
    let h = phase2_heuristic(cp, ep, usp, tables);
    if depth + h > limit {
        return None;
    }

    if cp == 0 && ep == 0 && usp == 0 {
        return Some(path.clone());
    }

    for p2_idx in 0..10 {
        let move_id = PHASE2_MOVE_IDS[p2_idx];
        let face = move_id / 3;
        if face == last_face {
            continue;
        }
        if last_face != 255 && are_opposite(face, last_face) && face > last_face {
            continue;
        }

        let new_cp = tables.cp_move[cp as usize][p2_idx];
        let new_ep = tables.ep_move[ep as usize][p2_idx];
        let new_usp = tables.usp_move[usp as usize][p2_idx];

        path.push(move_id);
        if let Some(solution) = phase2_dfs(new_cp, new_ep, new_usp, depth + 1, limit, path, tables, face) {
            return Some(solution);
        }
        path.pop();
    }

    None
}

// ============================================================================
// 3.8 Two-phase solve
// ============================================================================

enum SolveResult {
    AlreadySolved,
    Solved(Vec<u8>),
    #[allow(dead_code)]
    Unsolvable(String),
}

fn solve(cube: &CubieCube, tables: &Tables) -> SolveResult {
    let co = encode_co(cube);
    let eo = encode_eo(cube);
    let uds = encode_ud_slice(cube);

    // Check already solved
    if co == 0
        && eo == 0
        && uds == UDS_GOAL
        && encode_cp(cube) == 0
        && encode_ep(cube) == 0
        && encode_usp(cube) == 0
    {
        return SolveResult::AlreadySolved;
    }

    let mut best_solution: Option<Vec<u8>> = None;
    let mut best_length: usize = 26; // accept solutions up to 25 moves

    let p1_h = phase1_heuristic(co, eo, uds, tables);

    for p1_limit in p1_h..13.min(best_length as u8) {
        let p1_solutions = phase1_search_at_depth(co, eo, uds, tables, p1_limit);

        for p1_solution in &p1_solutions {
            if p1_solution.len() >= best_length {
                continue;
            }

            // Apply phase-1 moves to get G1-state cube
            let mut cube_g1 = cube.clone();
            for &m in p1_solution {
                cube_g1 = apply_move(&cube_g1, m);
            }

            let cp = encode_cp(&cube_g1);
            let ep = encode_ep(&cube_g1);
            let usp = encode_usp(&cube_g1);

            let remaining = best_length - p1_solution.len() - 1;

            // Get last face from phase 1 for move pruning at junction
            let last_face = if p1_solution.is_empty() {
                255u8
            } else {
                p1_solution[p1_solution.len() - 1] / 3
            };

            if let Some(p2_moves) = phase2_search(cp, ep, usp, tables, remaining as u8, last_face) {
                let mut total = p1_solution.clone();
                total.extend_from_slice(&p2_moves);

                if total.len() < best_length {
                    best_length = total.len();
                    best_solution = Some(total);
                }
            }
        }
    }

    match best_solution {
        Some(moves) => SolveResult::Solved(moves),
        None => SolveResult::Unsolvable("No solution found within 25 moves".to_string()),
    }
}

// ============================================================================
// 3.9 Output formatting
// ============================================================================

fn format_solution(moves: &[u8]) -> String {
    moves
        .iter()
        .map(|&m| MOVE_NAMES[m as usize])
        .collect::<Vec<_>>()
        .join(" ")
}

// ============================================================================
// 3.10 CLI entry point
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = if args.len() > 1 {
        args[1].clone()
    } else {
        // Read from stdin
        let stdin = io::stdin();
        match stdin.lock().lines().next() {
            Some(Ok(line)) => line,
            _ => {
                eprintln!("Error: no input provided");
                process::exit(1);
            }
        }
    };

    // Parse
    let facelet = match parse(&input) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Convert to cubies
    let cube = facelets_to_cubies(&facelet);

    // Validate
    if let Err(e) = validate(&facelet, &cube) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // Generate tables
    let tables = generate_tables();

    // Solve
    match solve(&cube, &tables) {
        SolveResult::AlreadySolved => {
            println!("Already solved");
        }
        SolveResult::Solved(moves) => {
            println!("{}", format_solution(&moves));
        }
        SolveResult::Unsolvable(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(2);
        }
    }
}
