use crossterm::event::KeyCode;

pub struct MainMenuScreen {
    pub cursor: usize,
}

pub enum MainMenuAction {
    None,
    HostGame,
    JoinGame,
    LocalDevGame,
    Settings,
    Exit,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    pub fn handle_input(&mut self, code: KeyCode) -> MainMenuAction {
        let max_cursor = if cfg!(debug_assertions) { 4 } else { 3 };
        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.cursor = self.cursor.saturating_sub(1);
                MainMenuAction::None
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.cursor = (self.cursor + 1).min(max_cursor);
                MainMenuAction::None
            }
            KeyCode::Enter => {
                if cfg!(debug_assertions) {
                    match self.cursor {
                        0 => MainMenuAction::HostGame,
                        1 => MainMenuAction::JoinGame,
                        2 => MainMenuAction::LocalDevGame,
                        3 => MainMenuAction::Settings,
                        4 => MainMenuAction::Exit,
                        _ => MainMenuAction::None,
                    }
                } else {
                    match self.cursor {
                        0 => MainMenuAction::HostGame,
                        1 => MainMenuAction::JoinGame,
                        2 => MainMenuAction::Settings,
                        3 => MainMenuAction::Exit,
                        _ => MainMenuAction::None,
                    }
                }
            }
            _ => MainMenuAction::None,
        }
    }
}
