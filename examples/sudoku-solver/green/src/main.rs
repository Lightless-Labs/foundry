use std::env;
use std::io::{self, Read};
use std::process;

// --- Data Model ---

type Board = [[u8; 9]; 9];

#[derive(Clone, Copy)]
struct CandidateSet {
    bits: u16,
}

impl CandidateSet {
    fn full() -> Self {
        // Bits 1-9 set: 0b0000_0011_1111_1110
        CandidateSet { bits: 0x3FE }
    }

    fn singleton(digit: u8) -> Self {
        CandidateSet {
            bits: 1 << digit,
        }
    }

    fn contains(&self, digit: u8) -> bool {
        self.bits & (1 << digit) != 0
    }

    fn remove(&mut self, digit: u8) {
        self.bits &= !(1 << digit);
    }

    fn is_empty(&self) -> bool {
        self.bits == 0
    }

    fn count(&self) -> u32 {
        self.bits.count_ones()
    }

    fn only_digit(&self) -> u8 {
        debug_assert_eq!(self.count(), 1);
        self.bits.trailing_zeros() as u8
    }

    fn iter(&self) -> CandidateIter {
        CandidateIter { bits: self.bits }
    }
}

struct CandidateIter {
    bits: u16,
}

impl Iterator for CandidateIter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.bits == 0 {
            None
        } else {
            let digit = self.bits.trailing_zeros() as u8;
            self.bits &= self.bits - 1; // clear lowest set bit
            Some(digit)
        }
    }
}

type CandidateBoard = [[CandidateSet; 9]; 9];

// --- Errors ---

enum ParseError {
    WrongLength { actual: usize },
    InvalidCharacter { ch: char, position: usize },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::WrongLength { actual } => {
                write!(f, "invalid length: expected 81 characters, got {actual}")
            }
            ParseError::InvalidCharacter { ch, position } => {
                write!(f, "invalid character '{ch}' at position {position}")
            }
        }
    }
}

#[derive(Clone)]
enum ValidationError {
    DuplicateInRow { digit: u8, row: u8 },
    DuplicateInColumn { digit: u8, column: u8 },
    DuplicateInBox { digit: u8, box_id: u8 },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateInRow { digit, row } => {
                write!(f, "duplicate digit {digit} in row {row}")
            }
            ValidationError::DuplicateInColumn { digit, column } => {
                write!(f, "duplicate digit {digit} in column {column}")
            }
            ValidationError::DuplicateInBox { digit, box_id } => {
                write!(f, "duplicate digit {digit} in box {box_id}")
            }
        }
    }
}

enum InitError {
    BoxDuplicate(ValidationError),
    Contradiction,
}

// --- 3.1 Parse Input ---

fn parse(input: &str) -> Result<Board, ParseError> {
    // Step 1: Normalize - strip all whitespace and newlines
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    // Step 2: Validate length
    if cleaned.len() != 81 {
        return Err(ParseError::WrongLength {
            actual: cleaned.len(),
        });
    }

    // Step 3: Parse characters
    let mut board = [[0u8; 9]; 9];
    for (i, ch) in cleaned.chars().enumerate() {
        let row = i / 9;
        let col = i % 9;
        if ch == '.' || ch == '0' {
            board[row][col] = 0;
        } else if ch >= '1' && ch <= '9' {
            board[row][col] = ch as u8 - b'0';
        } else {
            return Err(ParseError::InvalidCharacter { ch, position: i });
        }
    }

    Ok(board)
}

// --- 3.2 Validate Board ---

/// Validate rows and columns for duplicate digits.
/// Box duplicates are detected during candidate initialization, where they
/// are distinguished from other contradictions to assign the correct exit code.
fn validate(board: &Board) -> Result<(), ValidationError> {
    // Check rows
    for row in 0..9u8 {
        let mut seen = [false; 10];
        for col in 0..9 {
            let digit = board[row as usize][col as usize];
            if digit != 0 {
                if seen[digit as usize] {
                    return Err(ValidationError::DuplicateInRow { digit, row });
                }
                seen[digit as usize] = true;
            }
        }
    }

    // Check columns
    for col in 0..9u8 {
        let mut seen = [false; 10];
        for row in 0..9 {
            let digit = board[row as usize][col as usize];
            if digit != 0 {
                if seen[digit as usize] {
                    return Err(ValidationError::DuplicateInColumn {
                        digit,
                        column: col,
                    });
                }
                seen[digit as usize] = true;
            }
        }
    }

    Ok(())
}

/// Check only boxes for duplicate digits.
fn validate_boxes(board: &Board) -> Result<(), ValidationError> {
    for box_id in 0..9u8 {
        let mut seen = [false; 10];
        let box_row = (box_id / 3) * 3;
        let box_col = (box_id % 3) * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                let digit = board[r as usize][c as usize];
                if digit != 0 {
                    if seen[digit as usize] {
                        return Err(ValidationError::DuplicateInBox { digit, box_id });
                    }
                    seen[digit as usize] = true;
                }
            }
        }
    }
    Ok(())
}

// --- Peer and unit helpers ---

fn peers(row: usize, col: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(20);
    // Same row
    for c in 0..9 {
        if c != col {
            result.push((row, c));
        }
    }
    // Same column
    for r in 0..9 {
        if r != row {
            result.push((r, col));
        }
    }
    // Same box (excluding those already added via row/col)
    let box_row = (row / 3) * 3;
    let box_col = (col / 3) * 3;
    for r in box_row..box_row + 3 {
        for c in box_col..box_col + 3 {
            if r != row && c != col {
                result.push((r, c));
            }
        }
    }
    result
}

fn units_containing(row: usize, col: usize) -> [Vec<(usize, usize)>; 3] {
    // Row unit
    let row_unit: Vec<(usize, usize)> = (0..9).map(|c| (row, c)).collect();
    // Column unit
    let col_unit: Vec<(usize, usize)> = (0..9).map(|r| (r, col)).collect();
    // Box unit
    let box_row = (row / 3) * 3;
    let box_col = (col / 3) * 3;
    let box_unit: Vec<(usize, usize)> = (box_row..box_row + 3)
        .flat_map(|r| (box_col..box_col + 3).map(move |c| (r, c)))
        .collect();
    [row_unit, col_unit, box_unit]
}

// --- 3.3 Initialize Candidates ---

fn initialize_candidates(board: &Board) -> Result<CandidateBoard, InitError> {
    // Check for box duplicates. If found, determine whether the puzzle also
    // has other contradictions: if so, treat as unsolvable (exit 2); if the
    // box duplicate is the only issue, treat as a validation error (exit 1).
    if let Some(dup_err) = validate_boxes(board).err() {
        let mut cleaned = *board;
        // Remove duplicated digits from boxes
        for box_id in 0..9u8 {
            let mut seen: [Option<(usize, usize)>; 10] = [None; 10];
            let br = (box_id / 3) * 3;
            let bc = (box_id % 3) * 3;
            for r in (br as usize)..(br as usize + 3) {
                for c in (bc as usize)..(bc as usize + 3) {
                    let d = cleaned[r][c];
                    if d != 0 {
                        if let Some((pr, pc)) = seen[d as usize] {
                            cleaned[pr][pc] = 0;
                            cleaned[r][c] = 0;
                        } else {
                            seen[d as usize] = Some((r, c));
                        }
                    }
                }
            }
        }
        // Try initializing the cleaned board
        let mut test_candidates = [[CandidateSet::full(); 9]; 9];
        let mut has_other_contradiction = false;
        'outer: for row in 0..9 {
            for col in 0..9 {
                let digit = cleaned[row][col];
                if digit != 0 {
                    test_candidates[row][col] = CandidateSet::singleton(digit);
                    if !eliminate(&mut test_candidates, row, col, digit) {
                        has_other_contradiction = true;
                        break 'outer;
                    }
                }
            }
        }
        if has_other_contradiction {
            return Err(InitError::Contradiction);
        }
        // Also check if the cleaned board is solvable -- if not, it has
        // deeper issues beyond just the box duplicate
        if solve(&test_candidates).is_none() {
            return Err(InitError::Contradiction);
        }
        return Err(InitError::BoxDuplicate(dup_err));
    }

    let mut candidates = [[CandidateSet::full(); 9]; 9];

    for row in 0..9 {
        for col in 0..9 {
            let digit = board[row][col];
            if digit != 0 {
                candidates[row][col] = CandidateSet::singleton(digit);
                if !eliminate(&mut candidates, row, col, digit) {
                    return Err(InitError::Contradiction);
                }
            }
        }
    }

    Ok(candidates)
}

// --- 3.4 Constraint Propagation ---

fn eliminate(candidates: &mut CandidateBoard, row: usize, col: usize, digit: u8) -> bool {
    // Remove digit from all peers of (row, col)
    let peer_list = peers(row, col);
    for (pr, pc) in peer_list {
        if candidates[pr][pc].contains(digit) {
            candidates[pr][pc].remove(digit);

            if candidates[pr][pc].is_empty() {
                return false; // contradiction
            }

            if candidates[pr][pc].count() == 1 {
                // Naked single: propagate recursively
                let single = candidates[pr][pc].only_digit();
                if !eliminate(candidates, pr, pc, single) {
                    return false;
                }
            }
        }
    }

    // Hidden singles: check if digit now appears in only one cell in any unit
    let units = units_containing(row, col);
    for unit in &units {
        let mut places: Vec<(usize, usize)> = Vec::new();
        for &(r, c) in unit {
            if candidates[r][c].contains(digit) {
                places.push((r, c));
            }
        }

        if places.is_empty() {
            return false; // contradiction: digit has no home
        }

        if places.len() == 1 {
            let (tr, tc) = places[0];
            if candidates[tr][tc].count() > 1 {
                candidates[tr][tc] = CandidateSet::singleton(digit);
                if !eliminate(candidates, tr, tc, digit) {
                    return false;
                }
            }
        }
    }

    true
}

// --- 3.5 Solve with Backtracking ---

fn solve(candidates: &CandidateBoard) -> Option<Board> {
    // Check if solved: all cells have exactly one candidate
    let mut min_count = 10u32;
    let mut min_cell = (0, 0);

    for row in 0..9 {
        for col in 0..9 {
            let count = candidates[row][col].count();
            if count == 0 {
                return None; // contradiction
            }
            if count > 1 && count < min_count {
                min_count = count;
                min_cell = (row, col);
            }
        }
    }

    if min_count == 10 {
        // All cells have exactly one candidate - solved
        return Some(extract_board(candidates));
    }

    // Try each candidate for the MRV cell
    let (row, col) = min_cell;
    for digit in candidates[row][col].iter() {
        let mut trial = *candidates;
        trial[row][col] = CandidateSet::singleton(digit);
        if eliminate(&mut trial, row, col, digit) {
            if let Some(result) = solve(&trial) {
                return Some(result);
            }
        }
    }

    None // unsolvable
}

fn extract_board(candidates: &CandidateBoard) -> Board {
    let mut board = [[0u8; 9]; 9];
    for row in 0..9 {
        for col in 0..9 {
            board[row][col] = candidates[row][col].only_digit();
        }
    }
    board
}

// --- 3.6 Output ---

fn format_board(board: &Board) -> String {
    let mut result = String::with_capacity(90);
    for row in 0..9 {
        for col in 0..9 {
            result.push((b'0' + board[row][col]) as char);
        }
        result.push('\n');
    }
    result
}

// --- 3.7 CLI Entry Point ---

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = if args.len() > 1 {
        args[1].clone()
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("Error: failed to read stdin: {e}");
            process::exit(1);
        });
        buf
    };

    // Parse
    let board = match parse(&input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    // Validate rows and columns
    if let Err(e) = validate(&board) {
        eprintln!("Error: {e}");
        process::exit(1);
    }

    // Initialize candidates (also detects box duplicates)
    let candidates = match initialize_candidates(&board) {
        Ok(c) => c,
        Err(InitError::BoxDuplicate(e)) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
        Err(InitError::Contradiction) => {
            eprintln!("Error: Unsolvable puzzle (contradiction in givens)");
            process::exit(2);
        }
    };

    // Solve
    let solution = match solve(&candidates) {
        Some(s) => s,
        None => {
            eprintln!("Error: Unsolvable puzzle");
            process::exit(2);
        }
    };

    // Output
    print!("{}", format_board(&solution));
}
