use crate::local::app::App;
use common::messages::{ClientMessage, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_SERVER_MESSAGE_BYTES: usize = 1024 * 1024;
/// The server pings every 3 s: this much silence means the link is dead.
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(12);

fn to_ws_url(addr: String) -> String {
    let mut ws_url = if addr.starts_with("ws://") || addr.starts_with("wss://") {
        addr
    } else {
        format!("ws://{}", addr)
    };
    if !ws_url.ends_with("/ws") {
        if !ws_url.ends_with('/') {
            ws_url.push('/');
        }
        ws_url.push_str("ws");
    }
    ws_url
}

/// Turns low-level connection errors into an actionable message.
fn describe_connect_error(error: &tokio_tungstenite::tungstenite::Error, ws_url: &str) -> String {
    use tokio_tungstenite::tungstenite::Error;
    match error {
        Error::Io(io) if io.kind() == std::io::ErrorKind::ConnectionRefused => format!(
            "No server is running at {}. Start a dedicated server, set its address in Settings > Server, or host in LAN mode.",
            ws_url
        ),
        other => format!("Could not reach {}: {}", ws_url, other),
    }
}

fn encode(msg: &ClientMessage) -> Option<Message> {
    serde_json::to_string(msg)
        .ok()
        .map(|json| Message::Text(json.into()))
}

impl App {
    pub fn connect_to_server(&mut self, addr: String, pending_msg: Option<ClientMessage>) {
        self.network.network_error = None;
        let ws_url = to_ws_url(addr);

        let (tx_to_app, rx_from_ws) = tokio::sync::mpsc::unbounded_channel();
        let (tx_to_ws, rx_from_app) = tokio::sync::mpsc::unbounded_channel::<ClientMessage>();

        self.network.server_tx = Some(tx_to_ws);
        self.network.server_rx = Some(rx_from_ws);
        self.network.is_multiplayer = true;

        tokio::spawn(run_connection(ws_url, pending_msg, tx_to_app, rx_from_app));
    }
}

async fn run_connection(
    ws_url: String,
    pending_msg: Option<ClientMessage>,
    tx_to_app: UnboundedSender<ServerMessage>,
    mut rx_from_app: UnboundedReceiver<ClientMessage>,
) {
    // A server snapshot is a few tens of KiB: refuse anything absurd instead of
    // letting a malicious server make the client allocate 64 MiB messages.
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_SERVER_MESSAGE_BYTES))
        .max_frame_size(Some(MAX_SERVER_MESSAGE_BYTES));
    let connect = tokio::time::timeout(
        CONNECT_TIMEOUT,
        // Nagle disabled: small game messages must not wait to be batched.
        tokio_tungstenite::connect_async_with_config(&ws_url, Some(config), true),
    );
    let ws_stream = match connect.await {
        Ok(Ok((stream, _))) => stream,
        Ok(Err(e)) => {
            let _ = tx_to_app.send(ServerMessage::ConnectionFailed(describe_connect_error(&e, &ws_url)));
            return;
        }
        Err(_) => {
            let _ = tx_to_app.send(ServerMessage::ConnectionFailed(format!(
                "Server did not answer within {} s ({})",
                CONNECT_TIMEOUT.as_secs(),
                ws_url
            )));
            return;
        }
    };

    let (mut ws_write, mut ws_read) = ws_stream.split();
    if let Some(frame) = pending_msg.as_ref().and_then(encode) {
        let _ = ws_write.send(frame).await;
    }

    let lost = |reason: &str| ServerMessage::ConnectionFailed(format!("Connection lost: {}", reason));

    loop {
        tokio::select! {
            incoming = tokio::time::timeout(INACTIVITY_TIMEOUT, ws_read.next()) => {
                let frame = match incoming {
                    Err(_) => {
                        let _ = tx_to_app.send(lost("server stopped responding"));
                        break;
                    }
                    Ok(None) | Ok(Some(Ok(Message::Close(_)))) => {
                        let _ = tx_to_app.send(lost("server closed the connection"));
                        break;
                    }
                    Ok(Some(Err(e))) => {
                        let _ = tx_to_app.send(lost(&e.to_string()));
                        break;
                    }
                    Ok(Some(Ok(frame))) => frame,
                };
                let Message::Text(text) = frame else {
                    continue;
                };
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(server_msg) => {
                        // Answer heartbeats immediately (RTT accuracy), then let
                        // the app read the measured RTT they carry.
                        if matches!(server_msg, ServerMessage::Ping { .. }) {
                            if let Some(pong) = encode(&ClientMessage::Pong) {
                                let _ = ws_write.send(pong).await;
                            }
                        }
                        if tx_to_app.send(server_msg).is_err() {
                            break;
                        }
                    }
                    Err(_) => {}
                }
            }
            outgoing = rx_from_app.recv() => {
                // `None` = the app dropped the connection: leave politely.
                let client_msg = outgoing.unwrap_or(ClientMessage::LeaveLobby);
                let is_leave = matches!(client_msg, ClientMessage::LeaveLobby);
                if let Some(frame) = encode(&client_msg) {
                    let _ = ws_write.send(frame).await;
                }
                if is_leave {
                    let _ = ws_write.send(Message::Close(None)).await;
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_ws_url_adds_scheme_and_path() {
        let url = to_ws_url("127.0.0.1:27300".to_string());

        assert_eq!(url, "ws://127.0.0.1:27300/ws");
    }

    #[test]
    fn to_ws_url_keeps_existing_scheme_and_path() {
        let url = to_ws_url("wss://play.example.com/ws".to_string());

        assert_eq!(url, "wss://play.example.com/ws");
    }
}
