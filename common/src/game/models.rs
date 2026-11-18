use crate::game::rhythm::BeatAccuracy;
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub use crate::game::bonuses::{BonusType, SecondItem};
pub use crate::game::map_layout::MapLayout;
pub use crate::game::match_stats::{HitStats, KillEvent};
pub use crate::game::room_settings::{MAX_BOTS, RoomSettings};
pub use crate::game::rules::*;
pub use crate::game::skins::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Deathmatch,
    Score,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BotDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl BotDifficulty {
    pub fn label(self) -> &'static str {
        match self {
            BotDifficulty::Easy => "Easy",
            BotDifficulty::Normal => "Normal",
            BotDifficulty::Hard => "Hard",
        }
    }

    pub fn next(self) -> Self {
        match self {
            BotDifficulty::Easy => BotDifficulty::Normal,
            BotDifficulty::Normal => BotDifficulty::Hard,
            BotDifficulty::Hard => BotDifficulty::Easy,
        }
    }

    pub fn previous(self) -> Self {
        self.next().next()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Direction {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl Direction {
    /// Grid-cell delta (x, y) for this direction.
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Wall,
    Brick,
    Bomb { owner_id: u32, ticks_left: u8, range: u8, pierce: bool },
    Explosion { ticks_left: u8, owner_id: u32 },
    Bonus { bonus_type: BonusType, ticks_left: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub is_host: bool,
    pub name: String,
    pub skin: String,
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
    pub is_ready: bool,
    pub death_beat: Option<u64>,
    #[serde(default)]
    pub kills: u32,
    #[serde(default)]
    pub pierce: bool,
    #[serde(default)]
    pub facing: Direction,
    #[serde(default)]
    pub hits: HitStats,
}

impl Player {
    pub fn new(id: u32, name: impl Into<String>, skin: impl Into<String>, lives: u8) -> Self {
        Self {
            id,
            is_host: false,
            name: name.into(),
            skin: skin.into(),
            sub_x: 0,
            sub_y: 0,
            is_alive: true,
            score: 0,
            combo: 0,
            max_bombs: 1,
            active_bombs: 0,
            bomb_range: 1,
            last_acted_beat: None,
            last_accuracy: BeatAccuracy::Waiting,
            last_action_time: None,
            spam_lockout_until: None,
            active_emote: None,
            emote_until: None,
            lives,
            death_pos: None,
            respawn_timer: None,
            collected_bonuses: Vec::new(),
            is_spectator: false,
            second_item: None,
            shield_until_beat: None,
            is_ready: false,
            death_beat: None,
            kills: 0,
            pierce: false,
            facing: Direction::Down,
            hits: HitStats::default(),
        }
    }

    /// Resets every power-up collected during the current life.
    pub fn reset_power_ups(&mut self) {
        self.collected_bonuses.clear();
        self.second_item = None;
        self.max_bombs = 1;
        self.bomb_range = 1;
        self.pierce = false;
    }

    pub fn combo_multiplier(&self) -> u32 {
        (1 + self.combo / COMBO_STEP).min(MAX_COMBO_MULTIPLIER)
    }

    pub fn is_in_fever(&self) -> bool {
        self.combo >= FEVER_COMBO
    }

    /// Range of the next bomb, including the Fever bonus.
    pub fn effective_bomb_range(&self) -> usize {
        if self.is_in_fever() {
            self.bomb_range + FEVER_BONUS_RANGE
        } else {
            self.bomb_range
        }
    }

    pub fn is_protected(&self, current_beat: u64) -> bool {
        self.shield_until_beat
            .is_some_and(|until| current_beat <= until)
    }

    /// `true` if an input made at `now` would be refused by the anti-spam rule.
    pub fn would_be_throttled(&self, now: Instant) -> bool {
        self.spam_lockout_until.is_some_and(|lockout| now < lockout)
            || self
                .last_action_time
                .is_some_and(|last| now.duration_since(last) < crate::game::rules::SPAM_MIN_INTERVAL)
    }

    pub fn try_consume_action_lockout(&mut self) -> bool {
        let now = Instant::now();

        // Presses during the lockout are refused but do not extend it: otherwise
        // pressing on every beat at high BPM (beat < 300 ms) locks forever.
        if self.spam_lockout_until.is_some_and(|lockout| now < lockout) {
            return false;
        }

        if let Some(last_time) = self.last_action_time {
            let delay = now.duration_since(last_time);

            if delay < crate::game::rules::SPAM_MIN_INTERVAL {
                self.spam_lockout_until = Some(now + crate::game::rules::SPAM_LOCKOUT);
                self.last_action_time = Some(now);
                return false;
            }
        }

        self.last_action_time = Some(now);
        true
    }
}


#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;
