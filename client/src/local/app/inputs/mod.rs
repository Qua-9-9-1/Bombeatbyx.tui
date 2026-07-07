use crate::local::app::{App, AppState};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

pub mod game;
pub mod lobby;
pub mod menu;

impl App {
    pub(crate) fn handle_inputs(&mut self) -> std::io::Result<()> {
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                if self.active_notification.is_some() {
                    self.active_notification = None;
                    return Ok(());
                }

                if let Some(ref conf) = self.active_confirmation {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            let action = conf.action;
                            let target_id = conf.target_id;
                            self.execute_confirmed_host_action(action, target_id);
                            self.active_confirmation = None;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            self.active_confirmation = None;
                        }
                        _ => {}
                    }
                    return Ok(());
                }

                if key.code == KeyCode::Esc {
                    self.handle_esc();
                    return Ok(());
                }

                match self.state {
                    AppState::MainMenu => self.handle_main_menu_input(key.code),
                    AppState::Lobby => self.handle_lobby_input(key.code),
                    AppState::InGame => self.handle_in_game_input(key.code),
                    AppState::PauseMenu => self.handle_pause_menu_input(key.code),
                    AppState::SettingsMenu => self.handle_settings_menu_input(key.code),
                    AppState::HostModal => self.handle_host_modal_input(key.code),
                    AppState::JoinRoomMenu => self.handle_join_room_menu_input(key.code),
                    AppState::VictoryScreen => {
                        let elapsed = self
                            .victory_start_time
                            .map(|t| t.elapsed())
                            .unwrap_or(Duration::ZERO);
                        if elapsed >= Duration::from_millis(3000) {
                            self.state = AppState::Lobby;
                            self.victory_final_state = None;
                            self.victory_start_time = None;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn handle_esc(&mut self) {
        match self.state {
            AppState::InGame | AppState::Lobby => {
                if self.state == AppState::Lobby {
                    let my_player_ready = if let Some(ref ctx) = self.game_ctx {
                        ctx.state
                            .players
                            .iter()
                            .find(|p| p.id == self.current_player_id)
                            .map(|p| p.is_ready)
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if my_player_ready {
                        if self.network.is_multiplayer {
                            if let Some(ref tx) = self.network.server_tx {
                                let _ = tx.send(common::messages::ClientMessage::ToggleReady);
                            }
                        } else {
                            if let Some(ref mut ctx) = self.game_ctx {
                                if let Some(p) = ctx
                                    .state
                                    .players
                                    .iter_mut()
                                    .find(|p| p.id == self.current_player_id)
                                {
                                    p.is_ready = false;
                                }
                            }
                        }
                    }
                }
                self.paused_from = Some(self.state.clone());
                self.state = AppState::PauseMenu;
            }
            AppState::PauseMenu => {
                self.state = self.paused_from.take().unwrap_or(AppState::InGame);
            }
            AppState::SettingsMenu => {
                if self.capturing_key {
                    self.capturing_key = false;
                } else if self.paused_from.is_some() {
                    self.state = AppState::PauseMenu;
                } else {
                    self.state = AppState::MainMenu;
                }
            }
            AppState::MainMenu => self.game_run = false,
            AppState::HostModal => self.state = AppState::MainMenu,
            AppState::JoinRoomMenu => {
                if self.network.show_private_join_prompt {
                    self.network.show_private_join_prompt = false;
                } else {
                    self.network.is_multiplayer = false;
                    self.network.server_tx = None;
                    self.network.server_rx = None;
                    self.network.room_code = None;
                    self.state = AppState::MainMenu;
                }
            }
            AppState::VictoryScreen => {}
        }
    }
}
