use crate::*;
use std::clone::Clone;

fn draw_board(board: &Board, cursor_pos: &usize, legal_moves: &Vec<usize>, selected_piece: Option<usize>) {
    let mut board_representation = vec![vec![' '; 8]; 8];

    // Populate the board representation with pieces
    for tile in &board.tiles {
        let piece_char = match (tile.piece_on_tile.piece_type.clone(), tile.piece_on_tile.color.clone()) {
            (PieceType::Empty, _) => ' ',
            (PieceType::Pawn, Color::White) => '♙',
            (PieceType::Pawn, Color::Black) => '♟',
            (PieceType::King, Color::White) => '♔',
            (PieceType::King, Color::Black) => '♚',
            (PieceType::Bishop, Color::White) => '♗',
            (PieceType::Bishop, Color::Black) => '♝',
            (PieceType::Knight, Color::White) => '♘',
            (PieceType::Knight, Color::Black) => '♞',
            (PieceType::Rook, Color::White) => '♖',
            (PieceType::Rook, Color::Black) => '♜',
            (PieceType::Queen, Color::White) => '♕',
            (PieceType::Queen, Color::Black) => '♛',
        };
        board_representation[tile.pos / 8][tile.pos % 8] = piece_char;
    }

    // ANSI escape codes for coloring
    let reset_color = "\x1B[0m";
    let white_piece_color = "\x1B[97m"; // bright white
    let black_piece_color = "\x1B[90m"; // bright black (dark gray)
    let cursor_color = "\x1B[42m"; // green background
    let legal_move_color = "\x1B[43m"; // yellow background for legal moves
    let selected_piece_color = "\x1B[41m"; // red background for selected piece

    // Print the board row by row
    let mut i = 0;
    for row in board_representation.iter() {
        for &cell in row.iter() {
            if Some(i) == selected_piece {
                print!("{}{}{} ", selected_piece_color, cell, reset_color);
            } else if &i == cursor_pos {
                print!("{}{}{} ", cursor_color, cell, reset_color);
            } else if legal_moves.contains(&i) {
                print!("{}{}{} ", legal_move_color, cell, reset_color);
            } else {
                match cell {
                    '♙' | '♔' | '♗' | '♘' | '♖' | '♕' => print!("{}{}{} ", white_piece_color, cell, reset_color),
                    '♟' | '♚' | '♝' | '♞' | '♜' | '♛' => print!("{}{}{} ", black_piece_color, cell, reset_color),
                    _ => print!("{}{}{} ", reset_color, cell, reset_color),
                }
            }
            i += 1;
        }
        println!();
    }
}

fn clear_terminal() {
    // ANSI escape code to clear the terminal
     print!("\x1B[2J\x1B[H");
}

fn print_navigation() {
    // Placeholder function to print navigation instructions
    println!("Use arrow keys to move the cursor, press Enter to select a piece.");
}

fn print_current_player(current_player: &Color) {
    // Print the current player
    let player_color = match current_player {
        Color::White => "\x1B[97mWhite\x1B[0m", // bright white
        Color::Black => "\x1B[90mBlack\x1B[0m", // bright black (dark gray)
    };
    println!();
    println!("Current player: {}", player_color);
    println!();
}

pub fn draw(board: &Board, cursor_pos: &usize, current_player: &Color, legal_moves: &Vec<usize>, selected_piece: Option<usize>) {
    clear_terminal();
    print_navigation();
    print_current_player(current_player);
    draw_board(board, cursor_pos, legal_moves, selected_piece);
}
