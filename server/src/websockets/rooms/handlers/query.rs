use crate::state::SharedState;
use common::messages::{RoomInfo, ServerMessage};
use tokio::sync::mpsc::UnboundedSender;

pub async fn handle_get_rooms(
    tx: &UnboundedSender<ServerMessage>,
    state: &SharedState,
) -> Result<(), ()> {
    let s = state.lock().await;
    let mut room_infos = Vec::new();
    for room in s.rooms.values() {
        if room.is_public && !room.is_lan {
            let host_name = room
                .peers
                .values()
                .find(|p| Some(p.id) == room.host_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            room_infos.push(RoomInfo {
                code: room.code.clone(),
                host_name,
                player_count: room.peers.len(),
                max_players: 8,
                is_public: room.is_public,
                is_lan: room.is_lan,
            });
        }
    }
    let _ = tx.send(ServerMessage::RoomList(room_infos));
    Ok(())
}
