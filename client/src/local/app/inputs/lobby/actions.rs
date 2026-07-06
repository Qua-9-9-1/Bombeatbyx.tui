use crate::local::app::App;
use common::messages::ClientMessage;
use crossterm::event::KeyCode;

impl App {
    pub(super) fn handle_lobby_host_actions(&mut self, code: KeyCode) -> bool {
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
                                if let Some(pos) = other_players.iter().position(|&id| id == sel_id)
                                {
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
                    return true;
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    if let Some(target_id) = self.lobby_screen.selected_player_id {
                        if let Some(ref ctx) = self.game_ctx {
                            if let Some(p) = ctx.state.players.iter().find(|p| p.id == target_id) {
                                self.active_confirmation =
                                    Some(crate::local::app::ConfirmationPopup {
                                        title: "Transfer Host Rights".to_string(),
                                        message: format!("Transfer host rights to {}?", p.name),
                                        action: crate::local::app::HostAction::Transfer,
                                        target_id,
                                    });
                            }
                        }
                    }
                    return true;
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
                    return true;
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    if let Some(target_id) = self.lobby_screen.selected_player_id {
                        if let Some(ref ctx) = self.game_ctx {
                            if let Some(p) = ctx.state.players.iter().find(|p| p.id == target_id) {
                                self.active_confirmation =
                                    Some(crate::local::app::ConfirmationPopup {
                                        title: "Ban Player".to_string(),
                                        message: format!("Ban {} from the game?", p.name),
                                        action: crate::local::app::HostAction::Ban,
                                        target_id,
                                    });
                            }
                        }
                    }
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}
