use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

mod state;
mod ws;

use crate::state::ServerState;
use common::messages::ServerMessage;

#[tokio::main]
async fn main() {
    let state = Arc::new(tokio::sync::Mutex::new(ServerState::new()));

    let app = Router::new()
        .route("/ws", get(ws::ws_handler))
        .route("/", get(|| async { "Bombeat TUI Server is running!" }))
        .with_state(state.clone());

    let loop_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(16));
        loop {
            interval.tick().await;
            let mut state_update = None;
            {
                let mut s = loop_state.lock().await;
                if s.in_game {
                    if let Some(ref mut ctx) = s.game_ctx {
                        ctx.tick_game_logic();
                        state_update = Some(ctx.state.clone());
                    }
                }
            }
            if let Some(state) = state_update {
                let s = loop_state.lock().await;
                s.broadcast(ServerMessage::GameStateUpdate(state));
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on ws://localhost:3000/ws");
    axum::serve(listener, app).await.unwrap();
}
