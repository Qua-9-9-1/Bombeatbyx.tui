use crate::state::{Peer, Room, SharedState};
use crate::websockets::utils::generate_room_code;
use common::game::GameState;
use common::messages::ServerMessage;
use tokio::sync::mpsc::UnboundedSender;

pub async fn handle_create_room(
    my_room_code: &mut Option<String>,
    my_player_id: &mut Option<u32>,
    my_name: &str,
    my_skin: &str,
    tx: &UnboundedSender<ServerMessage>,
    state: &SharedState,
    client_ip: std::net::IpAddr,
    is_public: bool,
    is_lan: bool,
) -> Result<(), ()> {
    if !common::game::models::is_valid_player_name(my_name) {
        let _ = tx.send(ServerMessage::ConnectionFailed(
            "Invalid player name. Only alphanumeric characters, spaces, hyphens, and underscores are allowed (max 16 chars).".to_string(),
        ));
        return Err(());
    }

    if !common::game::models::ALL_SKINS.contains(&my_skin) {
        let _ = tx.send(ServerMessage::ConnectionFailed(
            "Invalid player skin".to_string(),
        ));
        return Err(());
    }

    let mut s = state.lock().await;
    if s.rooms.len() >= 1000 {
        let _ = tx.send(ServerMessage::ConnectionFailed(
            "Server is full (maximum room limit reached)".to_string(),
        ));
        return Err(());
    }
    let code = generate_room_code(&s.rooms);
    let mut room = Room::new(code.clone(), is_public, is_lan);
    let id = 1;
    room.host_id = Some(id);
    room.next_peer_id = 2;

    let peer = Peer {
        id,
        name: my_name.to_string(),
        skin: my_skin.to_string(),
        tx: tx.clone(),
        is_ready: false,
        is_spectator: false,
        ip: client_ip,
    };
    room.peers.insert(id, peer);
    s.rooms.insert(code.clone(), room);

    *my_room_code = Some(code.clone());
    *my_player_id = Some(id);

    let r = s
        .rooms
        .get(&code)
        .expect("room should exist after creation");
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
    Ok(())
}

pub async fn handle_join_room(
    my_room_code: &mut Option<String>,
    my_player_id: &mut Option<u32>,
    my_name: &mut String,
    my_skin: &mut String,
    tx: &UnboundedSender<ServerMessage>,
    state: &SharedState,
    client_ip: std::net::IpAddr,
    code: String,
    name: String,
    skin: String,
) -> Result<(), ()> {
    if !common::game::models::is_valid_player_name(&name) {
        let _ = tx.send(ServerMessage::ConnectionFailed(
            "Invalid player name. Only alphanumeric characters, spaces, hyphens, and underscores are allowed (max 16 chars).".to_string(),
        ));
        return Err(());
    }

    if !common::game::models::ALL_SKINS.contains(&skin.as_str()) {
        let _ = tx.send(ServerMessage::ConnectionFailed(
            "Invalid player skin".to_string(),
        ));
        return Err(());
    }

    *my_name = name.clone();
    *my_skin = skin.clone();
 
    let mut s = state.lock().await;
    let code_upper = code.to_uppercase();
    if let Some(room) = s.rooms.get_mut(&code_upper) {
        if room.banned_ips.contains(&client_ip) {
            let _ = tx.send(ServerMessage::ConnectionFailed(
                "You are banned from this room".to_string(),
            ));
            return Err(());
        }

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

        let is_spectator = room.in_game;
        let peer = Peer {
            id,
            name: name.clone(),
            skin: skin.clone(),
            tx: tx.clone(),
            is_ready: false,
            is_spectator,
            ip: client_ip,
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
                    death_beat: None,
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

        let players = room.get_lobby_players();
        let settings = room.room_settings.clone();
        if room.in_game {
            let _ = tx.send(ServerMessage::LobbyUpdate { players, settings });
        } else {
            room.broadcast(ServerMessage::LobbyUpdate { players, settings });
        }
    } else {
        let _ = tx.send(ServerMessage::ConnectionFailed(
            "Room not found".to_string(),
        ));
        return Err(());
    }
    Ok(())
}
