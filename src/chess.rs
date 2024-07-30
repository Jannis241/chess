use crate::*;

const UP: i32 = -8;
const DOWN: i32 = 8;
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
    pub piece_on_tile: Piece
}
#[derive(Clone, Debug)]
pub struct Board {
    pub tiles: Vec<Tile>,
    pub current_player: Color,
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
        Move {
            from,
            to,
        }
    }
}

impl Tile {
    pub fn new(pos: usize, piece_on_tile: Piece) -> Self {
        Tile {
            pos,
            piece_on_tile,
        }
    }    
}

impl Piece {
    pub fn new(color: Color, piece_type: PieceType, pos: usize) -> Self {
        Piece {
            color,
            piece_type,
            pos,
            move_count: 0
        }
    }
    pub fn get_legal_moves(&self, board: &Board, check_if_own_piece: bool) -> Vec<Move> {
        if check_if_own_piece && self.color != board.current_player {
            return vec![]
        }
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
                                if board.get_piece_at_pos(two_steps_pos).piece_type == PieceType::Empty {
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
                    (2, 1), (2, -1), (-2, 1), (-2, -1),
                    (1, 2), (1, -2), (-1, 2), (-1, -2)
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
                let directions = [
                    (1, 0), (-1, 0), (0, 1), (0, -1)
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
            PieceType::Bishop => {
                let directions = [
                    (1, 1), (1, -1), (-1, 1), (-1, -1)
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
            }PieceType::Queen => {
                let directions = [
                    (1, 0), (-1, 0), (0, 1), (0, -1), // Rook directions
                    (1, 1), (1, -1), (-1, 1), (-1, -1) // Bishop directions
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
            }PieceType::King => {
                let directions = [
                    (1, 0), (-1, 0), (0, 1), (0, -1), // Horizontal & Vertical
                    (1, 1), (1, -1), (-1, 1), (-1, -1) // Diagonal
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

                


            }
            PieceType::Empty => {
                return vec![]
            } 
        }

        // die base moves sind jetzt da
        // Todo: pins, check (schach) en passent, promotions checken, castle

        results
    }
}




impl Board {
    pub fn new() -> Self {
        let mut board = Board {
            tiles: Vec::new(),
            current_player: Color::White,
        };
        board.setup();

        return board;
    }
    fn create_default_chess_position(&self) -> HashMap<usize, Piece> {

        let mut position_map: HashMap<usize, Piece> = HashMap::new();
    
        // White pieces
        position_map.insert(0, Piece::new(Color::Black, PieceType::Rook, 0));
        position_map.insert(1, Piece::new(Color::Black, PieceType::Knight, 1));
        position_map.insert(2, Piece::new(Color::Black, PieceType::Bishop, 2));
        position_map.insert(3, Piece::new(Color::Black, PieceType::Queen, 3));
        position_map.insert(4, Piece::new(Color::Black, PieceType::King, 4));
        position_map.insert(5, Piece::new(Color::Black, PieceType::Bishop, 5));
        position_map.insert(6, Piece::new(Color::Black, PieceType::Knight, 6));
        position_map.insert(7, Piece::new(Color::Black, PieceType::Rook, 7));
    
        for i in 8..16 {
            position_map.insert(i, Piece::new(Color::Black, PieceType::Pawn, i));
        }
    
        // Black pieces
        position_map.insert(56, Piece::new(Color::White, PieceType::Rook, 56));
        position_map.insert(57, Piece::new(Color::White, PieceType::Knight, 57));
        position_map.insert(58, Piece::new(Color::White, PieceType::Bishop, 58));
        position_map.insert(59, Piece::new(Color::White, PieceType::Queen, 59));
        position_map.insert(60, Piece::new(Color::White, PieceType::King, 60));
        position_map.insert(61, Piece::new(Color::White, PieceType::Bishop, 61));
        position_map.insert(62, Piece::new(Color::White, PieceType::Knight, 62));
        position_map.insert(63, Piece::new(Color::White, PieceType::Rook, 63));
    
        for i in 48..56 {
            position_map.insert(i, Piece::new(Color::White, PieceType::Pawn, i));
        }
    
        // Empty pieces for the rest of the board
        for i in 16..48 {
            position_map.insert(i, Piece::new(Color::White, PieceType::Empty, i)); // or any other default for empty tiles
        }
    
        position_map
    }
    
    fn setup(&mut self) {
        let initial_pos = self.create_default_chess_position();

        for i in 0..64 {
            let piece = &initial_pos[&i];

            let tile = Tile::new(i, piece.clone());
            self.tiles.push(tile);
        }

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
        }
        else {
            panic!();
        }
    }

    fn add_offset_to_position(&self, position: usize, offset: i32) -> usize {
        return (position as i32 + offset) as usize
    }

    fn is_completly_left(&self, pos: usize) -> bool {
        return pos % 8 == 0;
    }

    fn is_completly_right(&self, pos: usize) -> bool {
        return pos % 8 == 7;
    }

    pub fn change_player(&mut self){
        if self.current_player == Color::White{
            self.current_player = Color::Black;
        } else {
            self.current_player = Color::White;
        }
    }
    fn execute_move(&mut self, player_move: &Move) {
        // Get the source and target positions
        let from_pos = player_move.from;
        let to_pos = player_move.to;

        // Clone the piece at the source position
        let mut piece = self.tiles[from_pos].piece_on_tile.clone();

        // Update the piece's position
        piece.pos = to_pos;
        piece.move_count += 1;

        // Place the piece on the target tile
        self.tiles[to_pos].piece_on_tile = piece;

        // Set the source tile to be empty
        self.tiles[from_pos].piece_on_tile = Piece::new(Color::White, PieceType::Empty, from_pos);
    }
    
    
    pub fn make_move(&mut self, player_move: Move) -> bool { 
        let selected_piece = self.tiles[player_move.from].piece_on_tile.clone();

        let legal_moves = selected_piece.get_legal_moves(self, true);
        if legal_moves.contains(&player_move) {
            self.execute_move(&player_move);
            self.change_player();
            return true;
        }

        return false
    }


}