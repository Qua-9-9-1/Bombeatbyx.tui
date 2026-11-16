use crate::local::app::App;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

const MAX_NAME_CHARS: usize = 12;
const FALLBACK_NAME: &str = "Player";

impl App {
    pub(super) fn handle_lobby_typing(&mut self, code: KeyCode) -> bool {
        if self.editing_name {
            match code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.editing_name = false;
                    if !common::game::models::is_valid_player_name(&self.profile.name) {
                        self.profile.name = FALLBACK_NAME.to_string();
                        self.sync_lobby_name();
                    }
                    let _ = self.profile.save();
                }
                KeyCode::Backspace => {
                    self.profile.name.pop();
                    self.sync_lobby_name();
                }
                KeyCode::Char(c) => {
                    let is_allowed = c.is_alphanumeric() || c == ' ' || c == '-' || c == '_';
                    if is_allowed && self.profile.name.chars().count() < MAX_NAME_CHARS {
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
        // Intermediate names (e.g. empty while retyping) are never sent.
        if !common::game::models::is_valid_player_name(&self.profile.name) {
            return;
        }
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
                let old_skin = if let Some(p) = ctx
                    .state
                    .players
                    .iter()
                    .find(|p| p.id == self.current_player_id)
                {
                    p.skin.clone()
                } else {
                    self.profile.skin.clone()
                };

                let new_skin = self.profile.skin.clone();

                if self.is_solo_mode {
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
