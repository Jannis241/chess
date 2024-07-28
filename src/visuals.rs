use crate::*;

fn draw_board(board: &Board, cursor_pos: &usize) {
    // current player
    // selector 
    // invalid move error
    // winner
    // legal moves
    // capture extra farbe
    // schach anzeige (könig)
    // player turn
    let mut board_representation = vec![vec![' '; 8]; 8];

    // Populate the board representation with pieces
    for tile in &board.tiles {
        let piece_char = match tile.piece_on_tile.piece_type {
            PieceType::Empty => '.',
            PieceType::Pawn => if tile.piece_on_tile.color == Color::White { 'P' } else { 'p' },
            PieceType::King => if tile.piece_on_tile.color == Color::White { 'K' } else { 'k' },
            PieceType::Bishop => if tile.piece_on_tile.color == Color::White { 'B' } else { 'b' },
            PieceType::Knight => if tile.piece_on_tile.color == Color::White { 'N' } else { 'n' },
            PieceType::Rook => if tile.piece_on_tile.color == Color::White { 'R' } else { 'r' },
            PieceType::Queen => if tile.piece_on_tile.color == Color::White { 'Q' } else { 'q' },
        };
        board_representation[tile.pos / 8][tile.pos % 8] = piece_char;
    }

    // Print the board row by row
    let mut i = 0;
    for row in board_representation.iter() {
        for &cell in row {
            if &i == cursor_pos {
                print!("#");
            } else {
                print!("{} ", cell);
            }
            i += 1;
        }
        println!();
    }
}

fn clear_terminal() {

}
fn print_navigation() {

}

pub fn draw(board: &Board, cursor_pos: &usize) {
    clear_terminal();
    print_navigation();
    draw_board(board, cursor_pos);
}
