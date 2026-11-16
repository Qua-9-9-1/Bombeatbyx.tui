use crate::local::app::{App, AppState};
use crate::screens::lobby::ROW_READY;
use crate::screens::main_menu::MainMenuAction;
use crossterm::event::KeyCode;

impl App {
    pub(crate) fn handle_main_menu_input(&mut self, code: KeyCode) {
        match self.main_menu_screen.handle_input(code) {
            MainMenuAction::HostGame => {
                self.is_solo_mode = false;
                self.state = AppState::HostModal;
                self.host_cursor = 0;
            }
            MainMenuAction::SoloGame => {
                self.is_solo_mode = true;
                self.network.is_multiplayer = false;
                self.state = AppState::Lobby;
                self.lobby_screen.cursor = ROW_READY;
                let ctx = common::game::GameContext::new(
                    self.room_settings.width,
                    self.room_settings.height,
                    self.room_settings.bpm,
                );
                self.game_ctx = Some(ctx);
                self.setup_local_lobby_players();
            }
            MainMenuAction::JoinGame => {
                self.is_solo_mode = false;
                self.state = AppState::JoinRoomMenu;
                self.join_cursor = 0;
                self.join_filter_mode = 0;
                self.network.lan_rooms.clear();
                self.network.online_rooms.clear();
                self.network.network_error = None;
                self.network.show_private_join_prompt = false;
                self.network.private_room_code_input.clear();
            }
            MainMenuAction::Settings => self.state = AppState::SettingsMenu,
            MainMenuAction::Exit => self.game_run = false,
            _ => {}
        }
    }
}
