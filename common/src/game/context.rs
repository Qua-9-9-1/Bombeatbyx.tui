use super::actions::GameAction;
use super::rhythm::RhythmEngine;
use super::state::GameState;

pub struct GameContext {
    pub state: GameState,
    pub rhythm: RhythmEngine,
}

impl GameContext {
    pub fn new(width: usize, height: usize, bpm: f64) -> Self {
        Self {
            state: GameState::new(width, height),
            rhythm: RhythmEngine::new(bpm),
        }
    }

    pub fn tick_game_logic(&mut self) -> bool {
        if self.rhythm.tick_logic() {
            self.state.tick_bombs_and_explosions();
            self.state.check_deaths();
            true
        } else {
            false
        }
    }

    pub fn process_player_action(&mut self, player_id: u32, action: GameAction) {
        let accuracy = self.rhythm.evaluate_accuracy();
        let current_beat = self.rhythm.beat_count;

        self.state
            .handle_action(player_id, action, accuracy, current_beat);
    }
}
