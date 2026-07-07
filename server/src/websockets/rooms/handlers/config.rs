use crate::state::SharedState;
use crate::websockets::rooms::start_game_in_room;
use common::game::models::RoomSettings;
use common::messages::ServerMessage;

pub async fn handle_update_settings(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
    settings: RoomSettings,
) -> Result<(), ()> {
    if let Some(code) = my_room_code {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            let is_host = room.host_id == *my_player_id;
            if is_host && !room.in_game {
                room.room_settings = settings;
                if let Some(pid) = my_player_id.as_ref() {
                    if let Some(host_peer) = room.peers.get_mut(pid) {
                        host_peer.is_ready = false;
                    }
                }
                let players = room.get_lobby_players();
                let settings = room.room_settings.clone();
                room.broadcast(ServerMessage::LobbyUpdate { players, settings });
            }
        }
    }
    Ok(())
}

pub async fn handle_toggle_ready(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
) -> Result<(), ()> {
    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            if !room.in_game {
                let mut trigger_start = false;
                if let Some(peer) = room.peers.get(player_id) {
                    let is_now_ready = !peer.is_ready;
                    let original_skin = peer.skin.clone();

                    let mut allow_ready = true;
                    if is_now_ready {
                        let skin_taken = room.peers.values().any(|other| {
                            other.id != *player_id && other.is_ready && other.skin == original_skin
                        });
                        if skin_taken {
                            allow_ready = false;
                        }
                    }

                    if let Some(peer_mut) = room.peers.get_mut(player_id) {
                        if allow_ready {
                            peer_mut.is_ready = is_now_ready;
                        }
                    }

                    let non_spectator_count =
                        room.peers.values().filter(|p| !p.is_spectator).count();
                    let all_ready = non_spectator_count >= 2
                        && room
                            .peers
                            .values()
                            .filter(|p| !p.is_spectator)
                            .all(|p| p.is_ready);
                    if all_ready {
                        trigger_start = true;
                    } else {
                        let players = room.get_lobby_players();
                        let settings = room.room_settings.clone();
                        room.broadcast(ServerMessage::LobbyUpdate { players, settings });
                    }
                }

                if trigger_start {
                    start_game_in_room(room);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Room, Peer, ServerState};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn toggle_ready_blocks_when_skin_already_taken_by_ready_player() {
        let state = Arc::new(Mutex::new(ServerState::new()));
        let mut room = Room::new("TEST".to_string(), true, false);
        room.host_id = Some(1);

        let (tx, _) = unbounded_channel();
        let ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();

        room.peers.insert(1, Peer {
            id: 1,
            name: "Player 1".to_string(),
            skin: "🤖".to_string(),
            tx: tx.clone(),
            is_ready: true,
            is_spectator: false,
            ip,
        });

        room.peers.insert(2, Peer {
            id: 2,
            name: "Player 2".to_string(),
            skin: "🤖".to_string(),
            tx: tx.clone(),
            is_ready: false,
            is_spectator: false,
            ip,
        });

        {
            let mut s = state.lock().await;
            s.rooms.insert("TEST".to_string(), room);
        }

        let res = handle_toggle_ready(
            &Some("TEST".to_string()),
            &Some(2),
            &state,
        ).await;

        assert!(res.is_ok());
        let s = state.lock().await;
        let room_after = s.rooms.get("TEST").unwrap();
        assert!(!room_after.peers.get(&2).unwrap().is_ready);
    }
}
