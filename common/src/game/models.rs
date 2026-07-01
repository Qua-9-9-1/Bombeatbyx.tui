use serde::{Deserialize, Serialize};

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
    pub max_bombs: u8,
    pub active_bombs: u8,
    pub bomb_range: usize,
    pub score: u32,
}
