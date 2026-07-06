use crate::state::SharedState;
use crate::websockets::rooms::{start_game_in_room, stop_game_in_room};
use common::game::actions::GameAction;

pub async fn handle_start_game(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
) -> Result<(), ()> {
    if let Some(code) = my_room_code {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            let is_host = room.host_id == *my_player_id;
            if is_host && !room.in_game {
                let non_spectator_count = room.peers.values().filter(|p| !p.is_spectator).count();
                let everyone_ready = non_spectator_count >= 2
                    && room
                        .peers
                        .values()
                        .filter(|p| !p.is_spectator)
                        .all(|p| p.is_ready);
                if everyone_ready {
                    start_game_in_room(room);
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_action(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
    action: GameAction,
) -> Result<(), ()> {
    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            if room.in_game {
                if let Some(ref mut ctx) = room.game_ctx {
                    ctx.process_player_action(*player_id, action);
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_stop_game(
    my_room_code: &Option<String>,
    my_player_id: &Option<u32>,
    state: &SharedState,
) -> Result<(), ()> {
    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        let mut s = state.lock().await;
        if let Some(room) = s.rooms.get_mut(code) {
            let is_host = room.host_id == Some(*player_id);
            if is_host && room.in_game {
                stop_game_in_room(room);
            }
        }
    }
    Ok(())
}
