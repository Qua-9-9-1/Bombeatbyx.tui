//! Gameplay tuning constants shared by client, server and AI.

pub const DEFAULT_BONUS_LIFETIME_TICKS: u8 = 20;
pub const BRICK_BONUS_LIFETIME_TICKS: u8 = 60;
pub const BRICK_DENSITY_PERCENT: u32 = 65;
pub const BRICK_BONUS_DROP_PERCENT: u32 = 35;
/// Cells within this Manhattan distance of a spawn never receive bricks.
pub const SPAWN_SAFE_RADIUS: i32 = 2;

pub const KILL_POINTS: u32 = 300;
pub const SELF_KILL_PENALTY: u32 = 150;
pub const BRICK_POINTS: u32 = 10;
pub const BONUS_PICKUP_POINTS: u32 = 25;

pub const MAX_LIVES: u8 = 9;
pub const DASH_DISTANCE: u32 = 3;

/// Every `COMBO_STEP` consecutive on-beat actions raise the score multiplier by one.
pub const COMBO_STEP: u32 = 8;
pub const MAX_COMBO_MULTIPLIER: u32 = 4;
/// From this combo, bombs gain one extra range ("Fever").
pub const FEVER_COMBO: u32 = 16;
pub const FEVER_BONUS_RANGE: usize = 1;
pub const BOMB_FUSE_BEATS: u8 = 4;
pub const SHIELD_DURATION_BEATS: u64 = 3;
pub const RESPAWN_PROTECTION_BEATS: u64 = 3;
