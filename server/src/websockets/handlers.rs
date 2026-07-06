use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc::unbounded_channel;

use crate::state::SharedState;
use crate::websockets::rooms::{disconnect_peer, process_client_message};
use common::messages::{ClientMessage, ServerMessage};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, addr.ip()))
}

pub async fn handle_socket(socket: WebSocket, state: SharedState, client_ip: std::net::IpAddr) {
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

    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
    let mut unanswered_pings = 0;

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                unanswered_pings += 1;
                if unanswered_pings > 3 {
                    break;
                }
                let _ = tx.send(ServerMessage::Ping);
            }
            msg_res = ws_read.next() => {
                match msg_res {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            if let ClientMessage::Pong = client_msg {
                                unanswered_pings = 0;
                            } else {
                                let res = process_client_message(
                                    &mut my_room_code,
                                    &mut my_player_id,
                                    &mut my_name,
                                    &mut my_skin,
                                    tx.clone(),
                                    client_msg,
                                    &state,
                                    client_ip,
                                ).await;
                                if res.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }

    write_task.abort();

    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        disconnect_peer(&code, player_id, &state).await;
    }
}
