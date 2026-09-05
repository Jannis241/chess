//! Core chess rules and board state management.
//!
//! This module provides legal move generation, move execution, game-end
//! detection, and board rendering helpers used by the terminal UI.

use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// Side to move / piece color.
pub enum Color {
    White,
    Black,
}

impl Color {
    fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// Chess piece kind.
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// A colored chess piece on the board.
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
/// Castling rights for both sides.
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct PositionKey {
    board: [Option<Piece>; 64],
    side_to_move: Color,
    castling: CastlingRights,
    en_passant: Option<usize>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// A move represented by source/target indices and optional promotion.
pub struct Move {
    /// Source square index in `0..64`.
    pub from: usize,
    /// Destination square index in `0..64`.
    pub to: usize,
    /// Promotion piece when the move is a pawn promotion.
    pub promotion: Option<PieceKind>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MoveKind {
    Quiet,
    EnPassant,
    CastleKingside,
    CastleQueenside,
    Promotion,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct LegalMove {
    mv: Move,
    kind: MoveKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Current game outcome/status.
pub enum GameStatus {
    Ongoing,
    Checkmate { winner: Color },
    Stalemate,
    DrawFiftyMove,
    DrawThreefoldRepetition,
    DrawInsufficientMaterial,
}

#[derive(Clone)]
/// Full chess game state including repetition tracking.
pub struct Chess {
    board: [Option<Piece>; 64],
    turn: Color,
    castling: CastlingRights,
    en_passant: Option<usize>,
    last_move: Option<Move>,
    halfmove_clock: u32,
    fullmove_number: u32,
    repetition: HashMap<PositionKey, u8>,
}

impl Chess {
    /// Creates a new game in the standard starting position.
    pub fn new() -> Self {
        Self::init("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    /// Creates a game from FEN, falling back to the standard start position on parse failure.
    pub fn init(fen: &str) -> Self {
        Self::from_fen(fen).unwrap_or_else(|_| {
            Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .expect("valid initial FEN")
        })
    }

    /// Parses a strict six-field FEN string into a [`Chess`] position.
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err("FEN must have exactly 6 fields".to_string());
        }

        let mut board = [None; 64];
        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 8 {
            return Err("FEN board must have 8 ranks".to_string());
        }

        for (fen_rank, rank_str) in ranks.iter().enumerate() {
            let board_rank = 7usize
                .checked_sub(fen_rank)
                .ok_or_else(|| "Invalid FEN rank".to_string())?;
            let mut file = 0usize;
            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    let skip = ch
                        .to_digit(10)
                        .ok_or_else(|| "Invalid board digit".to_string())?
                        as usize;
                    file += skip;
                } else {
                    if file >= 8 {
                        return Err("Too many files in FEN rank".to_string());
                    }
                    let piece = piece_from_fen_char(ch)?;
                    let idx = board_rank * 8 + file;
                    board[idx] = Some(piece);
                    file += 1;
                }
            }
            if file != 8 {
                return Err("FEN rank does not sum to 8 files".to_string());
            }
        }

        let turn = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid side-to-move field".to_string()),
        };

        let mut castling = CastlingRights::default();
        if parts[2] != "-" {
            for ch in parts[2].chars() {
                match ch {
                    'K' => castling.white_kingside = true,
                    'Q' => castling.white_queenside = true,
                    'k' => castling.black_kingside = true,
                    'q' => castling.black_queenside = true,
                    _ => return Err("Invalid castling rights".to_string()),
                }
            }
        }

        let en_passant = if parts[3] == "-" {
            None
        } else {
            Some(parse_square(parts[3])?)
        };

        let halfmove_clock = parts[4]
            .parse::<u32>()
            .map_err(|_| "Invalid halfmove clock".to_string())?;
        let fullmove_number = parts[5]
            .parse::<u32>()
            .map_err(|_| "Invalid fullmove number".to_string())?;

        let mut chess = Self {
            board,
            turn,
            castling,
            en_passant,
            last_move: None,
            halfmove_clock,
            fullmove_number,
            repetition: HashMap::new(),
        };
        chess.record_position();
        Ok(chess)
    }

    /// Renders the board with no active overlays.
    pub fn render(&self) -> String {
        self.render_with_selection(usize::MAX, None, &[], &[])
    }

    /// Renders the board with cursor/selection/status overlays used by the terminal UI.
    pub fn render_with_selection(
        &self,
        cursor_sq: usize,
        selected_sq: Option<usize>,
        legal_targets: &[usize],
        checked_kings: &[usize],
    ) -> String {
        let mut legal_mask = [false; 64];
        for &sq in legal_targets {
            if sq < 64 {
                legal_mask[sq] = true;
            }
        }
        let mut checked_mask = [false; 64];
        for &sq in checked_kings {
            if sq < 64 {
                checked_mask[sq] = true;
            }
        }
        let mut last_move_mask = [false; 64];
        if let Some(mv) = self.last_move {
            if mv.from < 64 {
                last_move_mask[mv.from] = true;
            }
            if mv.to < 64 {
                last_move_mask[mv.to] = true;
            }
        }

        let mut out = String::with_capacity(2200);
        out.push_str("  +-----+-----+-----+-----+-----+-----+-----+-----+\n");
        for rank in (0..8).rev() {
            out.push_str(&format!("{} |", rank + 1));
            for file in 0..8 {
                let idx = rank * 8 + file;
                let is_cursor = idx == cursor_sq;
                let is_selected = selected_sq == Some(idx);
                let is_legal_target = legal_mask[idx];
                let is_checked_king = checked_mask[idx];
                let is_last_move = last_move_mask[idx];
                let base_bg_code = if (file + rank) % 2 == 0 {
                    "48;5;236"
                } else {
                    "48;5;242"
                };
                let bg_code = if is_checked_king {
                    "48;5;124"
                } else if is_cursor || is_selected {
                    "48;5;22"
                } else if is_legal_target {
                    "48;5;58"
                } else if is_last_move {
                    "48;5;240"
                } else {
                    base_bg_code
                };
                let piece_symbol = self.board[idx]
                    .map(piece_to_symbol)
                    .unwrap_or_else(|| if is_legal_target { '·' } else { ' ' });
                let piece_style = self.board[idx]
                    .map(piece_style_code)
                    .unwrap_or(if is_legal_target { "1;97" } else { "39" });
                let cell = format!("\x1b[{bg_code};{piece_style}m  {piece_symbol}  \x1b[0m");
                out.push_str(&cell);
                out.push('|');
            }
            out.push('\n');
            out.push_str("  +-----+-----+-----+-----+-----+-----+-----+-----+\n");
        }
        out.push_str("     a     b     c     d     e     f     g     h\n");
        out
    }

    /// Returns the side to move.
    pub fn current_turn(&self) -> Color {
        self.turn
    }

    /// Returns the most recently executed move, if any.
    pub fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    /// Returns the piece on `idx` if occupied.
    pub fn piece_at(&self, idx: usize) -> Option<Piece> {
        self.board.get(idx).copied().flatten()
    }

    /// Returns deduplicated legal target squares for a piece origin square.
    pub fn legal_targets_from(&self, from: usize) -> Vec<usize> {
        if from >= 64 {
            return Vec::new();
        }
        let Some(piece) = self.board[from] else {
            return Vec::new();
        };
        if piece.color != self.turn {
            return Vec::new();
        }

        let mut seen = [false; 64];
        let mut targets = Vec::new();
        for legal in self.legal_moves() {
            if legal.mv.from == from && !seen[legal.mv.to] {
                seen[legal.mv.to] = true;
                targets.push(legal.mv.to);
            }
        }
        targets
    }

    /// Returns king-square indices currently in check.
    pub fn checked_king_squares(&self) -> Vec<usize> {
        let mut out = Vec::new();
        for color in [Color::White, Color::Black] {
            if self.is_in_check(color)
                && let Some(king_sq) = self.king_square(color)
            {
                out.push(king_sq);
            }
        }
        out
    }

    /// Evaluates the current game status (ongoing, mate, stalemate, or draw).
    pub fn status(&self) -> GameStatus {
        if self.halfmove_clock >= 100 {
            return GameStatus::DrawFiftyMove;
        }
        if self.repetition_count() >= 3 {
            return GameStatus::DrawThreefoldRepetition;
        }
        if self.is_insufficient_material() {
            return GameStatus::DrawInsufficientMaterial;
        }

        if !self.has_any_legal_move(self.turn) {
            if self.is_in_check(self.turn) {
                return GameStatus::Checkmate {
                    winner: self.turn.opposite(),
                };
            }
            return GameStatus::Stalemate;
        }
        GameStatus::Ongoing
    }

    /// Parses and executes coordinate notation like `e2e4` or `e7e8q`.
    pub fn try_make_move_str(&mut self, input: &str) -> Result<(), String> {
        let input = input.trim().to_lowercase();
        let bytes = input.as_bytes();
        if bytes.len() < 4 || bytes.len() > 5 {
            return Err("Move must be in format e2e4 or e7e8q".to_string());
        }

        let from = parse_square(&input[0..2])?;
        let to = parse_square(&input[2..4])?;
        let promotion = if bytes.len() == 5 {
            Some(parse_promotion(bytes[4] as char)?)
        } else {
            None
        };

        let wanted = Move {
            from,
            to,
            promotion,
        };
        self.try_make_move(wanted)
    }

    /// Attempts to execute a legal move from the current position.
    pub fn try_make_move(&mut self, wanted: Move) -> Result<(), String> {
        let mut fallback = None;
        let mut queen_promotion = None;
        for lm in self.legal_moves() {
            if !same_move_with_promotion_fallback(lm, wanted) {
                continue;
            }
            if lm.mv.promotion == Some(PieceKind::Queen) {
                queen_promotion = Some(lm);
                if wanted.promotion.is_none() {
                    break;
                }
            }
            if fallback.is_none() {
                fallback = Some(lm);
            }
        }

        let Some(chosen) = queen_promotion.or(fallback) else {
            return Err("Illegal move".to_string());
        };

        self.apply_legal_move(chosen);
        Ok(())
    }

    /// Returns all legal moves for the side to move.
    pub fn legal_moves_for_turn(&self) -> Vec<Move> {
        self.legal_moves().into_iter().map(|lm| lm.mv).collect()
    }

    #[cfg(test)]
    pub fn legal_move_count(&self) -> usize {
        self.legal_moves().len()
    }

    fn legal_moves(&self) -> Vec<LegalMove> {
        let pseudo = self.pseudo_legal_moves(self.turn);
        pseudo
            .into_iter()
            .filter(|mv| {
                let mut next = self.clone();
                next.apply_legal_move(*mv);
                !next.is_in_check(self.turn)
            })
            .collect()
    }

    fn has_any_legal_move(&self, color: Color) -> bool {
        for mv in self.pseudo_legal_moves(color) {
            let mut next = self.clone();
            next.apply_legal_move(mv);
            if !next.is_in_check(color) {
                return true;
            }
        }
        false
    }

    fn pseudo_legal_moves(&self, color: Color) -> Vec<LegalMove> {
        let mut out = Vec::new();

        for from in 0..64 {
            let Some(piece) = self.board[from] else {
                continue;
            };
            if piece.color != color {
                continue;
            }

            match piece.kind {
                PieceKind::Pawn => self.gen_pawn_moves(from, color, &mut out),
                PieceKind::Knight => self.gen_knight_moves(from, color, &mut out),
                PieceKind::Bishop => self.gen_slider_moves(
                    from,
                    color,
                    &mut out,
                    &[(1, 1), (-1, 1), (1, -1), (-1, -1)],
                ),
                PieceKind::Rook => self.gen_slider_moves(
                    from,
                    color,
                    &mut out,
                    &[(1, 0), (-1, 0), (0, 1), (0, -1)],
                ),
                PieceKind::Queen => self.gen_slider_moves(
                    from,
                    color,
                    &mut out,
                    &[
                        (1, 1),
                        (-1, 1),
                        (1, -1),
                        (-1, -1),
                        (1, 0),
                        (-1, 0),
                        (0, 1),
                        (0, -1),
                    ],
                ),
                PieceKind::King => self.gen_king_moves(from, color, &mut out),
            }
        }

        out
    }

    fn gen_pawn_moves(&self, from: usize, color: Color, out: &mut Vec<LegalMove>) {
        let (file, rank) = idx_to_file_rank(from);
        let (forward, start_rank, promo_rank) = match color {
            Color::White => (1i32, 1usize, 6usize),
            Color::Black => (-1i32, 6usize, 1usize),
        };

        let one_rank = rank as i32 + forward;
        if in_bounds(file as i32, one_rank) {
            let to = file_rank_to_idx(file, one_rank as usize);
            if self.board[to].is_none() {
                if rank == promo_rank {
                    for promo in [
                        PieceKind::Queen,
                        PieceKind::Rook,
                        PieceKind::Bishop,
                        PieceKind::Knight,
                    ] {
                        out.push(LegalMove {
                            mv: Move {
                                from,
                                to,
                                promotion: Some(promo),
                            },
                            kind: MoveKind::Promotion,
                        });
                    }
                } else {
                    out.push(LegalMove {
                        mv: Move {
                            from,
                            to,
                            promotion: None,
                        },
                        kind: MoveKind::Quiet,
                    });
                }

                if rank == start_rank {
                    let two_rank = rank as i32 + forward * 2;
                    if in_bounds(file as i32, two_rank) {
                        let to2 = file_rank_to_idx(file, two_rank as usize);
                        if self.board[to2].is_none() {
                            out.push(LegalMove {
                                mv: Move {
                                    from,
                                    to: to2,
                                    promotion: None,
                                },
                                kind: MoveKind::Quiet,
                            });
                        }
                    }
                }
            }
        }

        for df in [-1i32, 1i32] {
            let nf = file as i32 + df;
            let nr = rank as i32 + forward;
            if !in_bounds(nf, nr) {
                continue;
            }
            let to = file_rank_to_idx(nf as usize, nr as usize);
            if let Some(target) = self.board[to] {
                if target.color != color {
                    if rank == promo_rank {
                        for promo in [
                            PieceKind::Queen,
                            PieceKind::Rook,
                            PieceKind::Bishop,
                            PieceKind::Knight,
                        ] {
                            out.push(LegalMove {
                                mv: Move {
                                    from,
                                    to,
                                    promotion: Some(promo),
                                },
                                kind: MoveKind::Promotion,
                            });
                        }
                    } else {
                        out.push(LegalMove {
                            mv: Move {
                                from,
                                to,
                                promotion: None,
                            },
                            kind: MoveKind::Quiet,
                        });
                    }
                }
            } else if self.en_passant == Some(to) {
                out.push(LegalMove {
                    mv: Move {
                        from,
                        to,
                        promotion: None,
                    },
                    kind: MoveKind::EnPassant,
                });
            }
        }
    }

    fn gen_knight_moves(&self, from: usize, color: Color, out: &mut Vec<LegalMove>) {
        let (file, rank) = idx_to_file_rank(from);
        let jumps = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];
        for (df, dr) in jumps {
            let nf = file as i32 + df;
            let nr = rank as i32 + dr;
            if !in_bounds(nf, nr) {
                continue;
            }
            let to = file_rank_to_idx(nf as usize, nr as usize);
            if self.board[to].is_none_or(|p| p.color != color) {
                out.push(LegalMove {
                    mv: Move {
                        from,
                        to,
                        promotion: None,
                    },
                    kind: MoveKind::Quiet,
                });
            }
        }
    }

    fn gen_slider_moves(
        &self,
        from: usize,
        color: Color,
        out: &mut Vec<LegalMove>,
        dirs: &[(i32, i32)],
    ) {
        let (file, rank) = idx_to_file_rank(from);
        for &(df, dr) in dirs {
            let mut nf = file as i32 + df;
            let mut nr = rank as i32 + dr;
            while in_bounds(nf, nr) {
                let to = file_rank_to_idx(nf as usize, nr as usize);
                match self.board[to] {
                    None => out.push(LegalMove {
                        mv: Move {
                            from,
                            to,
                            promotion: None,
                        },
                        kind: MoveKind::Quiet,
                    }),
                    Some(p) if p.color != color => {
                        out.push(LegalMove {
                            mv: Move {
                                from,
                                to,
                                promotion: None,
                            },
                            kind: MoveKind::Quiet,
                        });
                        break;
                    }
                    Some(_) => break,
                }
                nf += df;
                nr += dr;
            }
        }
    }

    fn gen_king_moves(&self, from: usize, color: Color, out: &mut Vec<LegalMove>) {
        let (file, rank) = idx_to_file_rank(from);
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let nf = file as i32 + df;
                let nr = rank as i32 + dr;
                if !in_bounds(nf, nr) {
                    continue;
                }
                let to = file_rank_to_idx(nf as usize, nr as usize);
                if self.board[to].is_none_or(|p| p.color != color) {
                    out.push(LegalMove {
                        mv: Move {
                            from,
                            to,
                            promotion: None,
                        },
                        kind: MoveKind::Quiet,
                    });
                }
            }
        }

        let (home_rank, kside_empty, qside_empty, king_path_k, king_path_q, can_ks, can_q) =
            match color {
                Color::White => (
                    0usize,
                    [5usize, 6usize],
                    [1usize, 2usize, 3usize],
                    [4usize, 5usize, 6usize],
                    [4usize, 3usize, 2usize],
                    self.castling.white_kingside,
                    self.castling.white_queenside,
                ),
                Color::Black => (
                    7usize,
                    [61usize, 62usize],
                    [57usize, 58usize, 59usize],
                    [60usize, 61usize, 62usize],
                    [60usize, 59usize, 58usize],
                    self.castling.black_kingside,
                    self.castling.black_queenside,
                ),
            };

        let king_start = home_rank * 8 + 4;
        if from != king_start {
            return;
        }

        if can_ks
            && kside_empty.iter().all(|&sq| self.board[sq].is_none())
            && king_path_k
                .iter()
                .all(|&sq| !self.is_square_attacked(sq, color.opposite()))
            && self.board[home_rank * 8 + 7]
                == Some(Piece {
                    color,
                    kind: PieceKind::Rook,
                })
        {
            out.push(LegalMove {
                mv: Move {
                    from,
                    to: home_rank * 8 + 6,
                    promotion: None,
                },
                kind: MoveKind::CastleKingside,
            });
        }

        if can_q
            && qside_empty.iter().all(|&sq| self.board[sq].is_none())
            && king_path_q
                .iter()
                .all(|&sq| !self.is_square_attacked(sq, color.opposite()))
            && self.board[home_rank * 8]
                == Some(Piece {
                    color,
                    kind: PieceKind::Rook,
                })
        {
            out.push(LegalMove {
                mv: Move {
                    from,
                    to: home_rank * 8 + 2,
                    promotion: None,
                },
                kind: MoveKind::CastleQueenside,
            });
        }
    }

    fn apply_legal_move(&mut self, legal: LegalMove) {
        let mv = legal.mv;
        let mut piece = self.board[mv.from].expect("piece must exist");
        let captured = self.board[mv.to];

        self.board[mv.from] = None;

        match legal.kind {
            MoveKind::EnPassant => {
                let cap_sq = match piece.color {
                    Color::White => mv.to - 8,
                    Color::Black => mv.to + 8,
                };
                self.board[cap_sq] = None;
                self.board[mv.to] = Some(piece);
            }
            MoveKind::CastleKingside => {
                self.board[mv.to] = Some(piece);
                let home_rank = if piece.color == Color::White { 0 } else { 7 };
                let rook_from = home_rank * 8 + 7;
                let rook_to = home_rank * 8 + 5;
                self.board[rook_to] = self.board[rook_from];
                self.board[rook_from] = None;
            }
            MoveKind::CastleQueenside => {
                self.board[mv.to] = Some(piece);
                let home_rank = if piece.color == Color::White { 0 } else { 7 };
                let rook_from = home_rank * 8;
                let rook_to = home_rank * 8 + 3;
                self.board[rook_to] = self.board[rook_from];
                self.board[rook_from] = None;
            }
            MoveKind::Promotion => {
                piece.kind = mv.promotion.expect("promotion type must be set");
                self.board[mv.to] = Some(piece);
            }
            MoveKind::Quiet => {
                self.board[mv.to] = Some(piece);
            }
        }

        self.update_castling_rights_after_move(piece, mv.from, mv.to, captured);

        self.en_passant = None;
        if piece.kind == PieceKind::Pawn {
            let from_rank = mv.from / 8;
            let to_rank = mv.to / 8;
            if (from_rank as i32 - to_rank as i32).abs() == 2 {
                self.en_passant = Some((mv.from + mv.to) / 2);
            }
        }

        let is_capture = captured.is_some() || legal.kind == MoveKind::EnPassant;
        if piece.kind == PieceKind::Pawn || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.turn == Color::Black {
            self.fullmove_number += 1;
        }
        self.turn = self.turn.opposite();
        self.last_move = Some(mv);
        self.record_position();
    }

    fn update_castling_rights_after_move(
        &mut self,
        moved_piece: Piece,
        from: usize,
        to: usize,
        captured: Option<Piece>,
    ) {
        match moved_piece.kind {
            PieceKind::King => {
                if moved_piece.color == Color::White {
                    self.castling.white_kingside = false;
                    self.castling.white_queenside = false;
                } else {
                    self.castling.black_kingside = false;
                    self.castling.black_queenside = false;
                }
            }
            PieceKind::Rook => match (moved_piece.color, from) {
                (Color::White, 0) => self.castling.white_queenside = false,
                (Color::White, 7) => self.castling.white_kingside = false,
                (Color::Black, 56) => self.castling.black_queenside = false,
                (Color::Black, 63) => self.castling.black_kingside = false,
                _ => {}
            },
            _ => {}
        }

        if let Some(cap) = captured
            && cap.kind == PieceKind::Rook
        {
            match (cap.color, to) {
                (Color::White, 0) => self.castling.white_queenside = false,
                (Color::White, 7) => self.castling.white_kingside = false,
                (Color::Black, 56) => self.castling.black_queenside = false,
                (Color::Black, 63) => self.castling.black_kingside = false,
                _ => {}
            }
        }
    }

    fn is_square_attacked(&self, sq: usize, by: Color) -> bool {
        let (file, rank) = idx_to_file_rank(sq);
        let file = file as i32;
        let rank = rank as i32;

        let pawn_dir = match by {
            Color::White => -1,
            Color::Black => 1,
        };
        for df in [-1, 1] {
            let nf = file + df;
            let nr = rank + pawn_dir;
            if in_bounds(nf, nr) {
                let idx = file_rank_to_idx(nf as usize, nr as usize);
                if self.board[idx]
                    == Some(Piece {
                        color: by,
                        kind: PieceKind::Pawn,
                    })
                {
                    return true;
                }
            }
        }

        let knight_offsets = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];
        for (df, dr) in knight_offsets {
            let nf = file + df;
            let nr = rank + dr;
            if in_bounds(nf, nr) {
                let idx = file_rank_to_idx(nf as usize, nr as usize);
                if self.board[idx]
                    == Some(Piece {
                        color: by,
                        kind: PieceKind::Knight,
                    })
                {
                    return true;
                }
            }
        }

        for (df, dr) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
            if self.ray_attacked_by(sq, by, df, dr, &[PieceKind::Bishop, PieceKind::Queen]) {
                return true;
            }
        }
        for (df, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if self.ray_attacked_by(sq, by, df, dr, &[PieceKind::Rook, PieceKind::Queen]) {
                return true;
            }
        }

        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let nf = file + df;
                let nr = rank + dr;
                if in_bounds(nf, nr) {
                    let idx = file_rank_to_idx(nf as usize, nr as usize);
                    if self.board[idx]
                        == Some(Piece {
                            color: by,
                            kind: PieceKind::King,
                        })
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn ray_attacked_by(&self, sq: usize, by: Color, df: i32, dr: i32, kinds: &[PieceKind]) -> bool {
        let (file, rank) = idx_to_file_rank(sq);
        let mut nf = file as i32 + df;
        let mut nr = rank as i32 + dr;
        while in_bounds(nf, nr) {
            let idx = file_rank_to_idx(nf as usize, nr as usize);
            if let Some(piece) = self.board[idx] {
                if piece.color == by && kinds.contains(&piece.kind) {
                    return true;
                }
                return false;
            }
            nf += df;
            nr += dr;
        }
        false
    }

    fn king_square(&self, color: Color) -> Option<usize> {
        self.board.iter().enumerate().find_map(|(i, p)| {
            if *p
                == Some(Piece {
                    color,
                    kind: PieceKind::King,
                })
            {
                Some(i)
            } else {
                None
            }
        })
    }

    fn is_in_check(&self, color: Color) -> bool {
        let Some(king_sq) = self.king_square(color) else {
            return false;
        };
        self.is_square_attacked(king_sq, color.opposite())
    }

    fn effective_en_passant(&self) -> Option<usize> {
        let ep = self.en_passant?;
        let side = self.turn;
        let (file, rank) = idx_to_file_rank(ep);
        let capture_rank = match side {
            Color::White => rank.checked_sub(1)?,
            Color::Black => rank + 1,
        };
        if capture_rank >= 8 {
            return None;
        }
        for df in [-1i32, 1i32] {
            let nf = file as i32 + df;
            if !in_bounds(nf, capture_rank as i32) {
                continue;
            }
            let from = file_rank_to_idx(nf as usize, capture_rank);
            if self.board[from]
                == Some(Piece {
                    color: side,
                    kind: PieceKind::Pawn,
                })
            {
                return Some(ep);
            }
        }
        None
    }

    fn repetition_key(&self) -> PositionKey {
        PositionKey {
            board: self.board,
            side_to_move: self.turn,
            castling: self.castling,
            en_passant: self.effective_en_passant(),
        }
    }

    fn record_position(&mut self) {
        let key = self.repetition_key();
        *self.repetition.entry(key).or_insert(0) += 1;
    }

    fn repetition_count(&self) -> u8 {
        *self.repetition.get(&self.repetition_key()).unwrap_or(&0)
    }

    fn is_insufficient_material(&self) -> bool {
        let mut white_minors = Vec::new();
        let mut black_minors = Vec::new();
        let mut white_other = 0usize;
        let mut black_other = 0usize;

        for (idx, piece) in self.board.iter().enumerate() {
            let Some(piece) = piece else {
                continue;
            };
            match piece.kind {
                PieceKind::King => {}
                PieceKind::Bishop | PieceKind::Knight => {
                    if piece.color == Color::White {
                        white_minors.push((piece.kind, idx));
                    } else {
                        black_minors.push((piece.kind, idx));
                    }
                }
                _ => {
                    if piece.color == Color::White {
                        white_other += 1;
                    } else {
                        black_other += 1;
                    }
                }
            }
        }

        if white_other > 0 || black_other > 0 {
            return false;
        }

        let white_count = white_minors.len();
        let black_count = black_minors.len();

        if white_count == 0 && black_count == 0 {
            return true;
        }
        if white_count == 1 && black_count == 0 {
            return true;
        }
        if white_count == 0 && black_count == 1 {
            return true;
        }

        if white_count == 1
            && black_count == 1
            && white_minors[0].0 == PieceKind::Bishop
            && black_minors[0].0 == PieceKind::Bishop
        {
            let white_color = square_color(white_minors[0].1);
            let black_color = square_color(black_minors[0].1);
            return white_color == black_color;
        }

        false
    }
}

fn same_move_with_promotion_fallback(candidate: LegalMove, wanted: Move) -> bool {
    if candidate.mv.from != wanted.from || candidate.mv.to != wanted.to {
        return false;
    }
    match (candidate.mv.promotion, wanted.promotion) {
        (Some(c), Some(w)) => c == w,
        (Some(_), None) => true,
        (None, None) => true,
        (None, Some(_)) => false,
    }
}

/// Parses a FEN piece symbol into a typed piece.
fn piece_from_fen_char(ch: char) -> Result<Piece, String> {
    let color = if ch.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };
    let kind = match ch.to_ascii_lowercase() {
        'p' => PieceKind::Pawn,
        'n' => PieceKind::Knight,
        'b' => PieceKind::Bishop,
        'r' => PieceKind::Rook,
        'q' => PieceKind::Queen,
        'k' => PieceKind::King,
        _ => return Err("Invalid piece character in FEN".to_string()),
    };
    Ok(Piece { color, kind })
}

/// Returns the Unicode glyph used for board rendering.
fn piece_to_symbol(piece: Piece) -> char {
    match (piece.color, piece.kind) {
        (Color::White, PieceKind::Pawn) => '♙',
        (Color::White, PieceKind::Knight) => '♘',
        (Color::White, PieceKind::Bishop) => '♗',
        (Color::White, PieceKind::Rook) => '♖',
        (Color::White, PieceKind::Queen) => '♕',
        (Color::White, PieceKind::King) => '♔',
        (Color::Black, PieceKind::Pawn) => '♟',
        (Color::Black, PieceKind::Knight) => '♞',
        (Color::Black, PieceKind::Bishop) => '♝',
        (Color::Black, PieceKind::Rook) => '♜',
        (Color::Black, PieceKind::Queen) => '♛',
        (Color::Black, PieceKind::King) => '♚',
    }
}

/// Returns ANSI style code for a piece glyph color.
fn piece_style_code(piece: Piece) -> &'static str {
    match piece.color {
        Color::White => "1;97",
        Color::Black => "1;34",
    }
}

/// Parses algebraic square text (`e4`) into board index.
fn parse_square(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err("Square must be two characters, e.g. e4".to_string());
    }
    let b = s.as_bytes();
    let file = match b[0] {
        b'a'..=b'h' => (b[0] - b'a') as usize,
        _ => return Err(format!("Invalid file in square: {s}")),
    };
    let rank = match b[1] {
        b'1'..=b'8' => (b[1] - b'1') as usize,
        _ => return Err(format!("Invalid rank in square: {s}")),
    };
    Ok(file_rank_to_idx(file, rank))
}

/// Parses coordinate-notation promotion piece character.
fn parse_promotion(ch: char) -> Result<PieceKind, String> {
    match ch.to_ascii_lowercase() {
        'q' => Ok(PieceKind::Queen),
        'r' => Ok(PieceKind::Rook),
        'b' => Ok(PieceKind::Bishop),
        'n' => Ok(PieceKind::Knight),
        _ => Err("Promotion piece must be one of q, r, b, n".to_string()),
    }
}

/// Converts board index to `(file, rank)`.
fn idx_to_file_rank(idx: usize) -> (usize, usize) {
    (idx % 8, idx / 8)
}

/// Converts `(file, rank)` to board index.
fn file_rank_to_idx(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

/// Returns whether a `(file, rank)` pair is on board.
fn in_bounds(file: i32, rank: i32) -> bool {
    (0..8).contains(&file) && (0..8).contains(&rank)
}

/// Returns square color parity used for bishop-color comparisons.
fn square_color(idx: usize) -> bool {
    let (file, rank) = idx_to_file_rank(idx);
    (file + rank) % 2 == 0
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_position_has_20_moves() {
        let game = Chess::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("valid FEN");
        assert_eq!(game.legal_move_count(), 20);
        assert_eq!(game.status(), GameStatus::Ongoing);
    }

    #[test]
    fn en_passant_is_legal_when_available() {
        let mut game =
            Chess::from_fen("rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 2")
                .expect("valid FEN");
        game.try_make_move_str("d4e5")
            .expect("en passant should be legal");
    }

    #[test]
    fn castling_moves_work() {
        let mut game = Chess::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        game.try_make_move_str("e1g1")
            .expect("white castles kingside");
        game.try_make_move_str("e8c8")
            .expect("black castles queenside");
    }

    #[test]
    fn promotion_requires_valid_piece() {
        let mut game = Chess::from_fen("7k/P7/8/8/8/8/8/K7 w - - 0 1").expect("valid FEN");
        game.try_make_move_str("a7a8q")
            .expect("promotion should succeed");
        assert_eq!(
            game.board[file_rank_to_idx(0, 7)],
            Some(Piece {
                color: Color::White,
                kind: PieceKind::Queen
            })
        );
    }

    #[test]
    fn checkmate_is_detected() {
        let mut game = Chess::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("valid FEN");
        game.try_make_move_str("f2f3").unwrap();
        game.try_make_move_str("e7e5").unwrap();
        game.try_make_move_str("g2g4").unwrap();
        game.try_make_move_str("d8h4").unwrap();
        assert_eq!(
            game.status(),
            GameStatus::Checkmate {
                winner: Color::Black
            }
        );
    }

    #[test]
    fn stalemate_is_detected() {
        let game = Chess::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").expect("valid FEN");
        assert_eq!(game.status(), GameStatus::Stalemate);
    }

    #[test]
    fn insufficient_material_is_detected() {
        let game = Chess::from_fen("8/8/8/8/8/8/6k1/7K w - - 0 1").expect("valid FEN");
        assert_eq!(game.status(), GameStatus::DrawInsufficientMaterial);
    }

    #[test]
    fn threefold_repetition_is_detected() {
        let mut game = Chess::from_fen("7k/8/8/8/8/8/8/KN6 w - - 0 1").expect("valid FEN");
        let sequence = [
            "b1c3", "h8g8", "c3b1", "g8h8", "b1c3", "h8g8", "c3b1", "g8h8",
        ];
        for mv in sequence {
            game.try_make_move_str(mv)
                .expect("move in repetition line should be legal");
        }
        assert_eq!(game.status(), GameStatus::DrawThreefoldRepetition);
    }
}
