use crate::local::app::{App, AppState};
use crate::local::settings::GaugeSkin;
use crate::screens::main_menu::MainMenuAction;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_main_menu_input(&mut self, code: KeyCode) {
        match self.main_menu_screen.handle_input(code) {
            MainMenuAction::HostGame => {
                self.state = AppState::HostModal;
                self.host_cursor = 0;
            }
            MainMenuAction::JoinGame => {
                self.state = AppState::JoinRoomMenu;
                self.join_cursor = 0;
                self.join_filter_mode = 0;
                self.network.lan_rooms.clear();
                self.network.online_rooms.clear();
                self.network.show_private_join_prompt = false;
                self.network.private_room_code_input.clear();
                self.connect_to_server(
                    self.profile.server_addr.clone(),
                    Some(ClientMessage::GetRooms),
                );
            }
            MainMenuAction::Settings => self.state = AppState::SettingsMenu,
            MainMenuAction::Exit => self.game_run = false,
            _ => {}
        }
    }

    pub(crate) fn handle_settings_menu_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.settings_cursor = self.settings_cursor.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.settings_cursor = (self.settings_cursor + 1).min(2)
            }
            KeyCode::Left | KeyCode::Char('q') | KeyCode::Right | KeyCode::Char('d') => {
                match self.settings_cursor {
                    0 => {
                        self.profile.gauge_skin = match self.profile.gauge_skin {
                            GaugeSkin::NecroDancer => GaugeSkin::Simple,
                            GaugeSkin::Undertale => GaugeSkin::NecroDancer,
                            GaugeSkin::Simple => GaugeSkin::Undertale,
                        };
                    }
                    1 => {
                        self.profile.ascii_mode = !self.profile.ascii_mode;
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => match self.settings_cursor {
                1 => self.profile.ascii_mode = !self.profile.ascii_mode,
                2 => {
                    if self.paused_from.is_some() {
                        self.state = AppState::PauseMenu;
                    } else {
                        self.state = AppState::MainMenu;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub(crate) fn handle_host_modal_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.host_cursor = self.host_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.host_cursor = (self.host_cursor + 1).min(3);
            }
            KeyCode::Left | KeyCode::Char('q') | KeyCode::Right | KeyCode::Char('d') => {
                match self.host_cursor {
                    0 => self.host_mode = if self.host_mode == 0 { 1 } else { 0 },
                    1 => self.host_visibility = if self.host_visibility == 0 { 1 } else { 0 },
                    _ => {}
                }
            }
            KeyCode::Enter => match self.host_cursor {
                2 => {
                    let is_public = self.host_visibility == 0;
                    let is_lan = self.host_mode == 1;

                    if is_lan {
                        if let Err(e) = self.start_local_server() {
                            self.network.network_error = Some(e);
                            self.state = AppState::MainMenu;
                            return;
                        }
                        let addr = "127.0.0.1:3000".to_string();
                        self.connect_to_server(
                            addr,
                            Some(ClientMessage::CreateRoom { is_public, is_lan }),
                        );
                    } else {
                        let addr = self.profile.server_addr.clone();
                        self.connect_to_server(
                            addr,
                            Some(ClientMessage::CreateRoom { is_public, is_lan }),
                        );
                    }
                }
                3 => {
                    self.state = AppState::MainMenu;
                }
                _ => {}
            },
            KeyCode::Esc => {
                self.state = AppState::MainMenu;
            }
            _ => {}
        }
    }

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
                            "127.0.0.1:3000".to_string()
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

        let mut filtered_rooms = Vec::new();
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
                    Some(r.3),
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

                    let addr = if let Some(src) = room.5 {
                        format!("{}:3000", src.ip())
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
