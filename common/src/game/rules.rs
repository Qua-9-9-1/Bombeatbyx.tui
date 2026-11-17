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

/// Hit windows, as a fraction of the beat interval on each side of the beat.
/// One action per beat and the miss penalty keep wide windows spam-proof.
pub const PERFECT_WINDOW: f64 = 0.06;
pub const GREAT_WINDOW: f64 = 0.12;
pub const OK_WINDOW: f64 = 0.20;

/// Every `COMBO_STEP` consecutive on-beat actions raise the score multiplier by one.
pub const COMBO_STEP: u32 = 8;
pub const MAX_COMBO_MULTIPLIER: u32 = 4;
/// From this combo, bombs gain one extra range ("Fever").
pub const FEVER_COMBO: u32 = 16;
pub const FEVER_BONUS_RANGE: usize = 1;
pub const BOMB_FUSE_BEATS: u8 = 4;
pub const SHIELD_DURATION_BEATS: u64 = 3;
pub const RESPAWN_PROTECTION_BEATS: u64 = 3;

/// Room settings bounds, enforced by the lobby UI and re-checked by the server.
pub const MAP_SIZE_RANGE: std::ops::RangeInclusive<usize> = 7..=29;
pub const BPM_RANGE: std::ops::RangeInclusive<f64> = 40.0..=220.0;
pub const BONUS_EVERY_RANGE: std::ops::RangeInclusive<u32> = 1..=60;
pub const LIVES_RANGE: std::ops::RangeInclusive<u8> = 1..=10;
pub const TARGET_SCORE_RANGE: std::ops::RangeInclusive<u32> = 1000..=20000;
pub const TIME_LIMIT_RANGE: std::ops::RangeInclusive<u32> = 1..=20;
