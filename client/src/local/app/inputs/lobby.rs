use crate::local::app::App;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_lobby_input(&mut self, code: KeyCode) {
        let is_host = self.is_local_player_host();
        if self.lobby_screen.handle_input(
            &mut self.room_settings,
            &mut self.profile.skin,
            is_host,
            code,
        ) {
            if self.network.is_multiplayer {
                if let Some(ref tx) = self.network.server_tx {
                    let _ = tx.send(ClientMessage::StartGame);
                }
            } else {
                self.start_game();
            }
        } else {
            if self.network.is_multiplayer {
                self.sync_lobby_skin();
                if is_host {
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
