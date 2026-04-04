// Foundry Chess Engine 0.1.0
// A bitboard-based chess engine with UCI protocol support.

use std::io::{self, BufRead, Write as IoWrite};
use std::time::Instant;

// ============================================================
// Constants and types
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

const ALL_PIECE_TYPES: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King,
];

impl PieceType {
    fn index(self) -> usize {
        self as usize
    }
    fn from_index(i: usize) -> PieceType {
        ALL_PIECE_TYPES[i]
    }
}

type Bitboard = u64;

fn bit(sq: u8) -> Bitboard {
    1u64 << sq
}

fn sq_rank(sq: u8) -> u8 {
    sq / 8
}

fn sq_file(sq: u8) -> u8 {
    sq % 8
}

fn sq_from_rf(rank: u8, file: u8) -> u8 {
    rank * 8 + file
}

fn algebraic_to_sq(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let file = bytes[0].wrapping_sub(b'a');
    let rank = bytes[1].wrapping_sub(b'1');
    if file < 8 && rank < 8 {
        Some(rank * 8 + file)
    } else {
        None
    }
}

fn sq_to_algebraic(sq: u8) -> String {
    let file = (b'a' + sq_file(sq)) as char;
    let rank = (b'1' + sq_rank(sq)) as char;
    format!("{}{}", file, rank)
}

// Castling right bits
const WK: u8 = 1;
const WQ: u8 = 2;
const BK: u8 = 4;
const BQ: u8 = 8;

// ============================================================
// Move representation
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Move {
    from: u8,
    to: u8,
    promotion: Option<PieceType>,
}

impl Move {
    fn new(from: u8, to: u8) -> Move {
        Move { from, to, promotion: None }
    }
    fn with_promotion(from: u8, to: u8, promo: PieceType) -> Move {
        Move { from, to, promotion: Some(promo) }
    }
    fn to_uci(&self) -> String {
        let mut s = format!("{}{}", sq_to_algebraic(self.from), sq_to_algebraic(self.to));
        if let Some(p) = self.promotion {
            s.push(match p {
                PieceType::Knight => 'n',
                PieceType::Bishop => 'b',
                PieceType::Rook => 'r',
                PieceType::Queen => 'q',
                _ => unreachable!(),
            });
        }
        s
    }
    fn from_uci(s: &str, board: &Board) -> Option<Move> {
        if s.len() < 4 {
            return None;
        }
        let from = algebraic_to_sq(&s[0..2])?;
        let to = algebraic_to_sq(&s[2..4])?;
        let promotion = if s.len() > 4 {
            match s.as_bytes()[4] {
                b'n' => Some(PieceType::Knight),
                b'b' => Some(PieceType::Bishop),
                b'r' => Some(PieceType::Rook),
                b'q' => Some(PieceType::Queen),
                _ => None,
            }
        } else {
            None
        };
        // Check if this is a pawn reaching the back rank without explicit promotion
        // (shouldn't happen in valid UCI, but handle gracefully)
        let _ = board;
        Some(Move { from, to, promotion })
    }
    fn is_null(&self) -> bool {
        self.from == self.to
    }
}

const NULL_MOVE: Move = Move { from: 0, to: 0, promotion: None };

// ============================================================
// Zobrist hashing
// ============================================================

struct Zobrist {
    piece_sq: [[[u64; 64]; 6]; 2],  // [color][piece_type][sq]
    side_to_move: u64,
    castling: [u64; 16],
    en_passant: [u64; 8],  // per file
}

// Simple pseudo-random number generator for deterministic Zobrist keys
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn init_zobrist() -> Zobrist {
    let mut state: u64 = 0x12345678_9ABCDEF0;
    let mut z = Zobrist {
        piece_sq: [[[0u64; 64]; 6]; 2],
        side_to_move: 0,
        castling: [0u64; 16],
        en_passant: [0u64; 8],
    };
    for color in 0..2 {
        for pt in 0..6 {
            for sq in 0..64 {
                z.piece_sq[color][pt][sq] = xorshift64(&mut state);
            }
        }
    }
    z.side_to_move = xorshift64(&mut state);
    for i in 0..16 {
        z.castling[i] = xorshift64(&mut state);
    }
    for i in 0..8 {
        z.en_passant[i] = xorshift64(&mut state);
    }
    z
}

// ============================================================
// Board
// ============================================================

#[derive(Clone)]
struct Board {
    pieces: [[Bitboard; 6]; 2],
    occupancy: [Bitboard; 2],
    all_occupancy: Bitboard,
    mailbox: [Option<(Color, PieceType)>; 64],
    side_to_move: Color,
    castling_rights: u8,
    en_passant_square: Option<u8>,
    halfmove_clock: u16,
    fullmove_number: u16,
    zobrist_hash: u64,
}

struct UndoInfo {
    captured: Option<(Color, PieceType)>,
    castling_rights: u8,
    en_passant_square: Option<u8>,
    halfmove_clock: u16,
    zobrist_hash: u64,
}

impl Board {
    fn new() -> Board {
        Board {
            pieces: [[0; 6]; 2],
            occupancy: [0; 2],
            all_occupancy: 0,
            mailbox: [None; 64],
            side_to_move: Color::White,
            castling_rights: 0,
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_hash: 0,
        }
    }

    fn place_piece(&mut self, color: Color, pt: PieceType, sq: u8) {
        let b = bit(sq);
        self.pieces[color.index()][pt.index()] |= b;
        self.occupancy[color.index()] |= b;
        self.all_occupancy |= b;
        self.mailbox[sq as usize] = Some((color, pt));
    }

    fn remove_piece(&mut self, color: Color, pt: PieceType, sq: u8) {
        let b = bit(sq);
        self.pieces[color.index()][pt.index()] &= !b;
        self.occupancy[color.index()] &= !b;
        self.all_occupancy &= !b;
        self.mailbox[sq as usize] = None;
    }

    fn compute_zobrist(&self, zobrist: &Zobrist) -> u64 {
        let mut h = 0u64;
        for sq in 0..64u8 {
            if let Some((color, pt)) = self.mailbox[sq as usize] {
                h ^= zobrist.piece_sq[color.index()][pt.index()][sq as usize];
            }
        }
        if self.side_to_move == Color::Black {
            h ^= zobrist.side_to_move;
        }
        h ^= zobrist.castling[self.castling_rights as usize];
        if let Some(ep) = self.en_passant_square {
            h ^= zobrist.en_passant[sq_file(ep) as usize];
        }
        h
    }

    fn king_sq(&self, color: Color) -> u8 {
        self.pieces[color.index()][PieceType::King.index()].trailing_zeros() as u8
    }
}

// ============================================================
// FEN parsing and output
// ============================================================

fn parse_fen(fen: &str, zobrist: &Zobrist) -> Result<Board, String> {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    if fields.len() < 4 {
        return Err("Too few fields in FEN".to_string());
    }

    let mut board = Board::new();

    // Field 1: piece placement
    let ranks: Vec<&str> = fields[0].split('/').collect();
    if ranks.len() != 8 {
        return Err("Wrong rank count in FEN".to_string());
    }

    for (rank_idx, rank_str) in ranks.iter().enumerate() {
        let mut file: u8 = 0;
        for ch in rank_str.chars() {
            if ch.is_ascii_digit() {
                let skip = ch as u8 - b'0';
                file += skip;
                if file > 8 {
                    return Err(format!("Rank {} overflows (file count > 8)", rank_idx));
                }
            } else {
                if file >= 8 {
                    return Err(format!("Rank {} overflows (too many pieces)", rank_idx));
                }
                let (color, pt) = match ch {
                    'P' => (Color::White, PieceType::Pawn),
                    'N' => (Color::White, PieceType::Knight),
                    'B' => (Color::White, PieceType::Bishop),
                    'R' => (Color::White, PieceType::Rook),
                    'Q' => (Color::White, PieceType::Queen),
                    'K' => (Color::White, PieceType::King),
                    'p' => (Color::Black, PieceType::Pawn),
                    'n' => (Color::Black, PieceType::Knight),
                    'b' => (Color::Black, PieceType::Bishop),
                    'r' => (Color::Black, PieceType::Rook),
                    'q' => (Color::Black, PieceType::Queen),
                    'k' => (Color::Black, PieceType::King),
                    _ => return Err(format!("Bad piece char '{}' in FEN", ch)),
                };
                let sq = sq_from_rf(7 - rank_idx as u8, file);
                board.place_piece(color, pt, sq);
                file += 1;
            }
        }
        if file != 8 {
            return Err(format!("Rank {} length {} != 8", rank_idx, file));
        }
    }

    // Field 2: side to move
    board.side_to_move = match fields[1] {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err(format!("Invalid side to move '{}'", fields[1])),
    };

    // Field 3: castling rights
    board.castling_rights = 0;
    if fields[2] != "-" {
        for ch in fields[2].chars() {
            match ch {
                'K' => board.castling_rights |= WK,
                'Q' => board.castling_rights |= WQ,
                'k' => board.castling_rights |= BK,
                'q' => board.castling_rights |= BQ,
                _ => return Err(format!("Invalid castling char '{}'", ch)),
            }
        }
    }

    // Field 4: en passant
    if fields[3] != "-" {
        board.en_passant_square = Some(
            algebraic_to_sq(fields[3]).ok_or_else(|| format!("Invalid en passant '{}'", fields[3]))?
        );
    }

    // Field 5-6: halfmove clock, fullmove number (optional)
    if fields.len() > 4 {
        board.halfmove_clock = fields[4].parse().map_err(|_| "Invalid halfmove clock".to_string())?;
    }
    if fields.len() > 5 {
        board.fullmove_number = fields[5].parse().map_err(|_| "Invalid fullmove number".to_string())?;
    }

    board.zobrist_hash = board.compute_zobrist(zobrist);
    Ok(board)
}

fn board_to_fen(board: &Board) -> String {
    let mut fen = String::new();
    for rank in (0..8u8).rev() {
        let mut empty = 0;
        for file in 0..8u8 {
            let sq = sq_from_rf(rank, file);
            if let Some((color, pt)) = board.mailbox[sq as usize] {
                if empty > 0 {
                    fen.push(char::from_digit(empty, 10).unwrap());
                    empty = 0;
                }
                let ch = match pt {
                    PieceType::Pawn => 'p',
                    PieceType::Knight => 'n',
                    PieceType::Bishop => 'b',
                    PieceType::Rook => 'r',
                    PieceType::Queen => 'q',
                    PieceType::King => 'k',
                };
                fen.push(if color == Color::White { ch.to_ascii_uppercase() } else { ch });
            } else {
                empty += 1;
            }
        }
        if empty > 0 {
            fen.push(char::from_digit(empty, 10).unwrap());
        }
        if rank > 0 {
            fen.push('/');
        }
    }

    fen.push(' ');
    fen.push(if board.side_to_move == Color::White { 'w' } else { 'b' });

    fen.push(' ');
    if board.castling_rights == 0 {
        fen.push('-');
    } else {
        if board.castling_rights & WK != 0 { fen.push('K'); }
        if board.castling_rights & WQ != 0 { fen.push('Q'); }
        if board.castling_rights & BK != 0 { fen.push('k'); }
        if board.castling_rights & BQ != 0 { fen.push('q'); }
    }

    fen.push(' ');
    match board.en_passant_square {
        Some(sq) => fen.push_str(&sq_to_algebraic(sq)),
        None => fen.push('-'),
    }

    fen.push_str(&format!(" {} {}", board.halfmove_clock, board.fullmove_number));
    fen
}

// ============================================================
// Attack tables
// ============================================================

struct AttackTables {
    knight_attacks: [Bitboard; 64],
    king_attacks: [Bitboard; 64],
    pawn_attacks: [[Bitboard; 64]; 2],
    // For sliding pieces we use classical approach (ray-based)
    // We store ray masks for each direction
}

const FILE_A: Bitboard = 0x0101010101010101;
const FILE_B: Bitboard = 0x0202020202020202;
const FILE_G: Bitboard = 0x4040404040404040;
const FILE_H: Bitboard = 0x8080808080808080;
const NOT_FILE_A: Bitboard = !FILE_A;
const NOT_FILE_H: Bitboard = !FILE_H;
const NOT_FILE_AB: Bitboard = !(FILE_A | FILE_B);
const NOT_FILE_GH: Bitboard = !(FILE_G | FILE_H);

fn init_attack_tables() -> AttackTables {
    let mut tables = AttackTables {
        knight_attacks: [0; 64],
        king_attacks: [0; 64],
        pawn_attacks: [[0; 64]; 2],
    };

    for sq in 0..64u8 {
        let b = bit(sq);
        // Knight attacks
        let mut n = 0u64;
        n |= (b << 17) & NOT_FILE_A;
        n |= (b << 15) & NOT_FILE_H;
        n |= (b << 10) & NOT_FILE_AB;
        n |= (b << 6) & NOT_FILE_GH;
        n |= (b >> 17) & NOT_FILE_H;
        n |= (b >> 15) & NOT_FILE_A;
        n |= (b >> 10) & NOT_FILE_GH;
        n |= (b >> 6) & NOT_FILE_AB;
        tables.knight_attacks[sq as usize] = n;

        // King attacks
        let mut k = 0u64;
        k |= (b << 8);
        k |= (b >> 8);
        k |= (b << 1) & NOT_FILE_A;
        k |= (b >> 1) & NOT_FILE_H;
        k |= (b << 9) & NOT_FILE_A;
        k |= (b << 7) & NOT_FILE_H;
        k |= (b >> 9) & NOT_FILE_H;
        k |= (b >> 7) & NOT_FILE_A;
        tables.king_attacks[sq as usize] = k;

        // White pawn attacks (captures go up)
        let mut wp = 0u64;
        wp |= (b << 7) & NOT_FILE_H;
        wp |= (b << 9) & NOT_FILE_A;
        tables.pawn_attacks[Color::White.index()][sq as usize] = wp;

        // Black pawn attacks (captures go down)
        let mut bp = 0u64;
        bp |= (b >> 7) & NOT_FILE_A;
        bp |= (b >> 9) & NOT_FILE_H;
        tables.pawn_attacks[Color::Black.index()][sq as usize] = bp;
    }

    tables
}

// ============================================================
// Sliding piece attacks (classical approach with ray scanning)
// ============================================================

// Direction vectors for rays
const DIRS_BISHOP: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const DIRS_ROOK: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

fn ray_attacks(sq: u8, occ: Bitboard, dr: i8, df: i8) -> Bitboard {
    let mut attacks = 0u64;
    let mut r = sq_rank(sq) as i8 + dr;
    let mut f = sq_file(sq) as i8 + df;
    while r >= 0 && r < 8 && f >= 0 && f < 8 {
        let s = sq_from_rf(r as u8, f as u8);
        attacks |= bit(s);
        if occ & bit(s) != 0 {
            break;
        }
        r += dr;
        f += df;
    }
    attacks
}

fn bishop_attacks(sq: u8, occ: Bitboard) -> Bitboard {
    let mut attacks = 0u64;
    for &(dr, df) in &DIRS_BISHOP {
        attacks |= ray_attacks(sq, occ, dr, df);
    }
    attacks
}

fn rook_attacks(sq: u8, occ: Bitboard) -> Bitboard {
    let mut attacks = 0u64;
    for &(dr, df) in &DIRS_ROOK {
        attacks |= ray_attacks(sq, occ, dr, df);
    }
    attacks
}

fn queen_attacks(sq: u8, occ: Bitboard) -> Bitboard {
    bishop_attacks(sq, occ) | rook_attacks(sq, occ)
}

// ============================================================
// Attack detection
// ============================================================

fn is_attacked(board: &Board, sq: u8, by_color: Color, tables: &AttackTables) -> bool {
    let them = by_color.index();
    let occ = board.all_occupancy;

    // Knight attacks
    if tables.knight_attacks[sq as usize] & board.pieces[them][PieceType::Knight.index()] != 0 {
        return true;
    }
    // Pawn attacks (from sq perspective, check opposite color's pawn attack pattern)
    if tables.pawn_attacks[by_color.opposite().index()][sq as usize] & board.pieces[them][PieceType::Pawn.index()] != 0 {
        return true;
    }
    // King attacks
    if tables.king_attacks[sq as usize] & board.pieces[them][PieceType::King.index()] != 0 {
        return true;
    }
    // Bishop/queen attacks
    if bishop_attacks(sq, occ) & (board.pieces[them][PieceType::Bishop.index()] | board.pieces[them][PieceType::Queen.index()]) != 0 {
        return true;
    }
    // Rook/queen attacks
    if rook_attacks(sq, occ) & (board.pieces[them][PieceType::Rook.index()] | board.pieces[them][PieceType::Queen.index()]) != 0 {
        return true;
    }

    false
}

fn is_in_check(board: &Board, color: Color, tables: &AttackTables) -> bool {
    let king_bb = board.pieces[color.index()][PieceType::King.index()];
    if king_bb == 0 {
        return false; // No king on board (shouldn't happen in valid positions)
    }
    let king_sq = king_bb.trailing_zeros() as u8;
    is_attacked(board, king_sq, color.opposite(), tables)
}

// ============================================================
// Make / Unmake move
// ============================================================

// Table for castling rights updates on move from/to a square
fn castling_rights_mask(sq: u8) -> u8 {
    match sq {
        0 => !WQ,   // a1 rook
        7 => !WK,   // h1 rook
        4 => !(WK | WQ),  // e1 king
        56 => !BQ,  // a8 rook
        63 => !BK,  // h8 rook
        60 => !(BK | BQ), // e8 king
        _ => !0,
    }
}

fn make_move(board: &mut Board, mv: Move, zobrist: &Zobrist) -> UndoInfo {
    let us = board.side_to_move;
    let them = us.opposite();

    let undo = UndoInfo {
        captured: board.mailbox[mv.to as usize],
        castling_rights: board.castling_rights,
        en_passant_square: board.en_passant_square,
        halfmove_clock: board.halfmove_clock,
        zobrist_hash: board.zobrist_hash,
    };

    let (_, moving_pt) = board.mailbox[mv.from as usize].unwrap();

    // Remove en passant from hash
    if let Some(ep) = board.en_passant_square {
        board.zobrist_hash ^= zobrist.en_passant[sq_file(ep) as usize];
    }

    // Remove castling from hash
    board.zobrist_hash ^= zobrist.castling[board.castling_rights as usize];

    // Handle capture
    let mut captured_sq = mv.to;
    let is_en_passant = moving_pt == PieceType::Pawn && board.en_passant_square == Some(mv.to) && sq_file(mv.from) != sq_file(mv.to);

    if is_en_passant {
        // En passant: the captured pawn is on a different square
        captured_sq = if us == Color::White { mv.to - 8 } else { mv.to + 8 };
        let cap_pt = PieceType::Pawn;
        board.remove_piece(them, cap_pt, captured_sq);
        board.zobrist_hash ^= zobrist.piece_sq[them.index()][cap_pt.index()][captured_sq as usize];
    } else if let Some((cap_color, cap_pt)) = board.mailbox[mv.to as usize] {
        board.remove_piece(cap_color, cap_pt, mv.to);
        board.zobrist_hash ^= zobrist.piece_sq[cap_color.index()][cap_pt.index()][mv.to as usize];
    }

    // Remove piece from source
    board.remove_piece(us, moving_pt, mv.from);
    board.zobrist_hash ^= zobrist.piece_sq[us.index()][moving_pt.index()][mv.from as usize];

    // Place piece at destination (handle promotion)
    let dest_pt = mv.promotion.unwrap_or(moving_pt);
    board.place_piece(us, dest_pt, mv.to);
    board.zobrist_hash ^= zobrist.piece_sq[us.index()][dest_pt.index()][mv.to as usize];

    // Handle castling rook move
    if moving_pt == PieceType::King {
        let diff = mv.to as i8 - mv.from as i8;
        if diff == 2 {
            // Kingside castling
            let rook_from = mv.from + 3;
            let rook_to = mv.from + 1;
            board.remove_piece(us, PieceType::Rook, rook_from);
            board.place_piece(us, PieceType::Rook, rook_to);
            board.zobrist_hash ^= zobrist.piece_sq[us.index()][PieceType::Rook.index()][rook_from as usize];
            board.zobrist_hash ^= zobrist.piece_sq[us.index()][PieceType::Rook.index()][rook_to as usize];
        } else if diff == -2 {
            // Queenside castling
            let rook_from = mv.from - 4;
            let rook_to = mv.from - 1;
            board.remove_piece(us, PieceType::Rook, rook_from);
            board.place_piece(us, PieceType::Rook, rook_to);
            board.zobrist_hash ^= zobrist.piece_sq[us.index()][PieceType::Rook.index()][rook_from as usize];
            board.zobrist_hash ^= zobrist.piece_sq[us.index()][PieceType::Rook.index()][rook_to as usize];
        }
    }

    // Update castling rights
    board.castling_rights &= castling_rights_mask(mv.from);
    board.castling_rights &= castling_rights_mask(mv.to);

    // Update en passant square
    board.en_passant_square = None;
    if moving_pt == PieceType::Pawn {
        let diff = (mv.to as i8 - mv.from as i8).unsigned_abs();
        if diff == 16 {
            let ep_sq = if us == Color::White { mv.from + 8 } else { mv.from - 8 };
            board.en_passant_square = Some(ep_sq);
        }
    }

    // Update en passant in hash
    if let Some(ep) = board.en_passant_square {
        board.zobrist_hash ^= zobrist.en_passant[sq_file(ep) as usize];
    }

    // Update castling in hash
    board.zobrist_hash ^= zobrist.castling[board.castling_rights as usize];

    // Update halfmove clock
    if moving_pt == PieceType::Pawn || undo.captured.is_some() || is_en_passant {
        board.halfmove_clock = 0;
    } else {
        board.halfmove_clock += 1;
    }

    // Update fullmove number
    if us == Color::Black {
        board.fullmove_number += 1;
    }

    // Toggle side
    board.side_to_move = them;
    board.zobrist_hash ^= zobrist.side_to_move;

    undo
}

fn unmake_move(board: &mut Board, mv: Move, undo: &UndoInfo, zobrist: &Zobrist) {
    let them = board.side_to_move; // After toggle, 'them' was 'us' who made the move
    let us = them.opposite();
    // Actually: the side that made the move is opposite of current side_to_move
    // Let me clarify: after make_move, side_to_move was toggled.
    // So the side that MADE the move is board.side_to_move.opposite() = them.opposite()
    // Wait, let's be precise:
    // In make_move, us = board.side_to_move at call time, then side_to_move = them
    // So now board.side_to_move = them = us.opposite()
    // The mover was: us = board.side_to_move.opposite()
    // them in context of original move = board.side_to_move

    let mover = board.side_to_move.opposite();
    let opponent = board.side_to_move;

    // Remove the piece from destination
    let dest_pt = if mv.promotion.is_some() { mv.promotion.unwrap() } else {
        board.mailbox[mv.to as usize].unwrap().1
    };
    board.remove_piece(mover, dest_pt, mv.to);

    // Place original piece at source
    let orig_pt = if mv.promotion.is_some() { PieceType::Pawn } else { dest_pt };
    board.place_piece(mover, orig_pt, mv.from);

    // Restore captured piece
    let was_en_passant = orig_pt == PieceType::Pawn && undo.en_passant_square == Some(mv.to) && sq_file(mv.from) != sq_file(mv.to);

    if was_en_passant {
        let cap_sq = if mover == Color::White { mv.to - 8 } else { mv.to + 8 };
        board.place_piece(opponent, PieceType::Pawn, cap_sq);
    } else if let Some((cap_color, cap_pt)) = undo.captured {
        board.place_piece(cap_color, cap_pt, mv.to);
    }

    // Undo castling rook move
    if orig_pt == PieceType::King {
        let diff = mv.to as i8 - mv.from as i8;
        if diff == 2 {
            let rook_from = mv.from + 3;
            let rook_to = mv.from + 1;
            board.remove_piece(mover, PieceType::Rook, rook_to);
            board.place_piece(mover, PieceType::Rook, rook_from);
        } else if diff == -2 {
            let rook_from = mv.from - 4;
            let rook_to = mv.from - 1;
            board.remove_piece(mover, PieceType::Rook, rook_to);
            board.place_piece(mover, PieceType::Rook, rook_from);
        }
    }

    // Restore state
    board.side_to_move = mover;
    board.castling_rights = undo.castling_rights;
    board.en_passant_square = undo.en_passant_square;
    board.halfmove_clock = undo.halfmove_clock;
    board.zobrist_hash = undo.zobrist_hash;
    if mover == Color::Black {
        board.fullmove_number -= 1;
    }
}

// ============================================================
// Move generation
// ============================================================

fn generate_pseudo_legal_moves(board: &Board, tables: &AttackTables) -> Vec<Move> {
    let mut moves = Vec::with_capacity(256);
    let us = board.side_to_move;
    let them = us.opposite();
    let own = board.occupancy[us.index()];
    let enemy = board.occupancy[them.index()];
    let occ = board.all_occupancy;
    let empty = !occ;

    // Pawns
    let pawns = board.pieces[us.index()][PieceType::Pawn.index()];
    let (push_dir, start_rank_bb, promo_rank): (i8, Bitboard, u8) = if us == Color::White {
        (8, 0x000000000000FF00, 7)
    } else {
        (-8, 0x00FF000000000000, 0)
    };

    // Single pushes
    let single_push = if us == Color::White {
        (pawns << 8) & empty
    } else {
        (pawns >> 8) & empty
    };

    let mut bb = single_push;
    while bb != 0 {
        let to = bb.trailing_zeros() as u8;
        bb &= bb - 1;
        let from = (to as i8 - push_dir) as u8;
        if sq_rank(to) == promo_rank {
            moves.push(Move::with_promotion(from, to, PieceType::Queen));
            moves.push(Move::with_promotion(from, to, PieceType::Rook));
            moves.push(Move::with_promotion(from, to, PieceType::Bishop));
            moves.push(Move::with_promotion(from, to, PieceType::Knight));
        } else {
            moves.push(Move::new(from, to));
        }
    }

    // Double pushes
    let double_push = if us == Color::White {
        ((((pawns & start_rank_bb) << 8) & empty) << 8) & empty
    } else {
        ((((pawns & start_rank_bb) >> 8) & empty) >> 8) & empty
    };

    bb = double_push;
    while bb != 0 {
        let to = bb.trailing_zeros() as u8;
        bb &= bb - 1;
        let from = (to as i8 - 2 * push_dir) as u8;
        moves.push(Move::new(from, to));
    }

    // Pawn captures
    let (left_cap, right_cap) = if us == Color::White {
        ((pawns << 7) & NOT_FILE_H & enemy, (pawns << 9) & NOT_FILE_A & enemy)
    } else {
        ((pawns >> 9) & NOT_FILE_H & enemy, (pawns >> 7) & NOT_FILE_A & enemy)
    };

    let left_dir: i8 = if us == Color::White { 7 } else { -9 };
    bb = left_cap;
    while bb != 0 {
        let to = bb.trailing_zeros() as u8;
        bb &= bb - 1;
        let from = (to as i8 - left_dir) as u8;
        if sq_rank(to) == promo_rank {
            moves.push(Move::with_promotion(from, to, PieceType::Queen));
            moves.push(Move::with_promotion(from, to, PieceType::Rook));
            moves.push(Move::with_promotion(from, to, PieceType::Bishop));
            moves.push(Move::with_promotion(from, to, PieceType::Knight));
        } else {
            moves.push(Move::new(from, to));
        }
    }

    let right_dir: i8 = if us == Color::White { 9 } else { -7 };
    bb = right_cap;
    while bb != 0 {
        let to = bb.trailing_zeros() as u8;
        bb &= bb - 1;
        let from = (to as i8 - right_dir) as u8;
        if sq_rank(to) == promo_rank {
            moves.push(Move::with_promotion(from, to, PieceType::Queen));
            moves.push(Move::with_promotion(from, to, PieceType::Rook));
            moves.push(Move::with_promotion(from, to, PieceType::Bishop));
            moves.push(Move::with_promotion(from, to, PieceType::Knight));
        } else {
            moves.push(Move::new(from, to));
        }
    }

    // En passant
    if let Some(ep_sq) = board.en_passant_square {
        let ep_bb = bit(ep_sq);
        // Pawns that can capture en passant
        let ep_attackers = tables.pawn_attacks[them.index()][ep_sq as usize] & pawns;
        let mut att = ep_attackers;
        while att != 0 {
            let from = att.trailing_zeros() as u8;
            att &= att - 1;
            moves.push(Move::new(from, ep_sq));
        }
    }

    // Knights
    let mut knights = board.pieces[us.index()][PieceType::Knight.index()];
    while knights != 0 {
        let from = knights.trailing_zeros() as u8;
        knights &= knights - 1;
        let mut targets = tables.knight_attacks[from as usize] & !own;
        while targets != 0 {
            let to = targets.trailing_zeros() as u8;
            targets &= targets - 1;
            moves.push(Move::new(from, to));
        }
    }

    // Bishops
    let mut bishops = board.pieces[us.index()][PieceType::Bishop.index()];
    while bishops != 0 {
        let from = bishops.trailing_zeros() as u8;
        bishops &= bishops - 1;
        let mut targets = bishop_attacks(from, occ) & !own;
        while targets != 0 {
            let to = targets.trailing_zeros() as u8;
            targets &= targets - 1;
            moves.push(Move::new(from, to));
        }
    }

    // Rooks
    let mut rooks = board.pieces[us.index()][PieceType::Rook.index()];
    while rooks != 0 {
        let from = rooks.trailing_zeros() as u8;
        rooks &= rooks - 1;
        let mut targets = rook_attacks(from, occ) & !own;
        while targets != 0 {
            let to = targets.trailing_zeros() as u8;
            targets &= targets - 1;
            moves.push(Move::new(from, to));
        }
    }

    // Queens
    let mut queens = board.pieces[us.index()][PieceType::Queen.index()];
    while queens != 0 {
        let from = queens.trailing_zeros() as u8;
        queens &= queens - 1;
        let mut targets = queen_attacks(from, occ) & !own;
        while targets != 0 {
            let to = targets.trailing_zeros() as u8;
            targets &= targets - 1;
            moves.push(Move::new(from, to));
        }
    }

    // King
    let king_sq = board.king_sq(us);
    let mut targets = tables.king_attacks[king_sq as usize] & !own;
    while targets != 0 {
        let to = targets.trailing_zeros() as u8;
        targets &= targets - 1;
        moves.push(Move::new(king_sq, to));
    }

    // Castling — only generate if king is on its starting square
    if us == Color::White && king_sq == 4 {
        if board.castling_rights & WK != 0 {
            // f1=5, g1=6 must be empty; e1=4, f1=5, g1=6 not attacked
            if occ & (bit(5) | bit(6)) == 0
                && !is_attacked(board, 4, Color::Black, tables)
                && !is_attacked(board, 5, Color::Black, tables)
                && !is_attacked(board, 6, Color::Black, tables)
            {
                moves.push(Move::new(4, 6));
            }
        }
        if board.castling_rights & WQ != 0 {
            // b1=1, c1=2, d1=3 must be empty; c1=2, d1=3, e1=4 not attacked
            if occ & (bit(1) | bit(2) | bit(3)) == 0
                && !is_attacked(board, 4, Color::Black, tables)
                && !is_attacked(board, 3, Color::Black, tables)
                && !is_attacked(board, 2, Color::Black, tables)
            {
                moves.push(Move::new(4, 2));
            }
        }
    } else if us == Color::Black && king_sq == 60 {
        if board.castling_rights & BK != 0 {
            // f8=61, g8=62 must be empty; e8=60, f8=61, g8=62 not attacked
            if occ & (bit(61) | bit(62)) == 0
                && !is_attacked(board, 60, Color::White, tables)
                && !is_attacked(board, 61, Color::White, tables)
                && !is_attacked(board, 62, Color::White, tables)
            {
                moves.push(Move::new(60, 62));
            }
        }
        if board.castling_rights & BQ != 0 {
            // b8=57, c8=58, d8=59 must be empty; c8=58, d8=59, e8=60 not attacked
            if occ & (bit(57) | bit(58) | bit(59)) == 0
                && !is_attacked(board, 60, Color::White, tables)
                && !is_attacked(board, 59, Color::White, tables)
                && !is_attacked(board, 58, Color::White, tables)
            {
                moves.push(Move::new(60, 58));
            }
        }
    }

    moves
}

fn generate_legal_moves(board: &mut Board, tables: &AttackTables, zobrist: &Zobrist) -> Vec<Move> {
    let pseudo_moves = generate_pseudo_legal_moves(board, tables);
    let us = board.side_to_move;
    let mut legal = Vec::with_capacity(pseudo_moves.len());

    for mv in pseudo_moves {
        let undo = make_move(board, mv, zobrist);
        // Check if the move left our king in check
        if !is_in_check(board, us, tables) {
            legal.push(mv);
        }
        unmake_move(board, mv, &undo, zobrist);
    }

    legal
}

fn generate_legal_captures(board: &mut Board, tables: &AttackTables, zobrist: &Zobrist) -> Vec<Move> {
    let pseudo_moves = generate_pseudo_legal_moves(board, tables);
    let us = board.side_to_move;
    let enemy = board.occupancy[us.opposite().index()];
    let mut legal = Vec::with_capacity(64);

    for mv in pseudo_moves {
        // Only keep captures (including en passant and promotions)
        let is_capture = enemy & bit(mv.to) != 0;
        let is_ep = board.mailbox[mv.from as usize].map_or(false, |(_, pt)| pt == PieceType::Pawn)
            && board.en_passant_square == Some(mv.to)
            && sq_file(mv.from) != sq_file(mv.to);
        let is_promotion = mv.promotion.is_some();

        if !is_capture && !is_ep && !is_promotion {
            continue;
        }

        let undo = make_move(board, mv, zobrist);
        if !is_in_check(board, us, tables) {
            legal.push(mv);
        }
        unmake_move(board, mv, &undo, zobrist);
    }

    legal
}

// ============================================================
// Perft
// ============================================================

fn perft(board: &mut Board, depth: u32, tables: &AttackTables, zobrist: &Zobrist) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board, tables, zobrist);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut count = 0u64;
    for mv in moves {
        let undo = make_move(board, mv, zobrist);
        count += perft(board, depth - 1, tables, zobrist);
        unmake_move(board, mv, &undo, zobrist);
    }
    count
}

fn perft_divide(board: &mut Board, depth: u32, tables: &AttackTables, zobrist: &Zobrist) {
    let moves = generate_legal_moves(board, tables, zobrist);
    let mut total = 0u64;

    for mv in &moves {
        let undo = make_move(board, *mv, zobrist);
        let count = if depth <= 1 { 1 } else { perft(board, depth - 1, tables, zobrist) };
        unmake_move(board, *mv, &undo, zobrist);
        println!("{}: {}", mv.to_uci(), count);
        total += count;
    }
    println!("\nTotal: {}", total);
}

// ============================================================
// Evaluation (PeSTO)
// ============================================================

const MG_VALUE: [i32; 6] = [82, 337, 365, 477, 1025, 0];
const EG_VALUE: [i32; 6] = [94, 281, 297, 512, 936, 0];

const PHASE_WEIGHT: [i32; 6] = [0, 1, 1, 2, 4, 0];
const TOTAL_PHASE: i32 = 24;

// PeSTO piece-square tables (from White's perspective)
// Index 0 = a8, index 63 = h1
// Using the well-known PeSTO tables

#[rustfmt::skip]
const MG_PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,
];

#[rustfmt::skip]
const EG_PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8, -10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const MG_KNIGHT_TABLE: [i32; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
const EG_KNIGHT_TABLE: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
const MG_BISHOP_TABLE: [i32; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
const EG_BISHOP_TABLE: [i32; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
const MG_ROOK_TABLE: [i32; 64] = [
     32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
const EG_ROOK_TABLE: [i32; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
const MG_QUEEN_TABLE: [i32; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  11, -25, -31, -50, -27,
];

#[rustfmt::skip]
const EG_QUEEN_TABLE: [i32; 64] = [
     -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
const MG_KING_TABLE: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

#[rustfmt::skip]
const EG_KING_TABLE: [i32; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43,
];

const MG_PST: [[i32; 64]; 6] = [
    MG_PAWN_TABLE,
    MG_KNIGHT_TABLE,
    MG_BISHOP_TABLE,
    MG_ROOK_TABLE,
    MG_QUEEN_TABLE,
    MG_KING_TABLE,
];

const EG_PST: [[i32; 64]; 6] = [
    EG_PAWN_TABLE,
    EG_KNIGHT_TABLE,
    EG_BISHOP_TABLE,
    EG_ROOK_TABLE,
    EG_QUEEN_TABLE,
    EG_KING_TABLE,
];

// PeSTO tables are indexed with a8=0, h1=63
// Our board uses LERF: a1=0, h8=63
// So for white: we need to map our sq to PeSTO index
// Our sq: rank=sq/8, file=sq%8. a1=0(rank0,file0), a8=56(rank7,file0)
// PeSTO: a8=0, b8=1, ..., h8=7, a7=8, ..., h1=63
// PeSTO index = (7 - rank) * 8 + file
// For black: sq ^ 56 flips rank, then same PeSTO mapping

fn pesto_index(sq: u8, color: Color) -> usize {
    let r = sq_rank(sq);
    let f = sq_file(sq);
    match color {
        Color::White => ((7 - r) * 8 + f) as usize,
        Color::Black => (r * 8 + f) as usize,
    }
}

fn evaluate(board: &Board) -> i32 {
    let mut mg = [0i32; 2];
    let mut eg = [0i32; 2];
    let mut phase = 0i32;

    for sq in 0..64u8 {
        if let Some((color, pt)) = board.mailbox[sq as usize] {
            let ci = color.index();
            let pi = pt.index();
            let tbl_idx = pesto_index(sq, color);
            mg[ci] += MG_VALUE[pi] + MG_PST[pi][tbl_idx];
            eg[ci] += EG_VALUE[pi] + EG_PST[pi][tbl_idx];
            phase += PHASE_WEIGHT[pi];
        }
    }

    let mg_score = mg[board.side_to_move.index()] - mg[board.side_to_move.opposite().index()];
    let eg_score = eg[board.side_to_move.index()] - eg[board.side_to_move.opposite().index()];

    let clamped_phase = phase.min(TOTAL_PHASE).max(0);
    (mg_score * clamped_phase + eg_score * (TOTAL_PHASE - clamped_phase)) / TOTAL_PHASE
}

// ============================================================
// Transposition table
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
struct TTEntry {
    hash: u64,
    depth: i32,
    score: i32,
    flag: TTFlag,
    best_move: Move,
}

struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    size: usize,
}

impl TranspositionTable {
    fn new(size_mb: usize) -> TranspositionTable {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        TranspositionTable {
            entries: vec![None; num_entries],
            size: num_entries,
        }
    }

    fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let idx = (hash as usize) % self.size;
        self.entries[idx].as_ref().filter(|e| e.hash == hash)
    }

    fn store(&mut self, hash: u64, depth: i32, score: i32, flag: TTFlag, best_move: Move) {
        let idx = (hash as usize) % self.size;
        // Always replace (simple scheme)
        self.entries[idx] = Some(TTEntry {
            hash,
            depth,
            score,
            flag,
            best_move,
        });
    }
}

// ============================================================
// Search
// ============================================================

const MATE_SCORE: i32 = 100_000;
const INF: i32 = MATE_SCORE + 1000;

struct SearchState {
    nodes: u64,
    start_time: Instant,
    time_limit_ms: Option<u64>,
    pv: Vec<Move>,
    stopped: bool,
}

// MVV-LVA score for move ordering
fn mvv_lva_score(board: &Board, mv: &Move) -> i32 {
    // Victim value - attacker value / 100 (so captures of more valuable pieces by less valuable are ordered first)
    let victim = board.mailbox[mv.to as usize];
    let attacker = board.mailbox[mv.from as usize];

    let victim_val = match victim {
        Some((_, pt)) => match pt {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000,
        },
        None => 0,
    };

    let attacker_val = match attacker {
        Some((_, pt)) => match pt {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000,
        },
        None => 0,
    };

    // MVV-LVA: prioritize capturing high-value pieces with low-value pieces
    victim_val * 100 - attacker_val
}

fn order_moves(moves: &mut Vec<Move>, board: &Board, tt_move: Option<Move>) {
    moves.sort_by(|a, b| {
        let a_score = move_sort_score(board, a, tt_move);
        let b_score = move_sort_score(board, b, tt_move);
        b_score.cmp(&a_score)
    });
}

fn move_sort_score(board: &Board, mv: &Move, tt_move: Option<Move>) -> i32 {
    // TT move gets highest priority
    if let Some(ttm) = tt_move {
        if mv.from == ttm.from && mv.to == ttm.to && mv.promotion == ttm.promotion {
            return 1_000_000;
        }
    }

    // Captures
    if board.mailbox[mv.to as usize].is_some() {
        return 500_000 + mvv_lva_score(board, mv);
    }

    // Promotions
    if mv.promotion.is_some() {
        return 400_000;
    }

    0
}

fn alpha_beta(
    board: &mut Board,
    mut depth: i32,
    mut alpha: i32,
    beta: i32,
    ply: i32,
    tables: &AttackTables,
    zobrist: &Zobrist,
    tt: &mut TranspositionTable,
    state: &mut SearchState,
) -> i32 {
    if state.stopped {
        return 0;
    }

    // Check time
    if let Some(limit) = state.time_limit_ms {
        if state.nodes & 2047 == 0 {
            let elapsed = state.start_time.elapsed().as_millis() as u64;
            if elapsed >= limit {
                state.stopped = true;
                return 0;
            }
        }
    }

    state.nodes += 1;

    // TT probe
    let tt_move;
    if let Some(entry) = tt.probe(board.zobrist_hash) {
        tt_move = Some(entry.best_move);
        if entry.depth >= depth {
            match entry.flag {
                TTFlag::Exact => return entry.score,
                TTFlag::LowerBound => {
                    if entry.score >= beta {
                        return entry.score;
                    }
                }
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        return entry.score;
                    }
                }
            }
        }
    } else {
        tt_move = None;
    }

    // Check extension: if in check, extend search by 1 ply
    let in_check = is_in_check(board, board.side_to_move, tables);
    if in_check {
        depth += 1;
    }

    if depth <= 0 {
        return quiescence(board, alpha, beta, ply, tables, zobrist, state);
    }

    let mut moves = generate_legal_moves(board, tables, zobrist);

    if moves.is_empty() {
        if in_check {
            return -MATE_SCORE + ply;
        } else {
            return 0;
        }
    }

    order_moves(&mut moves, board, tt_move);

    let mut best_score = -INF;
    let mut best_move = moves[0];
    let old_alpha = alpha;

    for mv in &moves {
        let undo = make_move(board, *mv, zobrist);
        let score = -alpha_beta(board, depth - 1, -beta, -alpha, ply + 1, tables, zobrist, tt, state);
        unmake_move(board, *mv, &undo, zobrist);

        if state.stopped {
            return 0;
        }

        if score > best_score {
            best_score = score;
            best_move = *mv;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    // Store in TT
    let flag = if best_score <= old_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    tt.store(board.zobrist_hash, depth, best_score, flag, best_move);

    best_score
}

fn quiescence(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    ply: i32,
    tables: &AttackTables,
    zobrist: &Zobrist,
    state: &mut SearchState,
) -> i32 {
    state.nodes += 1;

    let in_check = is_in_check(board, board.side_to_move, tables);

    // When in check, we must search all evasions (can't stand pat)
    if in_check {
        let moves = generate_legal_moves(board, tables, zobrist);
        if moves.is_empty() {
            return -MATE_SCORE + ply;
        }
        let mut best = -INF;
        for mv in &moves {
            let undo = make_move(board, *mv, zobrist);
            let score = -quiescence(board, -beta, -alpha, ply + 1, tables, zobrist, state);
            unmake_move(board, *mv, &undo, zobrist);
            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                return beta;
            }
        }
        return best;
    }

    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut captures = generate_legal_captures(board, tables, zobrist);
    order_moves(&mut captures, board, None);

    for mv in &captures {
        let undo = make_move(board, *mv, zobrist);
        let score = -quiescence(board, -beta, -alpha, ply + 1, tables, zobrist, state);
        unmake_move(board, *mv, &undo, zobrist);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn iterative_deepening(
    board: &mut Board,
    max_depth: i32,
    time_limit_ms: Option<u64>,
    tables: &AttackTables,
    zobrist: &Zobrist,
    tt: &mut TranspositionTable,
) -> Option<Move> {
    let mut state = SearchState {
        nodes: 0,
        start_time: Instant::now(),
        time_limit_ms,
        pv: Vec::new(),
        stopped: false,
    };

    let mut best_move = None;

    for depth in 1..=max_depth {
        state.nodes = 0;

        // Root search: manually iterate over moves to track best_move directly
        let mut moves = generate_legal_moves(board, tables, zobrist);
        if moves.is_empty() {
            break;
        }
        let tt_move = tt.probe(board.zobrist_hash).map(|e| e.best_move);
        order_moves(&mut moves, board, tt_move);

        let mut best_score = -INF;
        let mut root_best = moves[0];
        let mut alpha = -INF;
        let beta = INF;

        for mv in &moves {
            let undo = make_move(board, *mv, zobrist);
            let score = -alpha_beta(board, depth - 1, -beta, -alpha, 1, tables, zobrist, tt, &mut state);
            unmake_move(board, *mv, &undo, zobrist);

            if state.stopped {
                break;
            }

            if score > best_score {
                best_score = score;
                root_best = *mv;
            }
            if score > alpha {
                alpha = score;
            }
        }

        if state.stopped && depth > 1 {
            break;
        }

        // Store root result in TT
        let flag = if best_score >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };
        tt.store(board.zobrist_hash, depth, best_score, flag, root_best);

        best_move = Some(root_best);
        let score = best_score;

        let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
        let nps = if elapsed_ms > 0 { state.nodes * 1000 / elapsed_ms } else { 0 };

        // Print UCI info
        let score_str = if score.abs() > MATE_SCORE - 1000 {
            let mate_in = if score > 0 {
                (MATE_SCORE - score + 1) / 2
            } else {
                -(MATE_SCORE + score + 1) / 2
            };
            format!("score mate {}", mate_in)
        } else {
            format!("score cp {}", score)
        };

        let pv_str = if let Some(m) = best_move {
            m.to_uci()
        } else {
            String::new()
        };

        println!(
            "info depth {} {} nodes {} time {} nps {} pv {}",
            depth, score_str, state.nodes, elapsed_ms, nps, pv_str
        );

        if state.stopped {
            break;
        }

        // Check time for next iteration
        if let Some(limit) = time_limit_ms {
            if elapsed_ms * 2 > limit {
                break;
            }
        }
    }

    best_move
}

// ============================================================
// UCI protocol
// ============================================================

fn uci_loop(tables: &AttackTables, zobrist: &Zobrist) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let startpos_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut board = parse_fen(startpos_fen, zobrist).unwrap();
    let mut tt = TranspositionTable::new(16);

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "uci" => {
                writeln!(stdout, "id name Foundry Chess 0.1.0").unwrap();
                writeln!(stdout, "id author Lightless Labs").unwrap();
                writeln!(stdout, "uciok").unwrap();
                stdout.flush().unwrap();
            }
            "isready" => {
                writeln!(stdout, "readyok").unwrap();
                stdout.flush().unwrap();
            }
            "position" => {
                if parts.len() < 2 {
                    continue;
                }
                let mut move_start = 0;
                if parts[1] == "startpos" {
                    board = parse_fen(startpos_fen, zobrist).unwrap();
                    move_start = 2;
                    if parts.len() > 2 && parts[2] == "moves" {
                        move_start = 3;
                    }
                } else if parts[1] == "fen" {
                    // Collect FEN fields until "moves" or end
                    let mut fen_parts = Vec::new();
                    let mut i = 2;
                    while i < parts.len() && parts[i] != "moves" {
                        fen_parts.push(parts[i]);
                        i += 1;
                    }
                    let fen_str = fen_parts.join(" ");
                    match parse_fen(&fen_str, zobrist) {
                        Ok(b) => board = b,
                        Err(_) => continue,
                    }
                    move_start = i;
                    if move_start < parts.len() && parts[move_start] == "moves" {
                        move_start += 1;
                    }
                }

                // Apply moves
                for i in move_start..parts.len() {
                    if let Some(mv) = Move::from_uci(parts[i], &board) {
                        // Find the matching legal move (to handle promotion correctly)
                        let legal = generate_legal_moves(&mut board, tables, zobrist);
                        let found = legal.iter().find(|m| {
                            m.from == mv.from && m.to == mv.to && m.promotion == mv.promotion
                        });
                        if let Some(legal_mv) = found {
                            make_move(&mut board, *legal_mv, zobrist);
                        }
                    }
                }
            }
            "go" => {
                let mut depth = 6; // default
                let mut time_limit = None;
                let mut is_perft = false;
                let mut perft_depth = 1u32;

                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "depth" => {
                            if i + 1 < parts.len() {
                                depth = parts[i + 1].parse().unwrap_or(6);
                                i += 1;
                            }
                        }
                        "movetime" => {
                            if i + 1 < parts.len() {
                                time_limit = parts[i + 1].parse().ok();
                                i += 1;
                            }
                        }
                        "perft" => {
                            is_perft = true;
                            if i + 1 < parts.len() {
                                perft_depth = parts[i + 1].parse().unwrap_or(1);
                                i += 1;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }

                if is_perft {
                    let count = perft(&mut board, perft_depth, tables, zobrist);
                    writeln!(stdout, "{}", count).unwrap();
                    stdout.flush().unwrap();
                } else {
                    let best = iterative_deepening(&mut board, depth, time_limit, tables, zobrist, &mut tt);
                    match best {
                        Some(mv) => {
                            writeln!(stdout, "bestmove {}", mv.to_uci()).unwrap();
                        }
                        None => {
                            // No legal moves - still need to output something
                            writeln!(stdout, "bestmove 0000").unwrap();
                        }
                    }
                    stdout.flush().unwrap();
                }
            }
            "perft" => {
                // Non-standard UCI extension: perft <depth>
                if parts.len() > 1 {
                    if let Ok(d) = parts[1].parse::<u32>() {
                        let count = perft(&mut board, d, tables, zobrist);
                        writeln!(stdout, "{}", count).unwrap();
                        stdout.flush().unwrap();
                    }
                }
            }
            "quit" => {
                std::process::exit(0);
            }
            _ => {
                // Unknown command, silently ignore per UCI spec
            }
        }
    }
}

// ============================================================
// CLI entry point
// ============================================================

fn print_usage() {
    eprintln!("Usage: chess-engine [COMMAND]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  uci                  Start UCI protocol mode");
    eprintln!("  perft <depth>        Run perft test");
    eprintln!("  divide <depth>       Run perft with move breakdown");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --fen <FEN>          Position in FEN notation");
    eprintln!("  --depth <N>          Search depth (for perft/divide)");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tables = init_attack_tables();
    let zobrist = init_zobrist();

    if args.len() < 2 {
        // Default to UCI mode
        uci_loop(&tables, &zobrist);
        return;
    }

    match args[1].as_str() {
        "uci" => {
            uci_loop(&tables, &zobrist);
        }
        "perft" | "divide" => {
            let is_divide = args[1].as_str() == "divide";

            // Parse arguments
            let mut fen: Option<String> = None;
            let mut depth: Option<u32> = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--fen" => {
                        // Consume all following args until next flag or end
                        // FEN can be passed as a single quoted arg or multiple args
                        i += 1;
                        let mut fen_parts = Vec::new();
                        while i < args.len() && !args[i].starts_with("--") {
                            // Also stop if it looks like a bare depth number and we don't have enough FEN fields yet
                            fen_parts.push(args[i].as_str());
                            i += 1;
                        }
                        if fen_parts.is_empty() {
                            eprintln!("Error: --fen requires a value");
                            std::process::exit(1);
                        }
                        fen = Some(fen_parts.join(" "));
                    }
                    "--depth" => {
                        if i + 1 < args.len() {
                            depth = Some(args[i + 1].parse().unwrap_or_else(|_| {
                                eprintln!("Error: invalid depth");
                                std::process::exit(1);
                            }));
                            i += 2;
                        } else {
                            eprintln!("Error: --depth requires a value");
                            std::process::exit(1);
                        }
                    }
                    other => {
                        // Try parsing as depth if no depth set yet
                        if depth.is_none() {
                            if let Ok(d) = other.parse::<u32>() {
                                depth = Some(d);
                                i += 1;
                                continue;
                            }
                        }
                        eprintln!("Error: unknown argument '{}'", other);
                        std::process::exit(1);
                    }
                }
            }

            let depth = depth.unwrap_or_else(|| {
                eprintln!("Error: depth is required");
                std::process::exit(1);
            });

            let fen_str = fen.unwrap_or_else(|| {
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
            });

            // FEN and depth parsed successfully

            let mut board = match parse_fen(&fen_str, &zobrist) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            if is_divide {
                perft_divide(&mut board, depth, &tables, &zobrist);
            } else {
                let count = perft(&mut board, depth, &tables, &zobrist);
                println!("{}", count);
            }
        }
        "--help" | "-h" | "help" => {
            print_usage();
        }
        _ => {
            eprintln!("Error: unknown command '{}'", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}
