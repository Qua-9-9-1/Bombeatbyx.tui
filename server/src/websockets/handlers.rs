use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};

use crate::state::{SharedState, peer_channel};
use crate::websockets::rooms::{disconnect_peer, process_client_message};
use common::messages::{ClientMessage, ServerMessage};
use std::time::{Duration, Instant};

const PING_INTERVAL: Duration = Duration::from_secs(3);
const MAX_UNANSWERED_PINGS: u32 = 3;
/// Client messages are tiny JSON enums: anything bigger is refused unread.
const MAX_CLIENT_MESSAGE_BYTES: usize = 16 * 1024;
/// Messages accepted per connection and per second; extra ones are dropped.
const MAX_MESSAGES_PER_SECOND: u32 = 60;

/// Fixed one-second window message counter.
struct RateLimiter {
    window_start: Instant,
    count: u32,
}

impl RateLimiter {
    fn new() -> Self {
        Self { window_start: Instant::now(), count: 0 }
    }

    fn allow(&mut self, now: Instant) -> bool {
        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.count = 0;
        }
        self.count += 1;
        self.count <= MAX_MESSAGES_PER_SECOND
    }
}
/// Weight of a new RTT sample in the moving average (1/4).
const RTT_SAMPLE_WEIGHT_DIVISOR: u32 = 4;

/// Exponential moving average so a single slow packet doesn't skew timing.
pub fn smooth_rtt(previous: Option<Duration>, sample: Duration) -> Duration {
    match previous {
        None => sample,
        Some(prev) => {
            prev - prev / RTT_SAMPLE_WEIGHT_DIVISOR + sample / RTT_SAMPLE_WEIGHT_DIVISOR
        }
    }
}

async fn record_latency(
    state: &SharedState,
    room_code: &Option<String>,
    player_id: Option<u32>,
    rtt: Duration,
) {
    let (Some(code), Some(id)) = (room_code, player_id) else {
        return;
    };
    let mut s = state.lock().await;
    if let Some(room) = s.rooms.get_mut(code) {
        room.latencies.insert(id, rtt);
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> axum::response::Response {
    let ip = addr.ip();
    if !state.lock().await.try_open_connection(ip) {
        return (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Too many connections from this address",
        )
            .into_response();
    }
    ws.max_message_size(MAX_CLIENT_MESSAGE_BYTES)
        .max_frame_size(MAX_CLIENT_MESSAGE_BYTES)
        .on_failed_upgrade({
            let state = state.clone();
            move |_| {
                tokio::spawn(async move { state.lock().await.close_connection(ip) });
            }
        })
        .on_upgrade(move |socket| async move {
            handle_socket(socket, state.clone(), ip).await;
            state.lock().await.close_connection(ip);
        })
        .into_response()
}

pub async fn handle_socket(socket: WebSocket, state: SharedState, client_ip: std::net::IpAddr) {
    let (mut ws_write, mut ws_read) = socket.split();
    let (tx, mut rx) = peer_channel();
    let mut rate_limiter = RateLimiter::new();

    let mut my_room_code: Option<String> = None;
    let mut my_player_id: Option<u32> = None;
    let mut my_name = "Player".to_string();
    let mut my_skin = common::game::models::ALL_SKINS[0].to_string();

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if ws_write.send(Message::Text(json_str.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    let mut unanswered_pings = 0;
    let mut ping_sent_at: Option<Instant> = None;
    let mut rtt: Option<Duration> = None;

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                unanswered_pings += 1;
                // A client that stopped reading would make its outbox grow forever.
                if unanswered_pings > MAX_UNANSWERED_PINGS || tx.is_saturated() {
                    break;
                }
                let rtt_ms = rtt.map_or(0, |d| d.as_millis().min(u32::MAX as u128) as u32);
                ping_sent_at = Some(Instant::now());
                let _ = tx.send(ServerMessage::Ping { rtt_ms });
            }
            msg_res = ws_read.next() => {
                match msg_res {
                    Some(Ok(Message::Text(text))) => {
                        if !rate_limiter.allow(Instant::now()) {
                            continue;
                        }
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            if let ClientMessage::Pong = client_msg {
                                // Only a Pong answering an outstanding Ping keeps the link alive.
                                if let Some(sent_at) = ping_sent_at.take() {
                                    unanswered_pings = 0;
                                    let smoothed = smooth_rtt(rtt, sent_at.elapsed());
                                    rtt = Some(smoothed);
                                    record_latency(&state, &my_room_code, my_player_id, smoothed).await;
                                }
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
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => {
                        break;
                    }
                    // Protocol-level ping/pong and binary frames are not ours to act on.
                    Some(Ok(_)) => {}
                }
            }
        }
    }

    write_task.abort();

    if let (Some(code), Some(player_id)) = (my_room_code, my_player_id) {
        disconnect_peer(&code, player_id, &state).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_drops_messages_over_the_limit_until_next_window() {
        let mut limiter = RateLimiter::new();
        let start = limiter.window_start;
        for _ in 0..MAX_MESSAGES_PER_SECOND {
            assert!(limiter.allow(start));
        }

        let flooded = limiter.allow(start);
        let next_window = limiter.allow(start + Duration::from_secs(1));

        assert!(!flooded);
        assert!(next_window);
    }

    #[test]
    fn smooth_rtt_uses_first_sample_directly() {
        let rtt = smooth_rtt(None, Duration::from_millis(80));

        assert_eq!(rtt, Duration::from_millis(80));
    }

    #[test]
    fn smooth_rtt_dampens_a_single_spike() {
        let rtt = smooth_rtt(Some(Duration::from_millis(40)), Duration::from_millis(440));

        assert_eq!(rtt, Duration::from_millis(140));
    }
}
