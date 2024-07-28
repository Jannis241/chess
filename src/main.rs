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
    
    let mut selected = false;

    let mut from = 0;
    
    loop {
        //intro();
        visuals::draw(&board, &cursor_pos);
        println!("it is {:?}s turn", board.current_player);
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
                            selected = false;
                            println!("invalid move")
                        }
                    }
                }
                else {
                    from = cursor_pos;
                    selected = true;
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
