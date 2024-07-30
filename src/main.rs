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
        //intro();
        if selected {
            let selected_piece = &board.get_piece_at_pos(from as i32);
            let legal_moves: Vec<usize> = board.get_legal_moves_for_piece(selected_piece, true).iter().map(|m| m.to).collect();
            let mut selected_piece_index = None; 

            if selected_piece.piece_type != PieceType::Empty && selected_piece.color == board.current_player{
                selected_piece_index = Some(from)
            } ;

            visuals::draw(&board, &cursor_pos, &board.current_player, &legal_moves, selected_piece_index);
        }
        else {
            visuals::draw(&board, &cursor_pos, &board.current_player, &vec![], None);
        }
        let inp = input::get();


        // Todo: implement de-select feature

        // navigation
        match inp {
            KeyCode::Down => {
                if cursor_pos + 8 >= 64 {
                    cursor_pos -= 56;
                }
                else {
                    cursor_pos += 8;
                }
            }
            KeyCode::Up => {
                if cursor_pos < 8 {
                    cursor_pos += 56;
                }
                else {

                    cursor_pos -= 8;
                }
            }
            KeyCode::Left => {
                if cursor_pos % 8 == 0 {
                    cursor_pos += 7;
                }
                else {
                    cursor_pos -= 1;
                }
            }

            KeyCode::Right => {
                if cursor_pos % 8 == 7 {
                    cursor_pos -= 7;
                }
                else {
                    cursor_pos += 1;
                }
            }
            KeyCode::Enter => {
                if selected {
                    if cursor_pos != from {

                        let valid = board.make_move(Move::new(from, cursor_pos));

                        if valid {
                            selected = false;
                            cursor_pos = 28; // reset cursor to the middle
                        }
                        else {
                            // Todo: make this pretty with ascii
                            let selected_piece = &board.get_piece_at_pos(cursor_pos as i32);

                            if selected_piece.color == board.current_player{
                                from = cursor_pos;
                                selected = true;
                            } 
                            else {
                                selected = false;
                            }
                        }
                    }
                }
                else {
                    let selected_piece = &board.get_piece_at_pos(cursor_pos as i32);

                    if selected_piece.piece_type != PieceType::Empty && selected_piece.color == board.current_player{
                        from = cursor_pos;
                        selected = true;
                    } 
                    else {
                        selected = false;
                    }
                    
                }
            }
            KeyCode::Esc => {
                selected = false;
                from = 0;
            }
            _ => panic!("shouldnt be here, something is wrong the the get input function")
        }

        


    }
}
