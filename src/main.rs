use crossterm::event::*;
use crossterm::terminal::*;
use crossterm::event;

use std::collections::HashMap;
use std::process::exit;

pub use chess::*;
pub use input::*;
pub use visuals::*;

pub mod chess;
pub mod visuals;
pub mod input;

fn main() {
    let mut board = Board::new();

    let mut cursor_pos = 28;
    

    let mut from = 0;
    let mut selected = false;
    loop {
        let moves = board.get_all_legal_moves(board.current_player.clone(), true);
        if moves.len() == 0 {
            board.change_player();
            visuals::draw_winner(&board, board.current_player.clone());
            exit(0)
        }
        // draw legal moves
        if selected {
            let selected_piece = &board.get_piece_at_pos(from as i32);
            let legal_moves: Vec<usize> = selected_piece.get_legal_moves(&board, true, true ).iter().map(|m| m.to).collect();        
            
            // if its their own piece draw legal moves
            if selected_piece.color == board.current_player{
                visuals::draw(&board, &cursor_pos, &board.current_player, &legal_moves, Some(from));
            } ;
        } else {
            // just draw the normal board (for example if the user has moved without selecting anything)
            visuals::draw(&board, &cursor_pos, &board.current_player, &vec![], None);
        }

        let inp = input::get();

        // navigation
        match inp {
            KeyCode::Down => {
                if cursor_pos + 8 <= 63 {
                    cursor_pos += 8;
                }
            }
            KeyCode::Up => {
                if cursor_pos >= 8 {
                    cursor_pos -= 8;
                }
            }
            KeyCode::Left => {
                if cursor_pos % 8 != 0 {
                    cursor_pos -= 1;
                }
            }

            KeyCode::Right => {
                if cursor_pos % 8 != 7 {
                    cursor_pos += 1;
                }
            }
            KeyCode::Enter => {
                if selected {
                    // check if the start position is not equal to the end position
                    if cursor_pos != from {

                        // try to make the move
                        let valid = board.make_move(Move::new(from, cursor_pos));
                        
                        // if the move is valid unselect the piece and reset the cursor
                        if valid {
                            selected = false;
                            cursor_pos = 28; // reset cursor to the middle
                        }

                        // move is not valid
                        else {
                            
                            let selected_piece = &board.get_piece_at_pos(cursor_pos as i32);

                            // if the user just switched from one piece to another update the start position
                            // from the move
                            if selected_piece.color == board.current_player && selected_piece.piece_type != PieceType::Empty{
                                from = cursor_pos;
                                selected = true;
                            } 

                            // the user clicked on an enemy or empty piece, so just unselect
                            else {
                                selected = false;
                            }
                        }
                    }
                    else {
                        selected = false;
                    }
                }
                else {
                    // user wants to select a piece
                    let selected_piece = &board.get_piece_at_pos(cursor_pos as i32);
                    
                    // check if their own piece, if yes then select it, else do nothing
                    if selected_piece.piece_type != PieceType::Empty && selected_piece.color == board.current_player{
                        from = cursor_pos;
                        selected = true;
                    } 
                }
            }
            // unselecting 
            KeyCode::Esc => {
                selected = false;
            }

            // cant be here because the input function filters everything exepect the valid inputs
            _ => panic!("shouldnt be here, something is wrong the the get input function")
        }
    }
}
