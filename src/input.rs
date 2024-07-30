use crate::*;

pub fn get() -> KeyCode {
    // Enable raw mode to get input immediately
    enable_raw_mode().unwrap();

    loop {
        let mut input: Option<KeyCode> = None;
        if event::poll(std::time::Duration::from_millis(500)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                input = match key_event.code {
                    KeyCode::Enter => Some(key_event.code),
                    KeyCode::Up => Some(key_event.code),
                    KeyCode::Down => Some(key_event.code),
                    KeyCode::Left => Some(key_event.code),
                    KeyCode::Right => Some(key_event.code),
                    KeyCode::Esc => Some(key_event.code),

                    KeyCode::Char('h') => Some(KeyCode::Left),
                    KeyCode::Char('j') => Some(KeyCode::Down),
                    KeyCode::Char('k') => Some(KeyCode::Up),
                    KeyCode::Char('l') => Some(KeyCode::Right),
                    KeyCode::Char(' ') => Some(KeyCode::Enter),
                    KeyCode::Char('q')=> exit(0),
                    
                    _ => None 
                };
            }
        }
        if let Some(inp) = input {
            // Cleanup terminal state
            disable_raw_mode().unwrap();
            return inp
        }
    }




}


