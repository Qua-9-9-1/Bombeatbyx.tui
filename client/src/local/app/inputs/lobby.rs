use crate::local::app::App;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_lobby_input(&mut self, code: KeyCode) {
        if self.editing_name {
            match code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.editing_name = false;
                    let _ = self.profile.save();
                }
                KeyCode::Backspace => {
                    self.profile.name.pop();
                    self.sync_lobby_name();
                }
                KeyCode::Char(c) => {
                    if self.profile.name.len() < 12 {
                        self.profile.name.push(c);
                        self.sync_lobby_name();
                    }
                }
                _ => {}
            }
            return;
        }

        let is_host = self.is_local_player_host();

        if is_host && !self.editing_name {
            match code {
                KeyCode::Tab => {
                    let other_players: Vec<u32> = if let Some(ref ctx) = self.game_ctx {
                        ctx.state
                            .players
                            .iter()
                            .filter(|p| p.id != self.current_player_id)
                            .map(|p| p.id)
                            .collect()
                    } else {
                        vec![]
                    };

                    if other_players.is_empty() {
                        self.lobby_screen.selected_player_id = None;
                    } else {
                        let next_sel = match self.lobby_screen.selected_player_id {
                            None => Some(other_players[0]),
                            Some(sel_id) => {
                                if let Some(pos) = other_players.iter().position(|&id| id == sel_id) {
                                    if pos + 1 < other_players.len() {
                                        Some(other_players[pos + 1])
                                    } else {
                                        None
                                    }
                                } else {
                                    Some(other_players[0])
                                }
                            }
                        };
                        self.lobby_screen.selected_player_id = next_sel;
                    }
                    return;
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    if let Some(target_id) = self.lobby_screen.selected_player_id {
                        if let Some(ref ctx) = self.game_ctx {
                            if let Some(p) = ctx.state.players.iter().find(|p| p.id == target_id) {
                                self.active_confirmation = Some(crate::local::app::ConfirmationPopup {
                                    title: "Transfer Host Rights".to_string(),
                                    message: format!("Transfer host rights to {}?", p.name),
                                    action: crate::local::app::HostAction::Transfer,
                                    target_id,
                                });
                            }
                        }
                    }
                    return;
                }
                KeyCode::Char('k') | KeyCode::Char('K') => {
                    if let Some(target_id) = self.lobby_screen.selected_player_id {
                        self.lobby_screen.selected_player_id = None;
                        if self.network.is_multiplayer {
                            if let Some(ref tx) = self.network.server_tx {
                                let _ = tx.send(ClientMessage::KickPlayer(target_id));
                            }
                        }
                    }
                    return;
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    if let Some(target_id) = self.lobby_screen.selected_player_id {
                        if let Some(ref ctx) = self.game_ctx {
                            if let Some(p) = ctx.state.players.iter().find(|p| p.id == target_id) {
                                self.active_confirmation = Some(crate::local::app::ConfirmationPopup {
                                    title: "Ban Player".to_string(),
                                    message: format!("Ban {} from the game?", p.name),
                                    action: crate::local::app::HostAction::Ban,
                                    target_id,
                                });
                            }
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        if code == KeyCode::Enter && self.lobby_screen.cursor == 7 {
            self.editing_name = true;
            return;
        }

        if self.lobby_screen.handle_input(
            &mut self.room_settings,
            &mut self.profile.skin,
            is_host,
            code,
        ) {
            let skin_taken = if let Some(ref ctx) = self.game_ctx {
                ctx.state.players.iter().any(|p| {
                    p.id != self.current_player_id && p.is_ready && p.skin == self.profile.skin
                })
            } else {
                false
            };

            if skin_taken {
                return;
            }

            if self.network.is_multiplayer {
                if let Some(ref tx) = self.network.server_tx {
                    let _ = tx.send(ClientMessage::ToggleReady);
                }
            } else {
                if let Some(ref mut ctx) = self.game_ctx {
                    if let Some(p) = ctx
                        .state
                        .players
                        .iter_mut()
                        .find(|p| p.id == self.current_player_id)
                    {
                        p.is_ready = !p.is_ready;
                    }
                }
            }
        } else {
            if self.network.is_multiplayer {
                self.sync_lobby_skin();
                let _ = self.profile.save();
                if is_host && self.lobby_screen.cursor < 7 {
                    if let Some(ref tx) = self.network.server_tx {
                        let _ = tx.send(ClientMessage::UpdateSettings(self.room_settings.clone()));
                    }
                }
            } else {
                self.sync_lobby_skin();
                let _ = self.profile.save();
            }
        }
    }

    pub(crate) fn sync_lobby_name(&mut self) {
        if self.network.is_multiplayer {
            if let Some(ref code) = self.network.room_code {
                if let Some(ref tx) = self.network.server_tx {
                    let _ = tx.send(ClientMessage::JoinRoom {
                        code: code.clone(),
                        name: self.profile.name.clone(),
                        skin: self.profile.skin.clone(),
                    });
                }
            }
        } else {
            if let Some(ref mut ctx) = self.game_ctx {
                if let Some(p) = ctx
                    .state
                    .players
                    .iter_mut()
                    .find(|p| p.id == self.current_player_id)
                {
                    p.name = self.profile.name.clone();
                }
            }
        }
    }

    pub(crate) fn sync_lobby_skin(&mut self) {
        if self.network.is_multiplayer {
            if let Some(ref code) = self.network.room_code {
                if let Some(ref tx) = self.network.server_tx {
                    let _ = tx.send(ClientMessage::JoinRoom {
                        code: code.clone(),
                        name: self.profile.name.clone(),
                        skin: self.profile.skin.clone(),
                    });
                }
            }
        } else {
            if let Some(ref mut ctx) = self.game_ctx {
                if let Some(p) = ctx
                    .state
                    .players
                    .iter_mut()
                    .find(|p| p.id == self.current_player_id)
                {
                    p.skin = self.profile.skin.clone();
                }
            }
        }
    }
}
