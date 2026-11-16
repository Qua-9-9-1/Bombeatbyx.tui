//! Applies authoritative server snapshots: rhythm clock alignment and
//! reconciliation of the local player's predicted action.

use crate::local::app::App;
use common::game::models::BOMB_FUSE_BEATS;
use common::game::{Cell, GameAction, GameState, Player};
use std::time::Duration;

/// Server tick period: a beat snapshot leaves the server up to one tick late.
const SERVER_TICK: Duration = Duration::from_millis(16);
/// Clock drift ignored when aligning on the server beat (avoids jitter).
const BEAT_SYNC_TOLERANCE: Duration = Duration::from_millis(25);
/// A local prediction is trusted at most this long past the round-trip time.
const PREDICTION_GRACE: Duration = Duration::from_millis(150);

impl App {
    pub(crate) fn apply_server_state(&mut self, mut new_state: GameState) {
        let one_way = self.network.rtt / 2;
        let prediction_window = self.network.rtt + PREDICTION_GRACE;
        let local_action = self
            .last_local_action
            .filter(|(_, _, at)| at.elapsed() < prediction_window);
        let Some(ref mut ctx) = self.game_ctx else {
            return;
        };

        // The first snapshot of a beat is sent on the server tick where it started.
        if new_state.current_beat != ctx.state.current_beat {
            let since_beat = one_way + SERVER_TICK / 2;
            ctx.rhythm
                .align(new_state.current_beat, since_beat, BEAT_SYNC_TOLERANCE);
        }

        let local_player = ctx
            .state
            .players
            .iter()
            .find(|p| p.id == self.current_player_id)
            .cloned();
        if let Some(local) = local_player {
            if let Some(server_player) = new_state
                .players
                .iter_mut()
                .find(|p| p.id == self.current_player_id)
            {
                keep_client_only_timers(server_player, &local);
                let is_unconfirmed = local.last_acted_beat > server_player.last_acted_beat;
                if is_unconfirmed && local_action.is_some() {
                    keep_prediction(server_player, &local);
                    if let Some((beat, GameAction::PlaceBomb, _)) = local_action {
                        predict_bomb(&mut new_state.grid, new_state.width, &local, beat);
                    }
                }
            }
        }
        ctx.state = new_state;
    }
}

/// Fields that are never serialized and only exist on this client.
fn keep_client_only_timers(server_player: &mut Player, local: &Player) {
    server_player.last_action_time = local.last_action_time;
    server_player.spam_lockout_until = local.spam_lockout_until;
    server_player.emote_until = local.emote_until;
    server_player.respawn_timer = local.respawn_timer;
}

/// Keeps the locally predicted result until the server acknowledges the action.
fn keep_prediction(server_player: &mut Player, local: &Player) {
    server_player.sub_x = local.sub_x;
    server_player.sub_y = local.sub_y;
    server_player.last_acted_beat = local.last_acted_beat;
    server_player.last_accuracy = local.last_accuracy.clone();
    server_player.combo = local.combo;
    server_player.score = local.score;
    server_player.active_bombs = local.active_bombs;
    server_player.max_bombs = local.max_bombs;
    server_player.bomb_range = local.bomb_range;
    server_player.collected_bonuses = local.collected_bonuses.clone();
    server_player.second_item = local.second_item;
    server_player.pierce = local.pierce;
    server_player.facing = local.facing;
    server_player.shield_until_beat = local.shield_until_beat;
}

fn predict_bomb(grid: &mut [Cell], width: usize, local: &Player, predicted_beat: u64) {
    if local.last_acted_beat != Some(predicted_beat) {
        return;
    }
    let idx = (local.sub_y as usize) * width + (local.sub_x / 2) as usize;
    if grid.get(idx) == Some(&Cell::Empty) {
        grid[idx] = Cell::Bomb {
            owner_id: local.id,
            ticks_left: BOMB_FUSE_BEATS,
            range: local.effective_bomb_range().min(u8::MAX as usize) as u8,
            pierce: local.pierce,
        };
    }
}
