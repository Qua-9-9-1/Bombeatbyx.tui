use crate::local::app::App;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_lobby_input(&mut self, code: KeyCode) {
        if self.editing_name {
            match code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.editing_name = false;
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
                if is_host && self.lobby_screen.cursor < 7 {
                    if let Some(ref tx) = self.network.server_tx {
                        let _ = tx.send(ClientMessage::UpdateSettings(self.room_settings.clone()));
                    }
                }
            } else {
                self.sync_lobby_skin();
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
