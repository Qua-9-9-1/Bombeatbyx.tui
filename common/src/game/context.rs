use super::actions::GameAction;
use super::rhythm::{BeatAccuracy, RhythmEngine};
use super::state::GameState;

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

    pub fn tick_game_logic(&mut self) -> bool {
        self.state.tick_respawns();
        if self.state.countdown.is_none() && self.state.game_over_countdown.is_none() {
            self.state.elapsed_time_secs = self.start_time.elapsed().as_secs() as u32;
        } else {
            self.start_time = std::time::Instant::now() - std::time::Duration::from_secs(self.state.elapsed_time_secs as u64);
        }

        let has_beat_ticked = self.rhythm.tick_logic();

        let current_beat = self.rhythm.beat_count;
        let progress = self.rhythm.progress();

        if progress > 0.15 && progress <= 0.5 {
            if self.last_closed_window_beat != Some(current_beat) {
                for player in &mut self.state.players {
                    if player.last_acted_beat != Some(current_beat) {
                        player.last_accuracy = BeatAccuracy::Waiting;
                    }
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

            self.state.tick_beat(current_beat);
            if self.state.countdown.is_none() && self.state.game_over_countdown.is_none() {
                self.state.tick_bombs_and_explosions();
                self.state.check_deaths(current_beat);
            }
            true
        } else {
            false
        }
    }

    pub fn process_player_action(&mut self, player_id: u32, action: GameAction) {
        let accuracy = self.rhythm.evaluate_accuracy();
        let progress = self.rhythm.progress();
        let target_beat = if progress > 0.5 {
            self.rhythm.beat_count + 1
        } else {
            self.rhythm.beat_count
        };

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
}
