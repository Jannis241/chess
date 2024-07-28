use crate::*;

#[derive(Clone, PartialEq)]
pub struct Move {
    pub from: usize,
    pub to: usize,
}
#[derive(PartialEq, Clone, Debug)]
pub enum Color {
    Black,
    White,
}
#[derive(Clone)]
pub struct Tile {
    pub pos: usize,
    pub piece_on_tile: Piece
}
#[derive(Clone)]
pub struct Board {
    pub tiles: Vec<Tile>,
    pub current_player: Color,
}
#[derive(Clone, Debug)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    pub pos: usize,
}
#[derive(Clone, Debug)]
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
        }
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
        position_map.insert(0, Piece::new(Color::White, PieceType::Rook, 0));
        position_map.insert(1, Piece::new(Color::White, PieceType::Knight, 1));
        position_map.insert(2, Piece::new(Color::White, PieceType::Bishop, 2));
        position_map.insert(3, Piece::new(Color::White, PieceType::Queen, 3));
        position_map.insert(4, Piece::new(Color::White, PieceType::King, 4));
        position_map.insert(5, Piece::new(Color::White, PieceType::Bishop, 5));
        position_map.insert(6, Piece::new(Color::White, PieceType::Knight, 6));
        position_map.insert(7, Piece::new(Color::White, PieceType::Rook, 7));
    
        for i in 8..16 {
            position_map.insert(i, Piece::new(Color::White, PieceType::Pawn, i));
        }
    
        // Black pieces
        position_map.insert(56, Piece::new(Color::Black, PieceType::Rook, 56));
        position_map.insert(57, Piece::new(Color::Black, PieceType::Knight, 57));
        position_map.insert(58, Piece::new(Color::Black, PieceType::Bishop, 58));
        position_map.insert(59, Piece::new(Color::Black, PieceType::Queen, 59));
        position_map.insert(60, Piece::new(Color::Black, PieceType::King, 60));
        position_map.insert(61, Piece::new(Color::Black, PieceType::Bishop, 61));
        position_map.insert(62, Piece::new(Color::Black, PieceType::Knight, 62));
        position_map.insert(63, Piece::new(Color::Black, PieceType::Rook, 63));
    
        for i in 48..56 {
            position_map.insert(i, Piece::new(Color::Black, PieceType::Pawn, i));
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


    fn get_legal_moves_for_piece(&mut self, piece: Piece) -> Vec<Move> {
        match piece.piece_type {
            _ => {

            }
        }
        vec![]
    }
    
    fn is_move_legal(&mut self, player_move: &Move) -> bool{
        let selected_piece = self.tiles[player_move.from].piece_on_tile.clone();
        return self.get_legal_moves_for_piece(selected_piece).contains(player_move);
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

        // Place the piece on the target tile
        self.tiles[to_pos].piece_on_tile = piece;

        // Set the source tile to be empty
        self.tiles[from_pos].piece_on_tile = Piece::new(Color::White, PieceType::Empty, from_pos);
    }
    
    
    pub fn make_move(&mut self, player_move: Move) -> bool { 
        if self.is_move_legal(&player_move){
            self.execute_move(&player_move);
            self.change_player();
            return true;
        }

        false // return if the move was valid or not
    }


}