use super::GameState;
use crate::game::models::{BRICK_DENSITY_PERCENT, GameMode, Player, RoomSettings};
use crate::game::spawns::GameRng;

pub const MATCH_START_COUNTDOWN_BEATS: u32 = 4;
pub const MATCH_END_COUNTDOWN_BEATS: u32 = 4;

impl GameState {
    pub fn new_match(settings: &RoomSettings, mut players: Vec<Player>) -> Self {
        for p in &mut players {
            p.reset_for_match(settings.lives);
        }

        let mut state =
            GameState::with_layout(settings.width, settings.height, settings.map_layout);
        state.brick_hints = settings.brick_hints;
        state.bpm = settings.bpm;
        state.sudden_death = settings.sudden_death;
        state.bonus_every = settings.bonus_every;
        state.mode = settings.mode;
        state.target_score = settings.target_score;
        state.time_limit_mins = settings.time_limit_mins;
        state.countdown = Some(MATCH_START_COUNTDOWN_BEATS);
        state.spawn_players(players);
        state.generate_bricks(&mut GameRng::from_time(), BRICK_DENSITY_PERCENT);
        state
    }

    pub fn is_match_over(&self) -> bool {
        if let Some(limit_mins) = self.time_limit_mins {
            if self.elapsed_time_secs >= limit_mins * 60 {
                return true;
            }
        }

        match self.mode {
            GameMode::Score => self.players.iter().any(|p| p.score >= self.target_score),
            GameMode::Deathmatch => {
                let competitors = self.players.iter().filter(|p| !p.is_spectator).count();
                let survivors = self
                    .players
                    .iter()
                    .filter(|p| !p.is_spectator && p.lives > 0)
                    .count();
                if competitors > 1 {
                    survivors <= 1
                } else {
                    survivors == 0
                }
            }
        }
    }

    /// Starts the game-over countdown once the match is decided.
    /// Returns `true` when the countdown is finished and the match must end.
    pub fn update_match_end(&mut self) -> bool {
        if self.game_over_countdown.is_none() && self.countdown.is_none() && self.is_match_over() {
            self.game_over_countdown = Some(MATCH_END_COUNTDOWN_BEATS);
        }
        self.game_over_countdown == Some(0)
    }
}

impl Player {
    pub fn reset_for_match(&mut self, lives: u8) {
        self.lives = lives;
        self.is_alive = !self.is_spectator;
        self.score = 0;
        self.kills = 0;
        self.combo = 0;
        self.death_pos = None;
        self.death_beat = None;
        self.respawn_timer = None;
        self.active_bombs = 0;
        self.reset_power_ups();
        self.shield_until_beat = None;
        self.last_acted_beat = None;
        self.hits = Default::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(id: u32, lives: u8) -> Player {
        Player::new(id, format!("Player {}", id), "🤖", lives)
    }

    #[test]
    fn new_match_resets_players_and_starts_countdown() {
        let settings = RoomSettings::default();
        let mut veteran = player(1, 0);
        veteran.score = 999;
        veteran.max_bombs = 5;

        let state = GameState::new_match(&settings, vec![veteran, player(2, 3)]);

        assert_eq!(state.countdown, Some(MATCH_START_COUNTDOWN_BEATS));
        assert_eq!(state.players[0].lives, settings.lives);
        assert_eq!(state.players[0].score, 0);
        assert_eq!(state.players[0].max_bombs, 1);
        assert!(state.players[0].is_alive);
    }

    #[test]
    fn is_match_over_deathmatch_true_when_one_survivor_left() {
        let mut state = GameState::new(15, 15);
        state.players = vec![player(1, 2), player(2, 0)];

        let over = state.is_match_over();

        assert!(over);
    }

    #[test]
    fn is_match_over_deathmatch_false_when_two_survivors_left() {
        let mut state = GameState::new(15, 15);
        state.players = vec![player(1, 2), player(2, 1)];

        let over = state.is_match_over();

        assert!(!over);
    }

    #[test]
    fn is_match_over_ignores_spectators() {
        let mut state = GameState::new(15, 15);
        let mut spectator = player(3, 0);
        spectator.is_spectator = true;
        state.players = vec![player(1, 2), player(2, 1), spectator];

        let over = state.is_match_over();

        assert!(!over);
    }

    #[test]
    fn is_match_over_score_true_when_target_reached() {
        let mut state = GameState::new(15, 15);
        state.mode = GameMode::Score;
        state.target_score = 500;
        let mut leader = player(1, 3);
        leader.score = 500;
        state.players = vec![leader, player(2, 3)];

        let over = state.is_match_over();

        assert!(over);
    }

    #[test]
    fn is_match_over_true_when_time_limit_elapsed() {
        let mut state = GameState::new(15, 15);
        state.players = vec![player(1, 3), player(2, 3)];
        state.time_limit_mins = Some(1);
        state.elapsed_time_secs = 60;

        let over = state.is_match_over();

        assert!(over);
    }

    #[test]
    fn update_match_end_starts_countdown_when_match_decided() {
        let mut state = GameState::new(15, 15);
        state.players = vec![player(1, 2), player(2, 0)];

        let finished = state.update_match_end();

        assert!(!finished);
        assert_eq!(state.game_over_countdown, Some(MATCH_END_COUNTDOWN_BEATS));
    }
}
