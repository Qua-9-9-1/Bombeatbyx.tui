//! Never trust the server blindly: a buggy or malicious server must not be
//! able to crash the client (zero BPM, empty map, grid shorter than its size).

use common::game::GameState;
use common::messages::ServerMessage;

fn state_ok(state: &GameState) -> bool {
    state.is_well_formed()
}

/// Sanitizes settings and drops messages carrying an unusable game state.
pub fn sanitize_server_message(msg: ServerMessage) -> Option<ServerMessage> {
    match msg {
        ServerMessage::Joined { your_id, room_code, current_state, settings } => {
            state_ok(&current_state).then(|| ServerMessage::Joined {
                your_id,
                room_code,
                current_state,
                settings: settings.sanitized(),
            })
        }
        ServerMessage::LobbyUpdate { players, settings } => Some(ServerMessage::LobbyUpdate {
            players,
            settings: settings.sanitized(),
        }),
        ServerMessage::GameStarted { initial_state } => {
            state_ok(&initial_state).then_some(ServerMessage::GameStarted { initial_state })
        }
        ServerMessage::GameStateUpdate(state) => {
            state_ok(&state).then_some(ServerMessage::GameStateUpdate(state))
        }
        ServerMessage::GameStopped { players, settings, victory_state } => {
            Some(ServerMessage::GameStopped {
                players,
                settings: settings.sanitized(),
                victory_state: victory_state.filter(state_ok),
            })
        }
        other => Some(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::game::RoomSettings;

    #[test]
    fn malformed_state_update_is_dropped() {
        let mut state = GameState::new(9, 9);
        state.grid.clear();

        let result = sanitize_server_message(ServerMessage::GameStateUpdate(state));

        assert!(result.is_none());
    }

    #[test]
    fn lobby_settings_are_sanitized() {
        let mut settings = RoomSettings::default();
        settings.bpm = 0.0;

        let result = sanitize_server_message(ServerMessage::LobbyUpdate {
            players: Vec::new(),
            settings,
        });

        let Some(ServerMessage::LobbyUpdate { settings, .. }) = result else {
            panic!("lobby update should be kept");
        };
        assert!(settings.bpm > 0.0);
    }
}
