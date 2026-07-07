use crate::local::app::App;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(super) fn handle_lobby_typing(&mut self, code: KeyCode) -> bool {
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
            return true;
        }
        false
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
                let old_skin = if let Some(p) = ctx.state.players.iter().find(|p| p.id == self.current_player_id) {
                    p.skin.clone()
                } else {
                    self.profile.skin.clone()
                };

                let new_skin = self.profile.skin.clone();

                if self.is_local_dev_bots {
                    if let Some(bot) = ctx
                        .state
                        .players
                        .iter_mut()
                        .find(|p| p.id != self.current_player_id && p.skin == new_skin)
                    {
                        bot.skin = old_skin;
                    }
                }

                if let Some(p) = ctx
                    .state
                    .players
                    .iter_mut()
                    .find(|p| p.id == self.current_player_id)
                {
                    p.skin = new_skin;
                }
            }
        }
    }
}
