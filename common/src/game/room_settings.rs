//! Host-chosen room settings, sent over the network and re-validated by the server.

use crate::game::map_layout::MapLayout;
use crate::game::models::{BotDifficulty, GameMode};
use crate::game::rules::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    pub width: usize,
    pub height: usize,
    pub bpm: f64,
    pub sudden_death: bool,
    pub bonus_every: u32,
    pub lives: u8,
    pub mode: GameMode,
    pub target_score: u32,
    pub time_limit_mins: Option<u32>,
    /// Solo mode only: number of AI opponents.
    #[serde(default = "default_bot_count")]
    pub bot_count: u8,
    /// Solo mode only: AI skill level.
    #[serde(default)]
    pub bot_difficulty: BotDifficulty,
    #[serde(default)]
    pub map_layout: MapLayout,
    /// Bricks hiding a bonus are drawn cracked.
    #[serde(default = "default_brick_hints")]
    pub brick_hints: bool,
}

pub const MAX_BOTS: u8 = 7;

fn default_bot_count() -> u8 {
    3
}

fn default_brick_hints() -> bool {
    true
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
            target_score: 1000,
            time_limit_mins: None,
            bot_count: default_bot_count(),
            bot_difficulty: BotDifficulty::Normal,
            map_layout: MapLayout::Classic,
            brick_hints: default_brick_hints(),
        }
    }
}

fn clamp_odd(value: usize, range: std::ops::RangeInclusive<usize>) -> usize {
    let clamped = value.clamp(*range.start(), *range.end());
    if clamped % 2 == 0 { clamped - 1 } else { clamped }
}

impl RoomSettings {
    /// Returns a copy with every field forced into its valid range.
    /// Settings received from the network must always go through this.
    pub fn sanitized(&self) -> Self {
        let bpm = if self.bpm.is_finite() {
            self.bpm.clamp(*BPM_RANGE.start(), *BPM_RANGE.end())
        } else {
            Self::default().bpm
        };
        Self {
            width: clamp_odd(self.width, MAP_SIZE_RANGE),
            height: clamp_odd(self.height, MAP_SIZE_RANGE),
            bpm,
            sudden_death: self.sudden_death,
            bonus_every: self
                .bonus_every
                .clamp(*BONUS_EVERY_RANGE.start(), *BONUS_EVERY_RANGE.end()),
            lives: self.lives.clamp(*LIVES_RANGE.start(), *LIVES_RANGE.end()),
            mode: self.mode,
            target_score: self
                .target_score
                .clamp(*TARGET_SCORE_RANGE.start(), *TARGET_SCORE_RANGE.end()),
            time_limit_mins: self
                .time_limit_mins
                .map(|m| m.clamp(*TIME_LIMIT_RANGE.start(), *TIME_LIMIT_RANGE.end())),
            bot_count: self.bot_count.clamp(1, MAX_BOTS),
            bot_difficulty: self.bot_difficulty,
            map_layout: self.map_layout,
            brick_hints: self.brick_hints,
        }
    }
}
