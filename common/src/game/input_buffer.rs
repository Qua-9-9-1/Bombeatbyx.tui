//! Off-beat inputs are not lost: they wait for the next beat.
//!
//! Each player owns a single slot. An input pressed outside the hit window is
//! stored and runs exactly on the next beat, without rhythm points and with a
//! combo reset. Further presses are ignored until the slot is consumed, so
//! mashing keys only ever produces one combo-less action per beat.

use super::actions::GameAction;
use super::context::GameContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferedAction {
    pub action: GameAction,
    /// Beat on which the action will run.
    pub beat: u64,
}

impl GameContext {
    /// `true` while the player already has an input waiting for a beat.
    pub fn has_buffered_action(&self, player_id: u32) -> bool {
        self.buffered_actions.contains_key(&player_id)
    }

    /// Stores an off-beat input for the next beat. Returns `false` when ignored.
    pub(crate) fn buffer_action(&mut self, player_id: u32, action: GameAction) -> bool {
        let state = &mut self.state;
        if state.countdown.is_some() || state.game_over_countdown.is_some() {
            return false;
        }
        if self.buffered_actions.contains_key(&player_id) {
            return false;
        }
        let beat = self.rhythm.beat_count + 1;
        let Some(player) = state.players.iter_mut().find(|p| p.id == player_id) else {
            return false;
        };
        if !player.is_alive || player.last_acted_beat == Some(beat) {
            return false;
        }
        if !player.try_consume_action_lockout() {
            return false;
        }
        self.buffered_actions
            .insert(player_id, BufferedAction { action, beat });
        true
    }

    /// Runs every buffered input due on `current_beat`. Called when a beat starts
    /// (by `tick_game_logic`, and by online clients for their local prediction).
    pub fn flush_buffered_actions(&mut self, current_beat: u64) {
        let due: Vec<(u32, BufferedAction)> = self
            .buffered_actions
            .iter()
            .filter(|(_, buffered)| buffered.beat <= current_beat)
            .map(|(id, buffered)| (*id, *buffered))
            .collect();
        for (player_id, buffered) in due {
            self.buffered_actions.remove(&player_id);
            self.state
                .handle_buffered_action(player_id, buffered.action, current_beat);
        }
    }
}

#[cfg(test)]
#[path = "input_buffer_tests.rs"]
mod tests;
