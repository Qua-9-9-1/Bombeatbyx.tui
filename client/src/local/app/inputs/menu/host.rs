use crate::local::app::{App, AppState};
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
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

                    self.profile.last_room_settings = self.room_settings.clone();
                    let _ = self.profile.save();

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
}
