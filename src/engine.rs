use crate::*;
pub struct Engine {
    board: Board,
    moves_ahead: i32,
}



impl Engine {
    fn new (board: Board, moves_ahead: i32) -> Self {
        Engine {
            board,
            moves_ahead,
        }
    }

    fn get_best_move(&self, anzahl_an_moves: i32) -> Vec<Move> {
        vec![]
    }
}