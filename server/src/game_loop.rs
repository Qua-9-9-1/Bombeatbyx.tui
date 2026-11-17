//! Authoritative simulation loop: ticks every running room and broadcasts the
//! game state when it changed.
//!
//! The global lock is held only to tick and clone states. Serializing them to
//! detect changes (the costly part) happens outside the lock, so connection
//! handlers and Pongs are not delayed by busy servers.

use crate::state::SharedState;
use crate::websockets::rooms::stop_game_in_room;
use common::game::GameState;
use common::messages::ServerMessage;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

const TICK: Duration = Duration::from_millis(16);

fn fingerprint(state: &GameState) -> Option<u64> {
    let json = serde_json::to_string(state).ok()?;
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    Some(hasher.finish())
}

/// Keeps the snapshots whose fingerprint differs from the last one sent, and
/// forgets rooms that are no longer playing.
fn changed_snapshots(
    snapshots: Vec<(String, GameState)>,
    last_sent: &mut HashMap<String, u64>,
) -> Vec<(String, GameState)> {
    last_sent.retain(|code, _| snapshots.iter().any(|(c, _)| c == code));
    snapshots
        .into_iter()
        .filter(|(code, state)| {
            let Some(print) = fingerprint(state) else {
                return false;
            };
            last_sent.insert(code.clone(), print) != Some(print)
        })
        .collect()
}

pub async fn run(state: SharedState) {
    let mut interval = tokio::time::interval(TICK);
    let mut last_sent: HashMap<String, u64> = HashMap::new();
    loop {
        interval.tick().await;

        let mut snapshots = Vec::new();
        {
            let mut s = state.lock().await;
            for room in s.rooms.values_mut().filter(|r| r.in_game) {
                let Some(ref mut ctx) = room.game_ctx else {
                    continue;
                };
                ctx.tick_game_logic();
                if ctx.state.update_match_end() {
                    let victory_state = Some(ctx.state.clone());
                    stop_game_in_room(room, victory_state);
                    continue;
                }
                snapshots.push((room.code.clone(), ctx.state.clone()));
            }
        }

        let updates = changed_snapshots(snapshots, &mut last_sent);
        if !updates.is_empty() {
            let s = state.lock().await;
            for (code, game_state) in updates {
                if let Some(room) = s.rooms.get(&code) {
                    room.broadcast(ServerMessage::GameStateUpdate(game_state));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_state_is_sent_only_once() {
        let mut last_sent = HashMap::new();
        let state = GameState::new(9, 9);

        let first = changed_snapshots(vec![("ROOM01".into(), state.clone())], &mut last_sent);
        let second = changed_snapshots(vec![("ROOM01".into(), state)], &mut last_sent);

        assert_eq!(first.len(), 1);
        assert!(second.is_empty());
    }

    #[test]
    fn stopped_rooms_are_forgotten() {
        let mut last_sent = HashMap::new();
        changed_snapshots(vec![("ROOM01".into(), GameState::new(9, 9))], &mut last_sent);

        changed_snapshots(Vec::new(), &mut last_sent);

        assert!(last_sent.is_empty());
    }
}
