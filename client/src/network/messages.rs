use crate::local::app::{App, AppState};
use common::game::GameContext;
use common::messages::ServerMessage;

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
            ServerMessage::GameStateUpdate(mut new_state) => {
                if let Some(ref mut ctx) = self.game_ctx {
                    ctx.rhythm.beat_count = new_state
                        .players
                        .iter()
                        .find(|p| p.id == self.current_player_id)
                        .and_then(|p| p.last_acted_beat)
                        .unwrap_or(ctx.rhythm.beat_count);

                    let mut updated_players = new_state.players.clone();
                    if let Some(p) = updated_players
                        .iter_mut()
                        .find(|p| p.id == self.current_player_id)
                    {
                        if let Some(local_p) = ctx.state.players.iter().find(|lp| lp.id == p.id) {
                            if local_p.last_acted_beat > p.last_acted_beat {
                                p.sub_x = local_p.sub_x;
                                p.sub_y = local_p.sub_y;
                                p.last_acted_beat = local_p.last_acted_beat;
                                p.last_accuracy = local_p.last_accuracy.clone();
                                p.combo = local_p.combo;
                                p.score = local_p.score;
                                p.active_bombs = local_p.active_bombs;
                                p.max_bombs = local_p.max_bombs;
                                p.bomb_range = local_p.bomb_range;
                                p.collected_bonuses = local_p.collected_bonuses.clone();
                                p.second_item = local_p.second_item;
                                p.shield_until_beat = local_p.shield_until_beat;

                                if let Some((predicted_beat, common::game::GameAction::PlaceBomb)) =
                                    self.last_local_action
                                {
                                    if Some(predicted_beat) == local_p.last_acted_beat {
                                        let grid_x = (local_p.sub_x / 2) as usize;
                                        let grid_y = (local_p.sub_y / 1) as usize;
                                        let idx = grid_y * new_state.width + grid_x;
                                        if idx < new_state.grid.len()
                                            && new_state.grid[idx] == common::game::Cell::Empty
                                        {
                                            new_state.grid[idx] = common::game::Cell::Bomb {
                                                owner_id: self.current_player_id,
                                                ticks_left: 4,
                                            };
                                        }
                                    }
                                }
                            }
                            p.last_action_time = local_p.last_action_time;
                            p.spam_lockout_until = local_p.spam_lockout_until;
                            p.emote_until = local_p.emote_until;
                            p.respawn_timer = local_p.respawn_timer;
                        }
                    }
                    ctx.state = new_state;
                    ctx.state.players = updated_players;
                }
            }
            ServerMessage::GameEnded => {
                self.state = AppState::MainMenu;
                self.network.is_multiplayer = false;
                self.network.server_tx = None;
                self.network.server_rx = None;
                self.network.room_code = None;
                self.paused_from = None;
                self.stop_local_server();
            }
            ServerMessage::ConnectionFailed(err) => {
                self.network.network_error = Some(err);
                self.state = AppState::MainMenu;
                self.network.is_multiplayer = false;
                self.network.server_tx = None;
                self.network.server_rx = None;
                self.network.room_code = None;
                self.paused_from = None;
                self.stop_local_server();
            }
            ServerMessage::Ping => {}
            ServerMessage::GameStopped { players, settings } => {
                self.room_settings = settings;
                if let Some(ref mut ctx) = self.game_ctx {
                    ctx.state.players = players;
                    ctx.rhythm = common::game::RhythmEngine::new(self.room_settings.bpm);
                }
                self.paused_from = None;
                self.state = AppState::Lobby;
            }
            ServerMessage::HostTransferred { new_host_id, new_host_name } => {
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
            ServerMessage::PlayerKicked { player_id, player_name } => {
                if player_id == self.current_player_id {
                    self.state = AppState::MainMenu;
                    self.network.is_multiplayer = false;
                    self.network.server_tx = None;
                    self.network.server_rx = None;
                    self.network.room_code = None;
                    self.paused_from = None;
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
            ServerMessage::PlayerBanned { player_id, player_name } => {
                if player_id == self.current_player_id {
                    self.state = AppState::MainMenu;
                    self.network.is_multiplayer = false;
                    self.network.server_tx = None;
                    self.network.server_rx = None;
                    self.network.room_code = None;
                    self.paused_from = None;
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
