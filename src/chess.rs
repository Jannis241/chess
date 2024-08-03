// Todo: Promotions, En passant, visuals verbessern (bei schach matt so eine linie anzeigen von wo, gewinner besser, current player besser ,figuren)

const UP: i32 = -8;
const LEFT: i32 = -1;
const RIGHT: i32 = 1;

#[derive(Clone, PartialEq, Debug)]
pub struct Move {
    pub from: usize,
    pub to: usize,
}
#[derive(PartialEq, Clone, Debug)]
pub enum Color {
    Black,
    White,
}
#[derive(Clone, PartialEq, Debug)]
pub struct Tile {
    pub pos: usize,
    pub piece_on_tile: Piece,
}
#[derive(Clone, Debug)]
pub struct Board {
    pub tiles: Vec<Tile>,
    pub current_player: Color,
    pub white_castle_queen_side: bool,
    pub white_castle_king_side: bool,
    pub black_castle_queen_side: bool,
    pub black_castle_king_side: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    pub pos: usize,
    pub move_count: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PieceType {
    Empty,
    Pawn,
    King,
    Bishop,
    Knight,
    Rook,
    Queen,
}

impl Move {
    pub fn new(from: usize, to: usize) -> Self {
        Move { from, to }
    }
}

impl Tile {
    pub fn new(pos: usize, piece_on_tile: Piece) -> Self {
        Tile { pos, piece_on_tile }
    }
}

impl Piece {
    pub fn new(color: Color, piece_type: PieceType, pos: usize) -> Self {
        Piece {
            color,
            piece_type,
            pos,
            move_count: 0,
        }
    }
    pub fn get_legal_moves(
        &self,
        board: &mut Board,
        check_if_own_piece: bool,
        checks: bool,
    ) -> Vec<Move> {
        if check_if_own_piece && self.color != board.current_player {
            return vec![];
        }

        println!("test änderung");

        let mut results = Vec::new();
        match self.piece_type {
            PieceType::Pawn => {
                let direction = if self.color == Color::White { 1 } else { -1 };
                let one_step = UP * direction;
                let two_steps = one_step * 2;

                // Check if 1 step up is empty
                let one_step_pos = self.pos as i32 + one_step;
                if board.is_in_board(one_step_pos) {
                    if board.get_piece_at_pos(one_step_pos).piece_type == PieceType::Empty {
                        results.push(Move::new(self.pos, one_step_pos as usize));

                        // Check for 2 steps up (only if at initial position)
                        if self.move_count == 0 {
                            let two_steps_pos = self.pos as i32 + two_steps;
                            if board.is_in_board(two_steps_pos) {
                                if board.get_piece_at_pos(two_steps_pos).piece_type
                                    == PieceType::Empty
                                {
                                    results.push(Move::new(self.pos, two_steps_pos as usize));
                                }
                            }
                        }
                    }
                }

                // Captures
                let left_capture_pos = self.pos as i32 + one_step + LEFT;
                let right_capture_pos = self.pos as i32 + one_step + RIGHT;

                if board.is_in_board(left_capture_pos) && !board.is_completly_left(self.pos) {
                    let p = board.get_piece_at_pos(left_capture_pos);
                    if p.color != self.color && p.piece_type != PieceType::Empty {
                        results.push(Move::new(self.pos, left_capture_pos as usize));
                    }
                }

                if board.is_in_board(right_capture_pos) && !board.is_completly_right(self.pos) {
                    let p = board.get_piece_at_pos(right_capture_pos);
                    if p.color != self.color && p.piece_type != PieceType::Empty {
                        results.push(Move::new(self.pos, right_capture_pos as usize));
                    }
                }
            }
            PieceType::Knight => {
                let directions = [
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, 2),
                    (1, -2),
                    (-1, 2),
                    (-1, -2),
                ];

                for &(dir_x, dir_y) in &directions {
                    let new_x = (self.pos % 8) as isize + dir_x;
                    let new_y = (self.pos / 8) as isize + dir_y;

                    if new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8 {
                        let new_pos = (new_y * 8 + new_x) as usize;
                        let piece = board.get_piece_at_pos(new_pos as i32);

                        if piece.piece_type == PieceType::Empty || piece.color != self.color {
                            results.push(Move::new(self.pos, new_pos));
                        }
                    }
                }
            }
            PieceType::Rook => {
                let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

                for &(dx, dy) in &directions {
                    let mut x = (self.pos % 8) as isize;
                    let mut y = (self.pos / 8) as isize;

                    loop {
                        x += dx;
                        y += dy;

                        if x < 0 || x >= 8 || y < 0 || y >= 8 {
                            break;
                        }

                        let new_pos = (y * 8 + x) as usize;
                        let piece = board.get_piece_at_pos(new_pos as i32);

                        if piece.piece_type == PieceType::Empty {
                            results.push(Move::new(self.pos, new_pos));
                        } else {
                            if piece.color != self.color {
                                results.push(Move::new(self.pos, new_pos));
                            }
                            break;
                        }
                    }
                }
            }
            PieceType::Bishop => {
                let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

                for &(dx, dy) in &directions {
                    let mut x = (self.pos % 8) as isize;
                    let mut y = (self.pos / 8) as isize;

                    loop {
                        x += dx;
                        y += dy;

                        if x < 0 || x >= 8 || y < 0 || y >= 8 {
                            break;
                        }

                        let new_pos = (y * 8 + x) as usize;
                        let piece = board.get_piece_at_pos(new_pos as i32);

                        if piece.piece_type == PieceType::Empty {
                            results.push(Move::new(self.pos, new_pos));
                        } else {
                            if piece.color != self.color {
                                results.push(Move::new(self.pos, new_pos));
                            }
                            break;
                        }
                    }
                }
            }
            PieceType::Queen => {
                let directions = [
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1), // Rook directions
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1), // Bishop directions
                ];

                for &(dx, dy) in &directions {
                    let mut x = (self.pos % 8) as isize;
                    let mut y = (self.pos / 8) as isize;

                    loop {
                        x += dx;
                        y += dy;

                        if x < 0 || x >= 8 || y < 0 || y >= 8 {
                            break;
                        }

                        let new_pos = (y * 8 + x) as usize;
                        let piece = board.get_piece_at_pos(new_pos as i32);

                        if piece.piece_type == PieceType::Empty {
                            results.push(Move::new(self.pos, new_pos));
                        } else {
                            if piece.color != self.color {
                                results.push(Move::new(self.pos, new_pos));
                            }
                            break;
                        }
                    }
                }
            }
            PieceType::King => {
                let directions = [
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1), // Horizontal & Vertical
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1), // Diagonal
                ];

                for &(dx, dy) in &directions {
                    let new_x = (self.pos % 8) as isize + dx;
                    let new_y = (self.pos / 8) as isize + dy;

                    if new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8 {
                        let new_pos = (new_y * 8 + new_x) as usize;
                        let piece = board.get_piece_at_pos(new_pos as i32);

                        if piece.piece_type == PieceType::Empty || piece.color != self.color {
                            results.push(Move::new(self.pos, new_pos));
                        }
                    }
                }
                if checks {
                    board.update_castle_options();
                }

                match self.color {
                    Color::Black => {
                        if board.black_castle_king_side {
                            results.push(Move::new(self.pos, self.pos + 2));
                        }
                        if board.black_castle_queen_side {
                            results.push(Move::new(self.pos, self.pos - 2));
                        }
                    }
                    Color::White => {
                        if board.white_castle_king_side {
                            results.push(Move::new(self.pos, self.pos + 2));
                        }
                        if board.white_castle_queen_side {
                            results.push(Move::new(self.pos, self.pos - 2));
                        }
                    }
                }
            }
            PieceType::Empty => return vec![],
        }

        if checks {
            results = results
                .into_iter()
                .filter(|m| !board.move_causes_check(self.color.clone(), m))
                .collect();
        }

        results
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            tiles: Vec::new(),
            current_player: Color::White,
            black_castle_king_side: false,
            white_castle_king_side: false,
            black_castle_queen_side: false,
            white_castle_queen_side: false,
        }
    }

    fn castle_check(&mut self, king_pos: i32, rook_pos: i32) -> bool {
        let dir: i32;

        if king_pos > rook_pos {
            dir = -1;
        } else {
            dir = 1;
        }

        if self.is_in_board(king_pos) && self.is_in_board(rook_pos) {
            let king_pos = king_pos as usize;
            let rook_pos = rook_pos as usize;
            if self.tiles[king_pos].piece_on_tile.move_count == 0
                && self.tiles[rook_pos].piece_on_tile.move_count == 0
            {
                let mut pos: i32 = king_pos as i32 + 1 * dir;
                while pos != rook_pos as i32 {
                    if self.tiles[pos as usize].piece_on_tile.piece_type != PieceType::Empty {
                        return false;
                    }
                    let opp_col = match self.current_player {
                        Color::Black => Color::White,
                        Color::White => Color::Black,
                    };
                    let opp_move = self.get_all_legal_moves(opp_col, false);
                    for m in opp_move {
                        if m.to == pos as usize {
                            return false;
                        }
                        if m.to == king_pos {
                            return false;
                        }
                    }
                    pos += 1 * dir;
                }
                return true;
            }
        }
        false
    }

    fn update_castle_options(&mut self) {
        // white king side
        let king_pos = self.get_king_pos(Color::White).piece_on_tile.pos as i32;

        if self.castle_check(king_pos, king_pos + 3) {
            self.white_castle_king_side = true;
        } else {
            self.white_castle_king_side = false;
        }

        // white queen side
        let king_pos = self.get_king_pos(Color::White).piece_on_tile.pos as i32;

        if self.castle_check(king_pos, king_pos - 4) {
            self.white_castle_queen_side = true;
        } else {
            self.white_castle_queen_side = false;
        }

        // black queen side
        let king_pos = self.get_king_pos(Color::Black).piece_on_tile.pos as i32;

        if self.castle_check(king_pos, king_pos - 4) {
            self.black_castle_queen_side = true;
        } else {
            self.black_castle_queen_side = false;
        }

        // black king side
        let king_pos = self.get_king_pos(Color::Black).piece_on_tile.pos as i32;

        if self.castle_check(king_pos, king_pos + 3) {
            self.black_castle_king_side = true;
        } else {
            self.black_castle_king_side = false;
        }
    }

    // color ist dann von dem spieler der den zug machen möchte
    fn move_causes_check(&self, color: Color, m: &Move) -> bool {
        let mut simulation = self.clone();
        simulation.execute_move(m);
        let opponent_color = match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        let king_pos = simulation.get_king_pos(color.clone()).pos;

        let opponent_moves = simulation.get_all_legal_moves(opponent_color, false);

        for opponent_move in opponent_moves {
            if opponent_move.to == king_pos {
                return true;
            }
        }
        false
    }

    pub fn get_all_legal_moves(&mut self, player_color: Color, check: bool) -> Vec<Move> {
        let mut result = Vec::new();
        for tile in self.tiles.clone() {
            let piece = &tile.piece_on_tile;
            if piece.color == player_color {
                let moves = piece.get_legal_moves(self, false, check);
                result.extend(moves);
            }
        }
        result
    }

    fn get_king_pos(&self, color: Color) -> Tile {
        for tile in self.tiles.clone() {
            if tile.piece_on_tile.piece_type == PieceType::King && tile.piece_on_tile.color == color
            {
                return tile;
            }
        }
        panic!("king not found");
    }

    pub fn setup(&mut self, fen: &str) {
        // Split the FEN string into its components
        let parts: Vec<&str> = fen.split_whitespace().collect();

        // FEN parts
        let pieces_part = parts[0];
        let active_color_part = parts[1];
        let castling_part = parts[2];

        // Initialize board tiles with empty pieces
        self.tiles = (0..64)
            .map(|i| Tile::new(i, Piece::new(Color::White, PieceType::Empty, i)))
            .collect();

        // Map piece characters to PieceType and Color
        let piece_map = |c: char, pos: usize| -> Piece {
            let color = if c.is_uppercase() {
                Color::White
            } else {
                Color::Black
            };
            let piece_type = match c.to_ascii_lowercase() {
                'p' => PieceType::Pawn,
                'r' => PieceType::Rook,
                'n' => PieceType::Knight,
                'b' => PieceType::Bishop,
                'q' => PieceType::Queen,
                'k' => PieceType::King,
                _ => PieceType::Empty,
            };
            Piece::new(color, piece_type, pos)
        };

        // Set up pieces on the board
        let mut pos = 0;
        for c in pieces_part.chars() {
            match c {
                '/' => continue,
                '1'..='8' => pos += c.to_digit(10).unwrap() as usize,
                _ => {
                    self.tiles[pos].piece_on_tile = piece_map(c, pos);
                    pos += 1;
                }
            }
        }

        // Set the active color
        self.current_player = if active_color_part == "w" {
            Color::White
        } else {
            Color::Black
        };

        // Set castling rights
        self.white_castle_king_side = castling_part.contains('K');
        self.white_castle_queen_side = castling_part.contains('Q');
        self.black_castle_king_side = castling_part.contains('k');
        self.black_castle_queen_side = castling_part.contains('q');
    }

    fn is_in_board(&self, pos: i32) -> bool {
        if pos >= 0 {
            let pos = pos as usize;
            let piece = self.tiles.get(pos);

            let res = match piece {
                Some(_) => true,
                None => false,
            };
            return res;
        }
        false
    }

    pub fn get_piece_at_pos(&self, pos: i32) -> Piece {
        let tile = self.tiles.get(pos as usize);

        if let Some(t) = tile {
            return t.piece_on_tile.clone();
        } else {
            panic!();
        }
    }

    fn is_completly_left(&self, pos: usize) -> bool {
        return pos % 8 == 0;
    }

    fn is_completly_right(&self, pos: usize) -> bool {
        return pos % 8 == 7;
    }

    pub fn change_player(&mut self) {
        if self.current_player == Color::White {
            self.current_player = Color::Black;
        } else {
            self.current_player = Color::White;
        }
    }

    fn execute_move(&mut self, player_move: &Move) {
        if self.tiles[player_move.from].piece_on_tile.piece_type == PieceType::King {
            if player_move.from + 2 == player_move.to {
                let rook_pos = player_move.from + 3;
                self.execute_move(&Move::new(rook_pos, rook_pos - 2))
            } else if player_move.from - 2 == player_move.to {
                let rook_pos = player_move.from - 4;
                self.execute_move(&Move::new(rook_pos, rook_pos + 3))
            }
        }
        let from_pos = player_move.from;
        let to_pos = player_move.to;

        let mut piece = self.tiles[from_pos].piece_on_tile.clone();
        piece.pos = to_pos;
        piece.move_count += 1;
        self.tiles[to_pos].piece_on_tile = piece;

        self.tiles[from_pos].piece_on_tile = Piece::new(Color::White, PieceType::Empty, from_pos);
    }

    pub fn make_move(&mut self, player_move: Move) -> bool {
        let selected_piece = self.tiles[player_move.from].piece_on_tile.clone();

        let legal_moves = selected_piece.get_legal_moves(self, true, true);
        if legal_moves.contains(&player_move) {
            self.execute_move(&player_move);
            self.change_player();
            return true;
        }

        return false;
    }
}
