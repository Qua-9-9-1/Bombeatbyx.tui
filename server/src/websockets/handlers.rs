use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc::unbounded_channel;

use crate::state::SharedState;
use crate::websockets::rooms::{process_client_message, disconnect_peer};
use common::messages::{ClientMessage, ServerMessage};

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut ws_write, mut ws_read) = socket.split();
    let (tx, mut rx) = unbounded_channel::<ServerMessage>();

    let mut my_room_code: Option<String> = None;
    let mut my_player_id: Option<u32> = None;
    let mut my_name = "Player".to_string();
    let mut my_skin = "🤖".to_string();

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if ws_write.send(Message::Text(json_str.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    while let Some(Ok(Message::Text(text))) = ws_read.next().await {
        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
            let res = process_client_message(
                &mut my_room_code,
                &mut my_player_id,
                &mut my_name,
                &mut my_skin,
                tx.clone(),
                client_msg,
                &state,
            ).await;
            if res.is_err() {
                break;
            }
        }
    }

    write_task.abort();

    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        disconnect_peer(&code, player_id, &state).await;
    }
}
