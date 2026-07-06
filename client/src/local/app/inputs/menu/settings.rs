use crate::local::app::{App, AppState};
use crate::local::settings::{GaugeSkin, KeyPreset};
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_settings_menu_input(&mut self, code: KeyCode) {
        let num_items = 10_usize;

        if self.capturing_key {
            if code == KeyCode::Esc {
                self.capturing_key = false;
                return;
            }
            let ch = match code {
                KeyCode::Char(c) => Some(c),
                _ => None,
            };
            if let Some(c) = ch {
                match self.settings_cursor {
                    1 => self.profile.key_up = c,
                    2 => self.profile.key_down = c,
                    3 => self.profile.key_left = c,
                    4 => self.profile.key_right = c,
                    5 => self.profile.key_bomb = c,
                    6 => self.profile.key_spell = c,
                    _ => {}
                }
                let _ = self.profile.save();
                self.capturing_key = false;
            }
            return;
        }

        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.settings_cursor = self.settings_cursor.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.settings_cursor = (self.settings_cursor + 1).min(num_items - 1)
            }
            KeyCode::Left | KeyCode::Char('q') | KeyCode::Right | KeyCode::Char('d') => {
                match self.settings_cursor {
                    0 => {
                        let next = match self.profile.key_preset {
                            KeyPreset::ZQSD => KeyPreset::WASD,
                            KeyPreset::WASD => KeyPreset::ZQSD,
                        };
                        self.profile.apply_preset(next);
                        let _ = self.profile.save();
                    }
                    7 => {
                        self.profile.gauge_skin = match self.profile.gauge_skin {
                            GaugeSkin::NecroDancer => GaugeSkin::Simple,
                            GaugeSkin::Undertale => GaugeSkin::NecroDancer,
                            GaugeSkin::Simple => GaugeSkin::Undertale,
                        };
                        let _ = self.profile.save();
                    }
                    8 => {
                        self.profile.ascii_mode = !self.profile.ascii_mode;
                        let _ = self.profile.save();
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => match self.settings_cursor {
                1 | 2 | 3 | 4 | 5 | 6 => {
                    self.capturing_key = true;
                }
                8 => {
                    self.profile.ascii_mode = !self.profile.ascii_mode;
                    let _ = self.profile.save();
                }
                9 => {
                    self.capturing_key = false;
                    if self.paused_from.is_some() {
                        self.state = AppState::PauseMenu;
                    } else {
                        self.state = AppState::MainMenu;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
