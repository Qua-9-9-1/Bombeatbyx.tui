use axum::{Router, routing::get};
use std::sync::Arc;

mod game_loop;
mod state;
mod websockets;

use crate::state::ServerState;

#[tokio::main]
async fn main() {
    let state = Arc::new(tokio::sync::Mutex::new(ServerState::new()));

    let app = Router::new()
        .route("/ws", get(websockets::ws_handler))
        .route("/", get(|| async { "Bombeatbyx TUI Server is running!" }))
        .with_state(state.clone());

    tokio::spawn(game_loop::run(state.clone()));

    let args: Vec<String> = std::env::args().collect();
    let mut port = 27300;
    if args.len() > 1 {
        if let Ok(p) = args[1].parse::<u16>() {
            port = p;
        } else if args.len() > 2 && args[1] == "--port" {
            if let Ok(p) = args[2].parse::<u16>() {
                port = p;
            }
        }
    }

    let listener = loop {
        match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(l) => break (l, port),
            Err(e) => {
                println!("Port {} is occupied, trying next...", port);
                port += 1;
                if port > 27310 {
                    panic!("Could not bind TcpListener on ports 27300-27310: {}", e);
                }
            }
        }
    };
    let (listener, bound_port) = listener;
    println!("Listening on ws://localhost:{}/ws", bound_port);

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
                    let msg = format!("BOMBEATBYX_LAN_ROOM:{}:{}:{}:{}", code, host, count, bound_port);
                    let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:27315");
                }
            }
        }
    });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("failed to serve axum app");
}
