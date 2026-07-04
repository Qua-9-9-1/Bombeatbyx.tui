use crate::state::{Peer, Room, SharedState, get_color_for_id};
use crate::websockets::utils::generate_room_code;
use common::game::{GameContext, GameState};
use common::messages::{ClientMessage, RoomInfo, ServerMessage};
use tokio::sync::mpsc::UnboundedSender;

pub async fn process_client_message(
    my_room_code: &mut Option<String>,
    my_player_id: &mut Option<u32>,
    my_name: &mut String,
    my_skin: &mut String,
    tx: UnboundedSender<ServerMessage>,
    client_msg: ClientMessage,
    state: &SharedState,
) -> Result<(), ()> {
    match client_msg {
        ClientMessage::GetRooms => {
            let s = state.lock().await;
            let mut room_infos = Vec::new();
            for room in s.rooms.values() {
                if room.is_public {
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
        }
        ClientMessage::CreateRoom { is_public, is_lan } => {
            let mut s = state.lock().await;
            let code = generate_room_code(&s.rooms);

            let mut room = Room::new(code.clone(), is_public, is_lan);
            let id = 1;
            room.host_id = Some(id);
            room.next_peer_id = 2;

            let color = get_color_for_id(id);
            let peer = Peer {
                id,
                name: my_name.clone(),
                skin: my_skin.clone(),
                color,
                tx: tx.clone(),
                is_ready: false,
                is_spectator: false,
            };
            room.peers.insert(id, peer);

            s.rooms.insert(code.clone(), room);

            *my_room_code = Some(code.clone());
            *my_player_id = Some(id);

            let r = s.rooms.get(&code).unwrap();
            let initial_lobby = GameState::new(r.room_settings.width, r.room_settings.height);
            let joined_msg = ServerMessage::Joined {
                your_id: id,
                room_code: code.clone(),
                current_state: initial_lobby,
                settings: r.room_settings.clone(),
            };
            let _ = tx.send(joined_msg);

            let players = r.get_lobby_players();
            r.broadcast(ServerMessage::LobbyUpdate {
                players,
                settings: r.room_settings.clone(),
            });
        }
        ClientMessage::JoinRoom { code, name, skin } => {
            *my_name = name.clone();
            *my_skin = skin.clone();

            let mut s = state.lock().await;
            let code_upper = code.to_uppercase();
            if let Some(room) = s.rooms.get_mut(&code_upper) {
                if let Some(player_id) = *my_player_id {
                    if let Some(peer) = room.peers.get_mut(&player_id) {
                        peer.name = name;
                        peer.skin = skin;
                        peer.is_ready = false;
                    }
                    let players = room.get_lobby_players();
                    room.broadcast(ServerMessage::LobbyUpdate {
                        players,
                        settings: room.room_settings.clone(),
                    });
                    return Ok(());
                }

                if room.peers.len() >= 8 {
                    let _ = tx.send(ServerMessage::ConnectionFailed("Room is full".to_string()));
                    return Err(());
                }

                let id = room.next_peer_id;
                room.next_peer_id += 1;

                let color = get_color_for_id(id);
                let is_spectator = room.in_game;
                let peer = Peer {
                    id,
                    name: name.clone(),
                    skin: skin.clone(),
                    color: color.clone(),
                    tx: tx.clone(),
                    is_ready: false,
                    is_spectator,
                };
                room.peers.insert(id, peer);

                *my_room_code = Some(code_upper.clone());
                *my_player_id = Some(id);

                if is_spectator {
                    if let Some(ref mut ctx) = room.game_ctx {
                        ctx.state.players.push(common::game::Player {
                            id,
                            is_host: false,
                            name: name.clone(),
                            skin: skin.clone(),
                            color: color.clone(),
                            sub_x: 0,
                            sub_y: 0,
                            is_alive: false,
                            score: 0,
                            combo: 0,
                            max_bombs: 1,
                            active_bombs: 0,
                            bomb_range: 1,
                            last_acted_beat: None,
                            last_accuracy: common::game::BeatAccuracy::Waiting,
                            last_action_time: None,
                            spam_lockout_until: None,
                            active_emote: None,
                            emote_until: None,
                            lives: 0,
                            death_pos: None,
                            respawn_timer: None,
                            collected_bonuses: Vec::new(),
                            is_spectator: true,
                            second_item: None,
                            shield_until_beat: None,
                            is_ready: false,
                        });
                    }
                }

                let current_state = if let Some(ref ctx) = room.game_ctx {
                    ctx.state.clone()
                } else {
                    GameState::new(room.room_settings.width, room.room_settings.height)
                };

                let joined_msg = ServerMessage::Joined {
                    your_id: id,
                    room_code: code_upper.clone(),
                    current_state,
                    settings: room.room_settings.clone(),
                };
                let _ = tx.send(joined_msg);

                if room.in_game {
                    let players = room.get_lobby_players();
                    let settings = room.room_settings.clone();
                    let _ = tx.send(ServerMessage::LobbyUpdate { players, settings });
                } else {
                    let players = room.get_lobby_players();
                    room.broadcast(ServerMessage::LobbyUpdate {
                        players,
                        settings: room.room_settings.clone(),
                    });
                }
            } else {
                let _ = tx.send(ServerMessage::ConnectionFailed(
                    "Room not found".to_string(),
                ));
                return Err(());
            }
        }
        ClientMessage::UpdateSettings(settings) => {
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
        }
        ClientMessage::StartGame => {
            if let Some(code) = my_room_code {
                let mut s = state.lock().await;
                if let Some(room) = s.rooms.get_mut(code) {
                    let is_host = room.host_id == *my_player_id;
                    if is_host && !room.in_game {
                        let everyone_ready =
                            !room.peers.is_empty() && room.peers.values().all(|p| p.is_ready);
                        if everyone_ready {
                            start_game_in_room(room);
                        }
                    }
                }
            }
        }
        ClientMessage::ToggleReady => {
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
                                    other.id != *player_id
                                        && other.is_ready
                                        && other.skin == original_skin
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

                            let all_ready =
                                !room.peers.is_empty() && room.peers.values().all(|p| p.is_ready);
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
        }
        ClientMessage::Action(action) => {
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
        }
        ClientMessage::LeaveLobby => {
            return Err(());
        }
        ClientMessage::Pong => {}
        ClientMessage::StopGame => {
            if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
                let mut s = state.lock().await;
                if let Some(room) = s.rooms.get_mut(code) {
                    let is_host = room.host_id == Some(*player_id);
                    if is_host && room.in_game {
                        stop_game_in_room(room);
                    }
                }
            }
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
                    stop_game_in_room(room);
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

pub fn stop_game_in_room(room: &mut Room) {
    room.in_game = false;
    room.game_ctx = None;
    for peer in room.peers.values_mut() {
        peer.is_ready = false;
        peer.is_spectator = false;
    }
    let players = room.get_lobby_players();
    let settings = room.room_settings.clone();
    room.broadcast(ServerMessage::GameStopped { players, settings });
}

fn start_game_in_room(room: &mut Room) {
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
        initial_state: room.game_ctx.as_ref().unwrap().state.clone(),
    });
}
