use crossterm::event::KeyCode;

pub struct MainMenuScreen {
    pub cursor: usize,
}

pub enum MainMenuAction {
    None,
    HostGame,
    JoinGame,
    Settings,
    Exit,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    pub fn handle_input(&mut self, code: KeyCode) -> MainMenuAction {
        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.cursor = self.cursor.saturating_sub(1);
                MainMenuAction::None
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.cursor = (self.cursor + 1).min(3);
                MainMenuAction::None
            }
            KeyCode::Enter => match self.cursor {
                0 => MainMenuAction::HostGame,
                1 => MainMenuAction::JoinGame,
                2 => MainMenuAction::Settings,
                3 => MainMenuAction::Exit,
                _ => MainMenuAction::None,
            },
            _ => MainMenuAction::None,
        }
    }
}
