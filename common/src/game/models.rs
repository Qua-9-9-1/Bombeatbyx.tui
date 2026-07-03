use serde::{Deserialize, Serialize};
use std::time::Instant;
use crate::game::rhythm::BeatAccuracy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Deathmatch,
    Score,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecondItem {
    Shield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusType {
    BombQty,
    BombRange,
    Shield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Wall,
    Brick,
    Bomb { owner_id: u32, ticks_left: u8 },
    Explosion { ticks_left: u8 },
    Bonus(BonusType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub is_host: bool,
    pub name: String,
    pub skin: String,
    pub color: String,
    pub sub_x: i32,
    pub sub_y: i32,
    pub is_alive: bool,
    pub score: u32,
    pub combo: u32,
    pub max_bombs: u8,
    pub active_bombs: u8,
    pub bomb_range: usize,
    pub last_acted_beat: Option<u64>,
    pub last_accuracy: BeatAccuracy,
    #[serde(skip)]
    pub last_action_time: Option<Instant>,
    #[serde(skip)]
    pub spam_lockout_until: Option<Instant>,
    pub active_emote: Option<String>,
    #[serde(skip)]
    pub emote_until: Option<Instant>,
    pub lives: u8,
    pub death_pos: Option<(i32, i32)>,
    #[serde(skip)]
    pub respawn_timer: Option<Instant>,
    pub collected_bonuses: Vec<String>,
    pub is_spectator: bool,
    pub second_item: Option<SecondItem>,
    pub shield_until_beat: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    pub width: usize,
    pub height: usize,
    pub bpm: f64,
    pub sudden_death: bool,
    pub bonus_every: u32,
    pub lives: u8,
    pub mode: GameMode,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            width: 15,
            height: 15,
            bpm: 60.0,
            sudden_death: false,
            bonus_every: 10,
            lives: 3,
            mode: GameMode::Deathmatch,
        }
    }
}