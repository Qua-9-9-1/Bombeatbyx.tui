use crate::local::app::{App, AppState};
use common::game::GameContext;
use common::messages::ServerMessage;
use std::time::Instant;

impl App {
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::RoomList(rooms) => {
                self.network.online_rooms = rooms;
            }
            ServerMessage::Joined {
                your_id,
                room_code,
                current_state,
                settings,
            } => {
                self.current_player_id = your_id;
                self.network.room_code = Some(room_code);
                self.room_settings = settings;

                let mut ctx = GameContext::new(
                    self.room_settings.width,
                    self.room_settings.height,
                    self.room_settings.bpm,
                );
                ctx.state = current_state;
                self.game_ctx = Some(ctx);
                let is_me_spectator = self
                    .game_ctx
                    .as_ref()
                    .and_then(|c| c.state.players.iter().find(|p| p.id == your_id))
                    .map(|p| p.is_spectator)
                    .unwrap_or(false);
                if is_me_spectator {
                    self.state = AppState::InGame;
                } else {
                    self.state = AppState::Lobby;
                }
            }
            ServerMessage::LobbyUpdate { players, settings } => {
                self.room_settings = settings.clone();
                if self.state != AppState::InGame && self.state != AppState::PauseMenu {
                    if let Some(ref mut ctx) = self.game_ctx {
                        ctx.state.players = players;
                        ctx.rhythm = common::game::RhythmEngine::new(settings.bpm);

                        if let Some(sel_id) = self.lobby_screen.selected_player_id {
                            if !ctx.state.players.iter().any(|p| p.id == sel_id) {
                                self.lobby_screen.selected_player_id = None;
                            }
                        }
                    }
                }
            }
            ServerMessage::GameStarted { initial_state } => {
                if let Some(ref mut ctx) = self.game_ctx {
                    ctx.state = initial_state;
                    ctx.rhythm = common::game::RhythmEngine::new(self.room_settings.bpm);
                    ctx.last_closed_window_beat = None;
                }
                self.paused_from = None;
                self.state = AppState::InGame;
            }
            ServerMessage::GameStateUpdate(new_state) => self.apply_server_state(new_state),
            ServerMessage::GameEnded => {
                self.state = AppState::MainMenu;
                self.network.is_multiplayer = false;
                self.network.server_tx = None;
                self.network.server_rx = None;
                self.network.room_code = None;
                self.paused_from = None;
                self.game_ctx = None;
                self.stop_local_server();
            }
            ServerMessage::ConnectionFailed(err) => {
                // The join screen has its own error line; elsewhere, say it right away.
                if self.state == AppState::JoinRoomMenu {
                    self.network.network_error = Some(err);
                } else {
                    self.active_notification = Some(crate::local::app::NotificationPopup {
                        title: "Connection failed".to_string(),
                        message: err,
                    });
                }
                self.network.is_multiplayer = false;
                self.network.server_tx = None;
                self.network.server_rx = None;
                self.network.room_code = None;

                match self.state {
                    AppState::JoinRoomMenu | AppState::HostModal => {}
                    _ => {
                        self.state = AppState::MainMenu;
                        self.paused_from = None;
                        self.game_ctx = None;
                        self.stop_local_server();
                    }
                }
            }
            ServerMessage::Ping { rtt_ms } => {
                self.network.rtt = std::time::Duration::from_millis(rtt_ms as u64);
            }
            ServerMessage::GameStopped {
                players,
                settings,
                victory_state,
            } => {
                self.room_settings = settings;
                let mut ctx = GameContext::new(
                    self.room_settings.width,
                    self.room_settings.height,
                    self.room_settings.bpm,
                );
                ctx.state.players = players;
                self.game_ctx = Some(ctx);
                self.paused_from = None;

                if let Some(v_state) = victory_state {
                    self.victory_final_state = Some(v_state);
                    self.victory_start_time = Some(Instant::now());
                    self.state = AppState::VictoryScreen;
                } else {
                    self.state = AppState::Lobby;
                }
            }
            ServerMessage::HostTransferred {
                new_host_id,
                new_host_name,
            } => {
                if new_host_id == self.current_player_id {
                    self.active_notification = Some(crate::local::app::NotificationPopup {
                        title: "Host Promotion".to_string(),
                        message: "You are now the host of this lobby!".to_string(),
                    });
                } else {
                    self.active_notification = Some(crate::local::app::NotificationPopup {
                        title: "Host Transferred".to_string(),
                        message: format!("{} is now the host.", new_host_name),
                    });
                }
            }
            ServerMessage::PlayerKicked {
                player_id,
                player_name,
            } => {
                if player_id == self.current_player_id {
                    self.state = AppState::MainMenu;
                    self.network.is_multiplayer = false;
                    self.network.server_tx = None;
                    self.network.server_rx = None;
                    self.network.room_code = None;
                    self.paused_from = None;
                    self.game_ctx = None;
                    self.stop_local_server();

                    self.active_notification = Some(crate::local::app::NotificationPopup {
                        title: "Kicked".to_string(),
                        message: "You have been kicked from the lobby.".to_string(),
                    });
                } else {
                    self.active_notification = Some(crate::local::app::NotificationPopup {
                        title: "Kicked".to_string(),
                        message: format!("{} has been kicked from the lobby.", player_name),
                    });
                }
            }
            ServerMessage::PlayerBanned {
                player_id,
                player_name,
            } => {
                if player_id == self.current_player_id {
                    self.state = AppState::MainMenu;
                    self.network.is_multiplayer = false;
                    self.network.server_tx = None;
                    self.network.server_rx = None;
                    self.network.room_code = None;
                    self.paused_from = None;
                    self.game_ctx = None;
                    self.stop_local_server();

                    self.active_notification = Some(crate::local::app::NotificationPopup {
                        title: "Banned".to_string(),
                        message: "You have been banned from the lobby.".to_string(),
                    });
                } else {
                    self.active_notification = Some(crate::local::app::NotificationPopup {
                        title: "Banned".to_string(),
                        message: format!("{} has been banned from the lobby.", player_name),
                    });
                }
            }
        }
    }
}
