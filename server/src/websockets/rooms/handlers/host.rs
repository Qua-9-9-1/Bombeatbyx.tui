use crate::state::SharedState;
use crate::websockets::rooms::disconnect_peer;
use common::messages::ServerMessage;

pub async fn handle_transfer_host(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
    target_id: u32,
) -> Result<(), ()> {
    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            let is_host = room.host_id == Some(*player_id);
            if is_host && room.peers.contains_key(&target_id) {
                room.host_id = Some(target_id);
                let target_name = room
                    .peers
                    .get(&target_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                room.broadcast(ServerMessage::HostTransferred {
                    new_host_id: target_id,
                    new_host_name: target_name,
                });
                let players = room.get_lobby_players();
                let settings = room.room_settings.clone();
                room.broadcast(ServerMessage::LobbyUpdate { players, settings });
            }
        }
    }
    Ok(())
}

pub async fn handle_kick_player(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
    target_id: u32,
) -> Result<(), ()> {
    let mut target_found = false;
    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            let is_host = room.host_id == Some(*player_id);
            if is_host && room.peers.contains_key(&target_id) {
                let target_name = room
                    .peers
                    .get(&target_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                target_found = true;
                room.broadcast(ServerMessage::PlayerKicked {
                    player_id: target_id,
                    player_name: target_name,
                });
            }
        }
    }
    if target_found {
        if let Some(code) = my_room_code.as_deref() {
            disconnect_peer(code, target_id, state).await;
        }
    }
    Ok(())
}

pub async fn handle_ban_player(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
    target_id: u32,
) -> Result<(), ()> {
    let mut target_found = false;
    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            let is_host = room.host_id == Some(*player_id);
            if is_host && room.peers.contains_key(&target_id) {
                if let Some(peer) = room.peers.get(&target_id) {
                    let target_name = peer.name.clone();
                    room.banned_ips.insert(peer.ip);
                    target_found = true;
                    room.broadcast(ServerMessage::PlayerBanned {
                        player_id: target_id,
                        player_name: target_name,
                    });
                }
            }
        }
    }
    if target_found {
        if let Some(code) = my_room_code.as_deref() {
            disconnect_peer(code, target_id, state).await;
        }
    }
    Ok(())
}
