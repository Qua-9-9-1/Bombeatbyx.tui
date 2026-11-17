use super::actions::GameAction;
use super::models::{Player, RoomSettings};
use super::rhythm::{BeatAccuracy, RhythmEngine};
use super::state::GameState;
use std::time::{Duration, Instant};

/// Upper bound of the latency compensation applied to a player's input.
pub const MAX_LATENCY_COMPENSATION: Duration = Duration::from_millis(150);

pub struct GameContext {
    pub state: GameState,
    pub rhythm: RhythmEngine,
    pub last_closed_window_beat: Option<u64>,
    pub start_time: std::time::Instant,
}

impl GameContext {
    pub fn new(width: usize, height: usize, bpm: f64) -> Self {
        Self {
            state: GameState::new(width, height),
            rhythm: RhythmEngine::new(bpm),
            last_closed_window_beat: None,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn new_match(settings: &RoomSettings, players: Vec<Player>) -> Self {
        let mut ctx = Self::new(settings.width, settings.height, settings.bpm);
        ctx.state = GameState::new_match(settings, players);
        ctx
    }

    pub fn tick_game_logic(&mut self) -> bool {
        self.state.tick_respawns();
        self.state.tick_emotes();
        if self.state.countdown.is_none() && self.state.game_over_countdown.is_none() {
            self.state.elapsed_time_secs = self.start_time.elapsed().as_secs() as u32;
        } else {
            self.start_time = std::time::Instant::now() - std::time::Duration::from_secs(self.state.elapsed_time_secs as u64);
        }

        let has_beat_ticked = self.rhythm.tick_logic();

        let current_beat = self.rhythm.beat_count;
        self.state.current_beat = current_beat;
        let progress = self.rhythm.progress();
        let is_playing = self.state.countdown.is_none() && self.state.game_over_countdown.is_none();

        if progress > crate::game::rules::OK_WINDOW && progress <= 0.5 {
            if self.last_closed_window_beat != Some(current_beat) {
                for player in &mut self.state.players {
                    if player.last_acted_beat != Some(current_beat) {
                        player.last_accuracy = BeatAccuracy::Waiting;
                    }
                }
                if is_playing {
                    self.state.reset_idle_combos(current_beat);
                }
                self.last_closed_window_beat = Some(current_beat);
            }
        }

        if has_beat_ticked {
            if let Some(c) = self.state.countdown {
                if c > 0 {
                    self.state.countdown = Some(c - 1);
                } else {
                    self.state.countdown = None;
                }
            } else if let Some(g) = self.state.game_over_countdown {
                if g > 0 {
                    self.state.game_over_countdown = Some(g - 1);
                } else {
                    self.state.game_over_countdown = Some(0);
                }
            }

            if is_playing {
                self.state.tick_beat(current_beat);
                self.state.tick_bombs_and_explosions();
                self.state.tick_sudden_death(current_beat);
            }
        }

        // Checked every tick so that walking into a live blast is lethal too.
        if is_playing {
            self.state.check_deaths(current_beat);
        }
        has_beat_ticked
    }

    /// Shifts every wall-clock deadline forward, as if `paused` never elapsed.
    /// Used when a local (offline) game is paused.
    pub fn shift_clock(&mut self, paused: std::time::Duration) {
        self.rhythm.last_beat_time += paused;
        self.rhythm.next_beat_time += paused;
        self.start_time += paused;
        for player in &mut self.state.players {
            if let Some(ref mut timer) = player.respawn_timer {
                *timer += paused;
            }
            if let Some(ref mut until) = player.emote_until {
                *until += paused;
            }
            if let Some(ref mut until) = player.spam_lockout_until {
                *until += paused;
            }
        }
    }

    pub fn process_player_action(&mut self, player_id: u32, action: GameAction) {
        self.process_player_action_with_latency(player_id, action, Duration::ZERO);
    }

    /// Judges the action at the moment the player pressed the key: reception
    /// time minus the one-way network delay (capped so it cannot be abused).
    pub fn process_player_action_with_latency(
        &mut self,
        player_id: u32,
        action: GameAction,
        one_way_latency: Duration,
    ) {
        let compensation = one_way_latency.min(MAX_LATENCY_COMPENSATION);
        let now = Instant::now();
        let pressed_at = now.checked_sub(compensation).unwrap_or(now);
        let (target_beat, accuracy) = self.rhythm.timing_at(pressed_at);

        self.state
            .handle_action(player_id, action, accuracy, target_beat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_game_logic_updates_elapsed_time() {
        let mut context = GameContext::new(5, 5, 60.0);
        context.start_time = std::time::Instant::now() - std::time::Duration::from_secs(5);

        context.tick_game_logic();

        assert!(context.state.elapsed_time_secs >= 5);
    }

    #[test]
    fn process_player_action_with_latency_judges_input_at_press_time() {
        let mut context = GameContext::new(5, 5, 60.0);
        context.state.players = vec![crate::game::models::Player::new(1, "P1", "🤖", 3)];
        let interval = context.rhythm.beat_interval;
        context.rhythm.beat_count = 8;
        // Received 100 ms after the beat, but pressed 80 ms earlier (latency).
        context.rhythm.last_beat_time = Instant::now() - interval.mul_f64(0.10);
        context.rhythm.next_beat_time = context.rhythm.last_beat_time + interval;

        context.process_player_action_with_latency(
            1,
            GameAction::MoveDown,
            Duration::from_millis(80),
        );

        assert_eq!(context.state.players[0].last_accuracy, BeatAccuracy::Perfect);
    }

    #[test]
    fn process_player_action_with_latency_caps_compensation() {
        let mut context = GameContext::new(5, 5, 60.0);
        context.state.players = vec![crate::game::models::Player::new(1, "P1", "🤖", 3)];
        let interval = context.rhythm.beat_interval;
        context.rhythm.beat_count = 8;
        context.rhythm.last_beat_time = Instant::now() - interval.mul_f64(0.40);
        context.rhythm.next_beat_time = context.rhythm.last_beat_time + interval;

        context.process_player_action_with_latency(
            1,
            GameAction::MoveDown,
            Duration::from_millis(390),
        );

        assert_eq!(context.state.players[0].last_accuracy, BeatAccuracy::Miss);
    }

    #[test]
    fn shift_clock_prevents_beats_from_catching_up_after_pause() {
        let mut context = GameContext::new(5, 5, 60.0);
        let paused = std::time::Duration::from_secs(10);
        context.rhythm.last_beat_time -= paused;
        context.rhythm.next_beat_time -= paused;

        context.shift_clock(paused);
        let has_ticked = context.rhythm.tick_logic();

        assert!(!has_ticked);
    }
}
