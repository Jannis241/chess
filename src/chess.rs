use std::{result, thread::panicking, vec};

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
                Some(p) => true,
                None => false,
            };
            return res;
        }
        false
    }


    fn get_valid_king_moves(&mut self) -> Vec<Move> {
        return vec![]
    }


    pub fn get_piece_at_pos(&self, pos: i32) -> Piece {
        let tile = self.tiles.get(pos as usize);

        if let Some(t) = tile {
            return t.piece_on_tile.clone();
        }
        else {
            println!("Error at get piece at pos, tile is out of bounds. Position: {}", pos);
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


    pub fn get_legal_moves_for_piece(&mut self, piece: &Piece, check_if_own_piece: bool) -> Vec<Move> {
        if check_if_own_piece && piece.color != self.current_player {
            return vec![]
        }
        
        let mut results: Vec<Move> = Vec::new();


        // ! AUFPASSEN WENN MAN GANZ LINKS ODER RECHTS IST DASS MAN NICHT OVERFLOWT, AM BESTEN
        // MIT LINKEM UND RECHTEN RAND CHECKEN

        match piece.piece_type {
            PieceType::Rook => {

            }
            PieceType::Knight => {

            }
            PieceType::Bishop => {

            }
            PieceType::Queen => {

            }
            PieceType::King => {
                results = self.get_valid_king_moves();
            }
            PieceType::Pawn => {
                let mut direction =  1; // default ist nach oben
                
                if piece.color == Color::Black {
                    direction = -1;
                }

                // check if 1 up is empty
                if self.is_in_board(piece.pos as i32 + UP * direction) {
                    if self.get_piece_at_pos(piece.pos as i32 + UP * direction).piece_type == PieceType::Empty{
                        let m = Move::new(piece.pos, self.add_offset_to_position(piece.pos, UP * direction));
                        results.push(m);
                    }
                }

                // check for 2 up move
                if piece.move_count == 0 {
                    if self.get_piece_at_pos(piece.pos as i32 + UP * direction + UP * direction).piece_type == PieceType::Empty{
                        let m = Move::new(piece.pos, self.add_offset_to_position(piece.pos, UP * direction + UP * direction));
                        results.push(m);
                    } 
                }


                // captures 

                if self.is_in_board(piece.pos as i32 + UP * direction + LEFT) && !self.is_completly_left(piece.pos){
                    let p = self.get_piece_at_pos(piece.pos as i32 + UP * direction + LEFT);
                    if p.color != piece.color && p.piece_type != PieceType::Empty{                        
                        let m = Move::new(piece.pos, self.add_offset_to_position(piece.pos, UP * direction + LEFT));
                        results.push(m);
                    } 
                }
                if self.is_in_board(piece.pos as i32 + UP * direction + RIGHT) && !self.is_completly_right(piece.pos){
                    let p = self.get_piece_at_pos(piece.pos as i32 + UP * direction + RIGHT);
                    if p.color != piece.color && p.piece_type != PieceType::Empty{
                        let m = Move::new(piece.pos, self.add_offset_to_position(piece.pos, UP * direction + RIGHT));
                        results.push(m);
                    } 
                }
            }
            PieceType::Empty =>{
                return vec![];
            }
        }

        return results;
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

        let legal_moves = self.get_legal_moves_for_piece(&selected_piece, true);
        println!("your pos: {}", player_move.from);
        println!("legal moves for your piece: {:?}", legal_moves);
        if legal_moves.contains(&player_move) {
            self.execute_move(&player_move);
            self.change_player();
            return true;
        }

        return false
    }


}