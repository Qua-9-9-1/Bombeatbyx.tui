use axum::{Router, routing::get};
use std::sync::Arc;

mod state;
mod websockets;

use crate::state::ServerState;
use crate::websockets::rooms::stop_game_in_room;
use common::messages::ServerMessage;

#[tokio::main]
async fn main() {
    let state = Arc::new(tokio::sync::Mutex::new(ServerState::new()));

    let app = Router::new()
        .route("/ws", get(websockets::ws_handler))
        .route("/", get(|| async { "Bombeat TUI Server is running!" }))
        .with_state(state.clone());

    let loop_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(16));
        loop {
            interval.tick().await;

            let mut updates = Vec::new();
            {
                let mut s = loop_state.lock().await;
                for room in s.rooms.values_mut() {
                    if room.in_game {
                        if let Some(ref mut ctx) = room.game_ctx {
                            ctx.tick_game_logic();

                            let active_non_spec =
                                room.peers.values().filter(|p| !p.is_spectator).count();
                            let alive_count = ctx
                                .state
                                .players
                                .iter()
                                .filter(|p| p.lives > 0 && !p.is_spectator)
                                .count();
                            if active_non_spec > 1 && alive_count <= 1 {
                                stop_game_in_room(room);
                                continue;
                            }

                            updates.push((room.code.clone(), ctx.state.clone()));
                        }
                    }
                }
            }

            if !updates.is_empty() {
                let s = loop_state.lock().await;
                for (code, state) in updates {
                    if let Some(room) = s.rooms.get(&code) {
                        room.broadcast(ServerMessage::GameStateUpdate(state));
                    }
                }
            }
        }
    });

    let udp_state = state.clone();
    tokio::spawn(async move {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok();
        if let Some(socket) = socket {
            let _ = socket.set_broadcast(true);
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let mut broadcast_rooms = Vec::new();
                {
                    let s = udp_state.lock().await;
                    for room in s.rooms.values() {
                        if room.is_lan && room.is_public {
                            let host_name = room
                                .peers
                                .values()
                                .find(|p| Some(p.id) == room.host_id)
                                .map(|p| p.name.clone())
                                .unwrap_or_else(|| "Unknown".to_string());
                            broadcast_rooms.push((room.code.clone(), host_name, room.peers.len()));
                        }
                    }
                }

                for (code, host, count) in broadcast_rooms {
                    let msg = format!("BOMBEAT_LAN_ROOM:{}:{}:{}", code, host, count);
                    let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:3001");
                }
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind TcpListener");
    println!("Listening on ws://localhost:3000/ws");
    axum::serve(listener, app)
        .await
        .expect("failed to serve axum app");
}
