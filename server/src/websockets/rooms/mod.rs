pub mod handlers;

use crate::state::{Room, SharedState};
use common::game::{GameContext, GameState};
use common::messages::{ClientMessage, ServerMessage};
use tokio::sync::mpsc::UnboundedSender;

use handlers::*;

pub async fn process_client_message(
    my_room_code: &mut Option<String>,
    my_player_id: &mut Option<u32>,
    my_name: &mut String,
    my_skin: &mut String,
    tx: UnboundedSender<ServerMessage>,
    client_msg: ClientMessage,
    state: &SharedState,
    client_ip: std::net::IpAddr,
) -> Result<(), ()> {
    match client_msg {
        ClientMessage::GetRooms => handle_get_rooms(&tx, state).await?,
        ClientMessage::CreateRoom { is_public, is_lan } => {
            handle_create_room(
                my_room_code,
                my_player_id,
                my_name,
                my_skin,
                &tx,
                state,
                client_ip,
                is_public,
                is_lan,
            )
            .await?;
        }
        ClientMessage::JoinRoom { code, name, skin } => {
            handle_join_room(
                my_room_code,
                my_player_id,
                my_name,
                my_skin,
                &tx,
                state,
                client_ip,
                code,
                name,
                skin,
            )
            .await?;
        }
        ClientMessage::UpdateSettings(settings) => {
            handle_update_settings(&*my_room_code, &*my_player_id, state, settings).await?;
        }
        ClientMessage::StartGame => {
            handle_start_game(&*my_room_code, &*my_player_id, state).await?;
        }
        ClientMessage::ToggleReady => {
            handle_toggle_ready(&*my_room_code, &*my_player_id, state).await?;
        }
        ClientMessage::Action(action) => {
            handle_action(&*my_room_code, &*my_player_id, state, action).await?;
        }
        ClientMessage::LeaveLobby => {
            return Err(());
        }
        ClientMessage::Pong => {}
        ClientMessage::StopGame => {
            handle_stop_game(&*my_room_code, &*my_player_id, state).await?;
        }
        ClientMessage::TransferHost(target_id) => {
            handle_transfer_host(&*my_room_code, &*my_player_id, state, target_id).await?;
        }
        ClientMessage::KickPlayer(target_id) => {
            handle_kick_player(&*my_room_code, &*my_player_id, state, target_id).await?;
        }
        ClientMessage::BanPlayer(target_id) => {
            handle_ban_player(&*my_room_code, &*my_player_id, state, target_id).await?;
        }
    }
    Ok(())
}

pub async fn disconnect_peer(code: &str, player_id: u32, state: &SharedState) {
    let mut s = state.lock().await;
    let mut remove_room = false;
    if let Some(room) = s.rooms.get_mut(code) {
        room.peers.remove(&player_id);

        if let Some(ref mut ctx) = room.game_ctx {
            ctx.state.players.retain(|p| p.id != player_id);
        }

        if room.peers.is_empty() {
            remove_room = true;
        } else {
            if room.host_id == Some(player_id) {
                room.host_id = room.peers.keys().min().copied();
            }
            if room.in_game {
                let active_players = room.peers.values().filter(|p| !p.is_spectator).count();
                if active_players <= 1 {
                    stop_game_in_room(room, None);
                    return;
                }
            }
            let players = room.get_lobby_players();
            let settings = room.room_settings.clone();
            if room.in_game {
                for peer in room.peers.values().filter(|p| p.is_spectator) {
                    let _ = peer.tx.send(ServerMessage::LobbyUpdate {
                        players: players.clone(),
                        settings: settings.clone(),
                    });
                }
            } else {
                room.broadcast(ServerMessage::LobbyUpdate { players, settings });
            }
        }
    }
    if remove_room {
        s.rooms.remove(code);
    }
}

pub fn stop_game_in_room(room: &mut Room, victory_state: Option<GameState>) {
    room.in_game = false;
    room.game_ctx = None;
    for peer in room.peers.values_mut() {
        peer.is_ready = false;
        peer.is_spectator = false;
    }
    let players = room.get_lobby_players();
    let settings = room.room_settings.clone();
    room.broadcast(ServerMessage::GameStopped { players, settings, victory_state });
}

pub fn start_game_in_room(room: &mut Room) {
    let mut players = room.get_lobby_players();
    for p in &mut players {
        p.lives = room.room_settings.lives;
        p.is_alive = !p.is_spectator;
        p.death_pos = None;
        p.respawn_timer = None;
        p.active_bombs = 0;
        p.max_bombs = 1;
        p.bomb_range = 1;
        p.collected_bonuses.clear();
        p.second_item = if p.id == 2 {
            Some(common::game::models::SecondItem::Shield)
        } else {
            None
        };
        p.shield_until_beat = None;
        p.combo = 0;
    }

    let mut new_state = GameState::new(room.room_settings.width, room.room_settings.height);
    new_state.bpm = room.room_settings.bpm;
    new_state.sudden_death = room.room_settings.sudden_death;
    new_state.bonus_every = room.room_settings.bonus_every;
    new_state.mode = room.room_settings.mode;
    new_state.target_score = room.room_settings.target_score;
    new_state.time_limit_mins = room.room_settings.time_limit_mins;
    new_state.spawn_players(players);

    let mut ctx = GameContext::new(
        room.room_settings.width,
        room.room_settings.height,
        room.room_settings.bpm,
    );
    ctx.state = new_state;

    room.game_ctx = Some(ctx);
    room.in_game = true;

    room.broadcast(ServerMessage::GameStarted {
        initial_state: room
            .game_ctx
            .as_ref()
            .expect("game_ctx should be initialized")
            .state
            .clone(),
    });
}
