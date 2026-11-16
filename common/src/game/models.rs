use crate::game::rhythm::BeatAccuracy;
use crate::game::spawns::GameRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecondItem {
    Shield,
    Detonator,
    Dash,
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
pub enum BonusType {
    BombQty,
    BombRange,
    Shield,
    Pierce,
    ExtraLife,
    Detonator,
    Dash,
}

impl BonusType {
    /// Relative drop weights: common upgrades first, rare items last.
    const WEIGHTS: &'static [(BonusType, u32)] = &[
        (BonusType::BombQty, 28),
        (BonusType::BombRange, 28),
        (BonusType::Shield, 12),
        (BonusType::Dash, 10),
        (BonusType::Detonator, 9),
        (BonusType::Pierce, 8),
        (BonusType::ExtraLife, 5),
    ];

    pub fn roll(rng: &mut GameRng) -> BonusType {
        let total: u32 = Self::WEIGHTS.iter().map(|(_, w)| w).sum();
        let mut pick = rng.next_u32() % total;
        for &(bonus, weight) in Self::WEIGHTS {
            if pick < weight {
                return bonus;
            }
            pick -= weight;
        }
        BonusType::BombQty
    }
}

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

    pub fn try_consume_action_lockout(&mut self) -> bool {
        let now = Instant::now();

        if let Some(lockout) = self.spam_lockout_until {
            if now < lockout {
                self.spam_lockout_until = Some(now + Duration::from_millis(300));
                return false;
            }
        }

        if let Some(last_time) = self.last_action_time {
            let delay = now.duration_since(last_time);

            if delay < Duration::from_millis(100) {
                self.spam_lockout_until = Some(now + Duration::from_millis(300));
                self.last_action_time = Some(now);
                return false;
            }
        }

        self.last_action_time = Some(now);
        true
    }
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
    pub target_score: u32,
    pub time_limit_mins: Option<u32>,
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
        }
    }
}

pub const ALL_SKINS: &[&str] = &["👶", "😑", "😎", "🤓", "🥰", "😱", "🤠", "🥸", "👿", "🐷", "🐸", "🎃", "🗿", "😡", "💩", "🤡", "👹", "👺", "👽", "🤖", "🍔", "🍙", "🍪", "🍎", "🥔"];

pub fn get_skin_name(skin: &str) -> &'static str {
    match skin {
        "👶" => "Baby",
        "😑" => "Neutral",
        "😎" => "Cool",
        "🤓" => "Nerd",
        "🥰" => "Romantic",
        "😱" => "Scared",
        "🤠" => "Cowboy",
        "🥸" => "Disguised",
        "👿" => "Devil",
        "🐷" => "Pig",
        "🐸" => "Frog",
        "🎃" => "Jack-o'-Lantern",
        "🗿" => "Golem",
        "😡" => "Angry",
        "💩" => "Poop",
        "🤡" => "Clown",
        "👹" => "Ogre",
        "👺" => "Goblin",
        "👽" => "Alien",
        "🤖" => "Robot",
        "🍔" => "Burger",
        "🍙" => "Rice",
        "🍪" => "Cookie",
        "🍎" => "Apple",
        "🥔" => "Potato",
        _ => "Player",
    }
}

pub fn get_skin_short_code(skin: &str) -> &'static str {
    match skin {
        "👶" => "BB",
        "😑" => "NT",
        "😎" => "CL",
        "🤓" => "ND",
        "🥰" => "RM",
        "😱" => "SC",
        "🤠" => "CW",
        "🥸" => "DS",
        "👿" => "DV",
        "🐷" => "PG",
        "🐸" => "FR",
        "🎃" => "JK",
        "🗿" => "GL",
        "😡" => "AN",
        "💩" => "PT",
        "🤡" => "CL",
        "👹" => "OG",
        "👺" => "GB",
        "👽" => "AL",
        "🤖" => "RO",
        "🍔" => "BG",
        "🍙" => "RC",
        "🍪" => "CK",
        "🍎" => "AP",
        "🥔" => "PT",
        _ => "PL",
    }
}

pub fn is_valid_player_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || name.chars().count() > 16 {
        return false;
    }
    name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_player_name() {
        assert!(is_valid_player_name("Player1"));
        assert!(is_valid_player_name("Jean-Michel"));
        assert!(is_valid_player_name("Aôut_éè"));
        assert!(is_valid_player_name("Bot_1"));
        
        assert!(!is_valid_player_name(""));
        assert!(!is_valid_player_name("   "));
        assert!(!is_valid_player_name("PlayerNameIsTooLongToBeValidForThisGame"));
        assert!(!is_valid_player_name("Player#1"));
        assert!(!is_valid_player_name("🤖"));
    }

    #[test]
    fn combo_multiplier_grows_every_step_and_is_capped() {
        let mut player = Player::new(1, "Player 1", "🤖", 3);

        let multipliers: Vec<u32> = [0, COMBO_STEP, COMBO_STEP * 2, COMBO_STEP * 50]
            .iter()
            .map(|&combo| {
                player.combo = combo;
                player.combo_multiplier()
            })
            .collect();

        assert_eq!(multipliers, vec![1, 2, 3, MAX_COMBO_MULTIPLIER]);
    }

    #[test]
    fn effective_bomb_range_adds_fever_bonus() {
        let mut player = Player::new(1, "Player 1", "🤖", 3);
        player.bomb_range = 2;
        player.combo = FEVER_COMBO;

        let range = player.effective_bomb_range();

        assert_eq!(range, 2 + FEVER_BONUS_RANGE);
    }

    #[test]
    fn try_consume_action_lockout_allows_action_when_no_previous_action() {
        let mut player = Player::new(1, "Player 1", "🤖", 3);

        let allowed = player.try_consume_action_lockout();

        assert!(allowed);
        assert!(player.last_action_time.is_some());
    }

    #[test]
    fn try_consume_action_lockout_blocks_action_when_spam_lockout_active() {
        let mut player = Player::new(1, "Player 1", "🤖", 3);
        player.last_action_time = Some(Instant::now());
        player.spam_lockout_until = Some(Instant::now() + Duration::from_millis(500));

        let allowed = player.try_consume_action_lockout();

        assert!(!allowed);
    }
}
