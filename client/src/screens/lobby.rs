use common::game::models::{GameMode, MAX_BOTS, RoomSettings};
use crossterm::event::KeyCode;

/// Lobby rows, top to bottom. Rows before `ROW_NAME` are host-only room settings;
/// the bot rows only exist in solo mode.
pub const ROW_TIME_LIMIT: usize = 8;
pub const ROW_MAP_LAYOUT: usize = 9;
pub const ROW_BRICK_HINTS: usize = 10;
pub const ROW_BOT_COUNT: usize = 11;
pub const ROW_BOT_LEVEL: usize = 12;
pub const ROW_NAME: usize = 13;
pub const ROW_SKIN: usize = 14;
pub const ROW_READY: usize = 15;

pub fn is_bot_row(row: usize) -> bool {
    row == ROW_BOT_COUNT || row == ROW_BOT_LEVEL
}

pub struct LobbyScreen {
    pub cursor: usize,
    pub selected_player_id: Option<u32>,
}

impl LobbyScreen {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            selected_player_id: None,
        }
    }

    pub fn handle_input(
        &mut self,
        room_settings: &mut RoomSettings,
        profile_skin: &mut String,
        is_host: bool,
        has_bots: bool,
        code: KeyCode,
    ) -> bool {
        match code {
            KeyCode::Up | KeyCode::Char('z') => self.move_cursor(-1, has_bots),
            KeyCode::Down | KeyCode::Char('s') => self.move_cursor(1, has_bots),
            KeyCode::Left | KeyCode::Char('q') => {
                self.adjust(room_settings, profile_skin, is_host, false)
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.adjust(room_settings, profile_skin, is_host, true)
            }
            KeyCode::Enter if self.cursor == ROW_READY => return true,
            _ => {}
        }
        false
    }

    fn move_cursor(&mut self, step: i32, has_bots: bool) {
        let mut row = self.cursor as i32;
        loop {
            row = (row + step).clamp(0, ROW_READY as i32);
            if has_bots || !is_bot_row(row as usize) {
                break;
            }
        }
        self.cursor = row as usize;
    }

    fn adjust(
        &mut self,
        room_settings: &mut RoomSettings,
        profile_skin: &mut String,
        is_host: bool,
        increase: bool,
    ) {
        if self.cursor == ROW_SKIN {
            self.modify_local_skin(profile_skin, increase);
        } else if is_host && self.cursor < ROW_NAME {
            self.modify_room_setting(room_settings, increase);
        }
    }

    fn modify_room_setting(&mut self, room_settings: &mut RoomSettings, increase: bool) {
        let sign = if increase { 1 } else { -1 };
        match self.cursor {
            0 => {
                room_settings.width =
                    ((room_settings.width as i32) + sign * 2).clamp(7, 29) as usize
            }
            1 => {
                room_settings.height =
                    ((room_settings.height as i32) + sign * 2).clamp(7, 29) as usize
            }
            2 => room_settings.bpm = (room_settings.bpm + sign as f64 * 5.0).clamp(40.0, 220.0),
            3 => room_settings.sudden_death = !room_settings.sudden_death,
            4 => {
                room_settings.bonus_every =
                    ((room_settings.bonus_every as i32) + sign).clamp(1, 60) as u32
            }
            5 => room_settings.lives = ((room_settings.lives as i32) + sign).clamp(1, 10) as u8,
            6 => {
                room_settings.mode = match room_settings.mode {
                    GameMode::Deathmatch => GameMode::Score,
                    GameMode::Score => GameMode::Deathmatch,
                };
            }
            7 => {
                room_settings.target_score =
                    ((room_settings.target_score as i32) + sign * 1000).clamp(1000, 20000) as u32;
            }
            ROW_TIME_LIMIT => {
                room_settings.time_limit_mins = match room_settings.time_limit_mins {
                    None => Some(if increase { 1 } else { 20 }),
                    Some(m) => {
                        let new_val = m as i32 + sign;
                        (1..=20).contains(&new_val).then_some(new_val as u32)
                    }
                };
            }
            ROW_MAP_LAYOUT => {
                room_settings.map_layout = if increase {
                    room_settings.map_layout.next()
                } else {
                    room_settings.map_layout.previous()
                };
            }
            ROW_BRICK_HINTS => room_settings.brick_hints = !room_settings.brick_hints,
            ROW_BOT_COUNT => {
                room_settings.bot_count =
                    ((room_settings.bot_count as i32) + sign).clamp(1, MAX_BOTS as i32) as u8;
            }
            ROW_BOT_LEVEL => {
                room_settings.bot_difficulty = if increase {
                    room_settings.bot_difficulty.next()
                } else {
                    room_settings.bot_difficulty.previous()
                };
            }
            _ => {}
        }
    }

    fn modify_local_skin(&mut self, profile_skin: &mut String, increase: bool) {
        let skins = common::game::models::ALL_SKINS;
        let current_idx = skins.iter().position(|&s| s == profile_skin).unwrap_or(0);
        let new_idx = if increase {
            (current_idx + 1) % skins.len()
        } else {
            (current_idx + skins.len() - 1) % skins.len()
        };
        *profile_skin = skins[new_idx].to_string();
    }
}
