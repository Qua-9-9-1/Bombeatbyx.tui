use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Game,
    Menu,
}

pub struct ControlsManager {
    pub mode: ControlMode,
    pub menu_cursor: usize,
    pub max_menu_items: usize,
}

impl ControlsManager {
    pub fn new() -> Self {
        Self {
            mode: ControlMode::Game,
            menu_cursor: 0,
            max_menu_items: 3,
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        if self.mode == ControlMode::Menu {
            self.handle_menu_input(key);
        }
    }

    fn handle_menu_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('m') => {
                self.mode = ControlMode::Game;
            }
            KeyCode::Up | KeyCode::Char('z') => {
                if self.menu_cursor > 0 {
                    self.menu_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if self.menu_cursor < self.max_menu_items - 1 {
                    self.menu_cursor += 1;
                }
            }
            KeyCode::Enter => {
                if self.menu_cursor == 0 {
                    self.mode = ControlMode::Game;
                }
            }
            _ => {}
        }
    }
}
