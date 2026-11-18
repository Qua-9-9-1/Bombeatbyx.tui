use crate::local::app::{App, AppState};
use crate::local::settings::{
    BeatSound, GaugeSkin, KeyPreset, MAX_SERVER_ADDR_CHARS, is_valid_server_addr,
};
use crossterm::event::KeyCode;

pub const SETTINGS_ROW_SOUND: usize = 9;
pub const SETTINGS_ROW_SERVER: usize = 10;
pub const SETTINGS_ROW_BACK: usize = 11;
const SETTINGS_ITEM_COUNT: usize = 12;

impl App {
    /// Switches the metronome mode and rings once so the player hears it.
    fn cycle_beat_sound(&mut self) {
        self.profile.beat_sound = self.profile.beat_sound.next();
        let _ = self.profile.save();
        if self.profile.beat_sound != BeatSound::Off {
            let _ = crate::tui::ring_bell();
        }
    }

    fn handle_server_addr_typing(&mut self, code: KeyCode) {
        let Some(draft) = self.server_addr_draft.as_mut() else {
            return;
        };
        match code {
            KeyCode::Enter if is_valid_server_addr(draft.trim()) => {
                self.profile.server_addr = draft.trim().to_string();
                self.server_addr_draft = None;
                let _ = self.profile.save();
            }
            KeyCode::Backspace => {
                draft.pop();
            }
            KeyCode::Char(c) if !c.is_whitespace() && draft.chars().count() < MAX_SERVER_ADDR_CHARS => {
                draft.push(c);
            }
            _ => {}
        }
    }

    pub(crate) fn handle_settings_menu_input(&mut self, code: KeyCode) {
        let num_items = SETTINGS_ITEM_COUNT;

        if self.server_addr_draft.is_some() {
            self.handle_server_addr_typing(code);
            return;
        }

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
                    SETTINGS_ROW_SOUND => self.cycle_beat_sound(),
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
                SETTINGS_ROW_SOUND => self.cycle_beat_sound(),
                SETTINGS_ROW_SERVER => {
                    self.server_addr_draft = Some(self.profile.server_addr.clone());
                }
                SETTINGS_ROW_BACK => {
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
