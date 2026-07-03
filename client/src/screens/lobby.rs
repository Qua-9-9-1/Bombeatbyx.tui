use crossterm::event::KeyCode;
use common::game::models::RoomSettings;

pub struct LobbyScreen {
    pub cursor: usize,
}

impl LobbyScreen {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    pub fn handle_input(
        &mut self,
        room_settings: &mut RoomSettings,
        profile_skin: &mut String,
        is_host: bool,
        code: KeyCode,
    ) -> bool {
        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') => {
                let max_row = if is_host { 8 } else { 7 };
                self.cursor = (self.cursor + 1).min(max_row);
            }
            KeyCode::Left | KeyCode::Char('q') => {
                if self.cursor == 7 {
                    self.modify_local_skin(profile_skin, false);
                } else if is_host && self.cursor < 7 {
                    self.modify_room_setting(room_settings, false);
                }
            }
            KeyCode::Right | KeyCode::Char('d') => {
                if self.cursor == 7 {
                    self.modify_local_skin(profile_skin, true);
                } else if is_host && self.cursor < 7 {
                    self.modify_room_setting(room_settings, true);
                }
            }
            KeyCode::Enter if self.cursor == 8 && is_host => {
                return true;
            }
            _ => {}
        }
        false
    }

    fn modify_room_setting(&mut self, room_settings: &mut RoomSettings, increase: bool) {
        let sign = if increase { 1 } else { -1 };
        match self.cursor {
            0 => room_settings.width = ((room_settings.width as i32) + sign * 2).clamp(7, 29) as usize,
            1 => room_settings.height = ((room_settings.height as i32) + sign * 2).clamp(7, 29) as usize,
            2 => room_settings.bpm = (room_settings.bpm + sign as f64 * 5.0).clamp(40.0, 220.0),
            3 => room_settings.sudden_death = !room_settings.sudden_death,
            4 => room_settings.bonus_every = ((room_settings.bonus_every as i32) + sign).clamp(1, 60) as u32,
            5 => room_settings.lives = ((room_settings.lives as i32) + sign).clamp(1, 9) as u8,
            6 => {
                room_settings.mode = match room_settings.mode {
                    common::game::models::GameMode::Deathmatch => common::game::models::GameMode::Score,
                    common::game::models::GameMode::Score => common::game::models::GameMode::Deathmatch,
                };
            }
            _ => {}
        }
    }

    fn modify_local_skin(&mut self, profile_skin: &mut String, increase: bool) {
        let skins = vec!["🤖", "🐱", "🐸", "🦊", "🐧"];
        let current_idx = skins.iter().position(|&s| s == profile_skin).unwrap_or(0);
        let new_idx = if increase { (current_idx + 1) % skins.len() } else { (current_idx + skins.len() - 1) % skins.len() };
        *profile_skin = skins[new_idx].to_string();
    }
}