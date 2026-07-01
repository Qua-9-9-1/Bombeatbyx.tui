use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Wall,
    Brick,
    Bomb { owner_id: u32, ticks_left: u8 },
    Explosion { ticks_left: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub sub_x: i32,
    pub sub_y: i32,
    pub is_alive: bool,
    pub score: u32,
    pub max_bombs: u8,
    pub active_bombs: u8,
    pub bomb_range: usize,
    pub last_acted_beat: Option<u64>,
    #[serde(skip)]
    pub last_action_time: Option<Instant>,
    #[serde(skip)]
    pub spam_lockout_until: Option<Instant>,
}
