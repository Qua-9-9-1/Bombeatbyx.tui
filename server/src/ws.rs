use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::state::{SharedState, Peer, get_color_for_id};
use common::messages::{ClientMessage, ServerMessage};
use common::game::{GameContext, GameState};

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut ws_write, mut ws_read) = socket.split();
    let (tx, mut rx) = unbounded_channel::<ServerMessage>();

    let my_id = register_peer(tx.clone(), &state).await;

    let mut write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if ws_write.send(Message::Text(json_str.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let recv_state = state.clone();
    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_read.next().await {
            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                if process_client_message(my_id, client_msg, &recv_state).await.is_err() {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = &mut write_task => {}
        _ = &mut read_task => {}
    }

    disconnect_peer(my_id, &state).await;
}

async fn register_peer(tx: UnboundedSender<ServerMessage>, state: &SharedState) -> u32 {
    let mut s = state.lock().await;
    let id = s.next_peer_id;
    s.next_peer_id += 1;

    if s.host_id.is_none() {
        s.host_id = Some(id);
    }
    let color = get_color_for_id(id);

    let peer = Peer {
        id,
        name: format!("Player{}", id),
        skin: "🤖".to_string(),
        color,
        tx,
    };

    s.peers.insert(id, peer);

    let initial_lobby = GameState::new(s.room_settings.width, s.room_settings.height);
    let joined_msg = ServerMessage::Joined {
        your_id: id,
        current_state: initial_lobby,
        settings: s.room_settings.clone(),
    };
    let _ = s.peers.get(&id).unwrap().tx.send(joined_msg);

    let players = s.get_lobby_players();
    s.broadcast(ServerMessage::LobbyUpdate {
        players,
        settings: s.room_settings.clone(),
    });

    id
}

async fn process_client_message(
    my_id: u32,
    client_msg: ClientMessage,
    state: &SharedState,
) -> Result<(), ()> {
    let mut s = state.lock().await;
    match client_msg {
        ClientMessage::JoinLobby { name, skin } => {
            if let Some(peer) = s.peers.get_mut(&my_id) {
                peer.name = name;
                peer.skin = skin;
            }
            let players = s.get_lobby_players();
            let settings = s.room_settings.clone();
            s.broadcast(ServerMessage::LobbyUpdate { players, settings });
        }
        ClientMessage::UpdateSettings(settings) => {
            let is_host = s.host_id == Some(my_id);
            if is_host && !s.in_game {
                s.room_settings = settings;
                let players = s.get_lobby_players();
                let settings = s.room_settings.clone();
                s.broadcast(ServerMessage::LobbyUpdate { players, settings });
            }
        }
        ClientMessage::StartGame => {
            let is_host = s.host_id == Some(my_id);
            if is_host && !s.in_game {
                let mut players = s.get_lobby_players();
                for p in &mut players {
                    p.lives = s.room_settings.lives;
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

                let mut new_state = GameState::new(s.room_settings.width, s.room_settings.height);
                new_state.bpm = s.room_settings.bpm;
                new_state.sudden_death = s.room_settings.sudden_death;
                new_state.bonus_every = s.room_settings.bonus_every;
                new_state.mode = s.room_settings.mode;
                new_state.spawn_players(players);

                let mut ctx = GameContext::new(s.room_settings.width, s.room_settings.height, s.room_settings.bpm);
                ctx.state = new_state;

                s.game_ctx = Some(ctx);
                s.in_game = true;

                s.broadcast(ServerMessage::GameStarted {
                    initial_state: s.game_ctx.as_ref().unwrap().state.clone(),
                });
            }
        }
        ClientMessage::Action(action) => {
            if s.in_game {
                if let Some(ref mut ctx) = s.game_ctx {
                    ctx.process_player_action(my_id, action);
                }
            }
        }
        ClientMessage::LeaveLobby => {
            return Err(());
        }
    }
    Ok(())
}

async fn disconnect_peer(my_id: u32, state: &SharedState) {
    let mut s = state.lock().await;
    s.peers.remove(&my_id);

    if s.peers.is_empty() {
        s.in_game = false;
        s.game_ctx = None;
        s.next_peer_id = 1;
        s.host_id = None;
    } else {
        if s.host_id == Some(my_id) {
            s.host_id = s.peers.keys().min().copied();
        }
        let players = s.get_lobby_players();
        let settings = s.room_settings.clone();
        s.broadcast(ServerMessage::LobbyUpdate { players, settings });
    }
}
