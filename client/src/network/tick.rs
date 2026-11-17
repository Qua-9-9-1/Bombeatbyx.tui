use crate::local::app::{App, AppState};
use common::game::models::is_valid_player_name;
use std::time::{Duration, Instant};

/// LAN announcements read per frame: a flood cannot freeze the UI.
const MAX_DATAGRAMS_PER_TICK: usize = 32;
/// Rooms kept in the LAN list: a flood cannot grow memory.
const MAX_LAN_ROOMS: usize = 32;
const ROOM_CODE_LEN: usize = 6;
const MAX_ROOM_PLAYERS: usize = 8;

/// A parsed `BOMBEATBYX_LAN_ROOM:<code>:<host>:<players>:<port>` datagram.
/// Anything malformed is rejected, including names that could carry
/// terminal control characters.
fn parse_announcement(msg: &str) -> Option<(String, String, usize, u16)> {
    let rest = msg.strip_prefix("BOMBEATBYX_LAN_ROOM:")?;
    let parts: Vec<&str> = rest.split(':').collect();
    let [code, host, count, port] = parts.as_slice() else {
        return None;
    };
    let code_ok = code.len() == ROOM_CODE_LEN
        && code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    let count: usize = count.parse().ok().filter(|c| *c <= MAX_ROOM_PLAYERS)?;
    let port: u16 = port.parse().ok().filter(|p| *p > 0)?;
    (code_ok && is_valid_player_name(host)).then(|| (code.to_string(), host.to_string(), count, port))
}

impl App {
    pub(crate) fn handle_network_tick(&mut self, udp_socket: &Option<std::net::UdpSocket>) {
        if self.state == AppState::JoinRoomMenu {
            if let Some(s) = udp_socket {
                let mut buf = [0; 1024];
                for _ in 0..MAX_DATAGRAMS_PER_TICK {
                    let Ok((amt, src)) = s.recv_from(&mut buf) else {
                        break;
                    };
                    let Some((code, host_name, count, tcp_port)) = std::str::from_utf8(&buf[..amt])
                        .ok()
                        .and_then(parse_announcement)
                    else {
                        continue;
                    };
                    let entry = (code, host_name, count, src, tcp_port, Instant::now());
                    let rooms = &mut self.network.lan_rooms;
                    if let Some(pos) = rooms.iter().position(|r| r.0 == entry.0) {
                        rooms[pos] = entry;
                    } else if rooms.len() < MAX_LAN_ROOMS {
                        rooms.push(entry);
                    }
                }
            }
            self.network
                .lan_rooms
                .retain(|r| r.5.elapsed() < Duration::from_secs(3));
        }

        if self.network.is_multiplayer {
            if let Some(mut rx) = self.network.server_rx.take() {
                while let Ok(msg) = rx.try_recv() {
                    self.handle_server_message(msg);
                }
                self.network.server_rx = Some(rx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_announcement_reads_a_valid_room() {
        let parsed = parse_announcement("BOMBEATBYX_LAN_ROOM:AB12CD:Alice:3:27301");

        assert_eq!(parsed, Some(("AB12CD".to_string(), "Alice".to_string(), 3, 27301)));
    }

    #[test]
    fn parse_announcement_rejects_malformed_datagrams() {
        let bad = [
            "BOMBEATBYX_LAN_ROOM:AB12CD:Alice:3",
            "BOMBEATBYX_LAN_ROOM:short:Alice:3:27301",
            "BOMBEATBYX_LAN_ROOM:AB12CD:\u{1b}[2J:3:27301",
            "BOMBEATBYX_LAN_ROOM:AB12CD:Alice:99:27301",
            "BOMBEATBYX_LAN_ROOM:AB12CD:Alice:3:0",
            "OTHER:AB12CD:Alice:3:27301",
        ];

        let accepted = bad.iter().filter(|m| parse_announcement(m).is_some()).count();

        assert_eq!(accepted, 0);
    }
}
