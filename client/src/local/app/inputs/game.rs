use crate::local::app::{App, AppState};
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_in_game_input(&mut self, code: KeyCode) {
        self.trigger_game_action(code);
    }

    pub(crate) fn handle_pause_menu_input(&mut self, code: KeyCode) {
        let is_host_ingame = self.network.is_multiplayer
            && self.is_local_player_host()
            && self.paused_from == Some(AppState::InGame);
        let max_idx = if is_host_ingame { 3 } else { 2 };

        match code {
            KeyCode::Up | KeyCode::Char('z') => {
                self.pause_cursor = self.pause_cursor.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.pause_cursor = (self.pause_cursor + 1).min(max_idx)
            }
            KeyCode::Enter => {
                if is_host_ingame {
                    match self.pause_cursor {
                        0 => self.state = self.paused_from.take().unwrap_or(AppState::InGame),
                        1 => self.state = AppState::SettingsMenu,
                        2 => {
                            if let Some(ref tx) = self.network.server_tx {
                                let _ = tx.send(ClientMessage::StopGame);
                            }
                            self.paused_from = None;
                        }
                        3 => {
                            if let Some(ref tx) = self.network.server_tx {
                                let _ = tx.send(ClientMessage::LeaveLobby);
                            }
                            self.network.is_multiplayer = false;
                            self.network.server_tx = None;
                            self.network.server_rx = None;
                            self.network.room_code = None;
                            self.stop_local_server();
                            self.state = AppState::MainMenu;
                        }
                        _ => {}
                    }
                } else {
                    match self.pause_cursor {
                        0 => self.state = self.paused_from.take().unwrap_or(AppState::InGame),
                        1 => self.state = AppState::SettingsMenu,
                        2 => {
                            if self.network.is_multiplayer {
                                if let Some(ref tx) = self.network.server_tx {
                                    let _ = tx.send(ClientMessage::LeaveLobby);
                                }
                                self.network.is_multiplayer = false;
                                self.network.server_tx = None;
                                self.network.server_rx = None;
                                self.network.room_code = None;
                                self.stop_local_server();
                            }
                            self.state = AppState::MainMenu;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
