use crate::local::app::{App, AppState};
use std::time::{Duration, Instant};

impl App {
    pub(crate) fn handle_network_tick(&mut self, udp_socket: &Option<std::net::UdpSocket>) {
        if self.state == AppState::JoinRoomMenu {
            if let Some(s) = udp_socket {
                let mut buf = [0; 1024];
                while let Ok((amt, src)) = s.recv_from(&mut buf) {
                    if let Ok(msg) = std::str::from_utf8(&buf[..amt]) {
                        if msg.starts_with("BOMBEATBYX_LAN_ROOM:") {
                            let parts: Vec<&str> = msg.split(':').collect();
                            if parts.len() == 5 {
                                let code = parts[1].to_string();
                                let host_name = parts[2].to_string();
                                let count: usize = parts[3].parse().unwrap_or(1);
                                let tcp_port: u16 = parts[4].parse().unwrap_or(27300);
                                if let Some(pos) =
                                    self.network.lan_rooms.iter().position(|r| r.0 == code)
                                {
                                    self.network.lan_rooms[pos] =
                                        (code, host_name, count, src, tcp_port, Instant::now());
                                } else {
                                    self.network.lan_rooms.push((
                                        code,
                                        host_name,
                                        count,
                                        src,
                                        tcp_port,
                                        Instant::now(),
                                    ));
                                }
                            }
                        }
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
