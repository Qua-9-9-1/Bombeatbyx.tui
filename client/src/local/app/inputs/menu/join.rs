use crate::local::app::App;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_join_room_menu_input(&mut self, code: KeyCode) {
        if self.network.show_private_join_prompt {
            match code {
                KeyCode::Esc => {
                    self.network.show_private_join_prompt = false;
                }
                KeyCode::Enter => {
                    if !self.network.private_room_code_input.is_empty() {
                        let code_to_join = self.network.private_room_code_input.clone();
                        self.network.show_private_join_prompt = false;

                        let addr = if self.join_filter_mode == 2 {
                            format!("127.0.0.1:{}", self.network.local_server_port.unwrap_or(27300))
                        } else {
                            self.profile.server_addr.clone()
                        };

                        self.connect_to_server(
                            addr,
                            Some(ClientMessage::JoinRoom {
                                code: code_to_join,
                                name: self.profile.name.clone(),
                                skin: self.profile.skin.clone(),
                            }),
                        );
                    }
                }
                KeyCode::Backspace => {
                    self.network.private_room_code_input.pop();
                }
                KeyCode::Char(c) => {
                    if self.network.private_room_code_input.len() < 8 && c.is_alphanumeric() {
                        self.network
                            .private_room_code_input
                            .push(c.to_ascii_uppercase());
                    }
                }
                _ => {}
            }
            return;
        }

        let mut filtered_rooms: Vec<(String, String, usize, String, &str, Option<(std::net::SocketAddr, u16)>)> = Vec::new();
        if self.join_filter_mode == 0 {
            for r in &self.network.online_rooms {
                filtered_rooms.push((
                    r.code.clone(),
                    r.host_name.clone(),
                    r.player_count,
                    "Online".to_string(),
                    if r.is_public { "Public" } else { "Private" },
                    None,
                ));
            }
        } else {
            for r in &self.network.lan_rooms {
                filtered_rooms.push((
                    r.0.clone(),
                    r.1.clone(),
                    r.2,
                    "LAN".to_string(),
                    "Public",
                    Some((r.3, r.4)),
                ));
            }
        }

        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.join_cursor = self.join_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if !filtered_rooms.is_empty() {
                    self.join_cursor = (self.join_cursor + 1).min(filtered_rooms.len() - 1);
                }
            }
            KeyCode::Left | KeyCode::Char('q') => {
                self.join_filter_mode = if self.join_filter_mode == 0 { 1 } else { 0 };
                self.join_cursor = 0;
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.join_filter_mode = if self.join_filter_mode == 0 { 1 } else { 0 };
                self.join_cursor = 0;
            }
            KeyCode::Enter => {
                if !filtered_rooms.is_empty() && self.join_cursor < filtered_rooms.len() {
                    let room = &filtered_rooms[self.join_cursor];
                    let code_to_join = room.0.clone();

                    let addr = if let Some((src, tcp_port)) = room.5 {
                        format!("{}:{}", src.ip(), tcp_port)
                    } else {
                        self.profile.server_addr.clone()
                    };

                    self.connect_to_server(
                        addr,
                        Some(ClientMessage::JoinRoom {
                            code: code_to_join,
                            name: self.profile.name.clone(),
                            skin: self.profile.skin.clone(),
                        }),
                    );
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.network.show_private_join_prompt = true;
                self.network.private_room_code_input.clear();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.network.lan_rooms.clear();
                self.network.online_rooms.clear();
                self.connect_to_server(
                    self.profile.server_addr.clone(),
                    Some(ClientMessage::GetRooms),
                );
            }
            _ => {}
        }
    }
}
