use crate::game::rhythm::BeatAccuracy;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

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
}

impl Player {
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
    fn try_consume_action_lockout_allows_action_when_no_previous_action() {
        let mut player = Player {
            id: 1,
            is_host: true,
            name: "Player 1".to_string(),
            skin: "🤖".to_string(),
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
            lives: 3,
            death_pos: None,
            respawn_timer: None,
            collected_bonuses: Vec::new(),
            is_spectator: false,
            second_item: None,
            shield_until_beat: None,
            is_ready: false,
            death_beat: None,
        };

        let allowed = player.try_consume_action_lockout();

        assert!(allowed);
        assert!(player.last_action_time.is_some());
    }

    #[test]
    fn try_consume_action_lockout_blocks_action_when_spam_lockout_active() {
        let mut player = Player {
            id: 1,
            is_host: true,
            name: "Player 1".to_string(),
            skin: "🤖".to_string(),
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
            last_action_time: Some(Instant::now()),
            spam_lockout_until: Some(Instant::now() + Duration::from_millis(500)),
            active_emote: None,
            emote_until: None,
            lives: 3,
            death_pos: None,
            respawn_timer: None,
            collected_bonuses: Vec::new(),
            is_spectator: false,
            second_item: None,
            shield_until_beat: None,
            is_ready: false,
            death_beat: None,
        };

        let allowed = player.try_consume_action_lockout();

        assert!(!allowed);
    }
}
