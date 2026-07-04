use crate::local::app::App;
use common::messages::{ClientMessage, ServerMessage};

impl App {
    pub fn connect_to_server(&mut self, addr: String, pending_msg: Option<ClientMessage>) {
        self.network.network_error = None;
        let mut ws_url = if addr.starts_with("ws://") || addr.starts_with("wss://") {
            addr
        } else {
            format!("ws://{}", addr)
        };
        if !ws_url.ends_with("/ws") {
            if ws_url.ends_with('/') {
                ws_url.push_str("ws");
            } else {
                ws_url.push_str("/ws");
            }
        }

        let (tx_to_app, rx_from_ws) = tokio::sync::mpsc::unbounded_channel();
        let (tx_to_ws, mut rx_from_app) = tokio::sync::mpsc::unbounded_channel::<ClientMessage>();

        self.network.server_tx = Some(tx_to_ws);
        self.network.server_rx = Some(rx_from_ws);
        self.network.is_multiplayer = true;

        tokio::spawn(async move {
            use futures_util::{SinkExt, StreamExt};
            let connect_result = tokio_tungstenite::connect_async(&ws_url).await;

            let ws_stream = match connect_result {
                Ok((stream, _)) => stream,
                Err(e) => {
                    let _ = tx_to_app.send(ServerMessage::ConnectionFailed(e.to_string()));
                    return;
                }
            };

            let (mut ws_write, mut ws_read) = ws_stream.split();

            if let Some(msg) = pending_msg {
                if let Ok(json_str) = serde_json::to_string(&msg) {
                    let _ = ws_write
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            json_str.into(),
                        ))
                        .await;
                }
            }

            loop {
                tokio::select! {
                    Some(msg_res) = ws_read.next() => {
                        match msg_res {
                            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                     if let ServerMessage::Ping = server_msg {
                                         if let Ok(json_str) = serde_json::to_string(&ClientMessage::Pong) {
                                             let _ = ws_write.send(tokio_tungstenite::tungstenite::Message::Text(json_str.into())).await;
                                         }
                                     } else if tx_to_app.send(server_msg).is_err() {
                                         break;
                                     }
                                }
                            }
                            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                let _ = tx_to_app.send(ServerMessage::GameEnded);
                                break;
                            }
                            Err(_) => {
                                let _ = tx_to_app.send(ServerMessage::GameEnded);
                                break;
                            }
                            _ => {}
                        }
                    }
                    res = rx_from_app.recv() => {
                        match res {
                            Some(client_msg) => {
                                let is_leave = matches!(client_msg, ClientMessage::LeaveLobby);
                                if let Ok(json_str) = serde_json::to_string(&client_msg) {
                                    let _ = ws_write.send(tokio_tungstenite::tungstenite::Message::Text(json_str.into())).await;
                                }
                                if is_leave {
                                    let _ = ws_write.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
                                    break;
                                }
                            }
                            None => {
                                if let Ok(json_str) = serde_json::to_string(&ClientMessage::LeaveLobby) {
                                    let _ = ws_write.send(tokio_tungstenite::tungstenite::Message::Text(json_str.into())).await;
                                }
                                let _ = ws_write.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
