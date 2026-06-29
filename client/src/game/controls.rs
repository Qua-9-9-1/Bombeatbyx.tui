use crossterm::event::{KeyCode, KeyEvent};

pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Quit,
    None,
}

pub fn handle_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Quit,
        KeyCode::Up | KeyCode::Char('z') => Action::MoveUp,
        KeyCode::Down | KeyCode::Char('s') => Action::MoveDown,
        KeyCode::Left | KeyCode::Char('q') => Action::MoveLeft,
        KeyCode::Right | KeyCode::Char('d') => Action::MoveRight,
        _ => Action::None,
    }
}