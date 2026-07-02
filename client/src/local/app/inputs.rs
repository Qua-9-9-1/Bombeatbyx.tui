use crate::local::app::{App, AppState};
use crate::local::settings::GaugeSkin;
use crate::screens::main_menu::MainMenuAction;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

impl App {
    pub(crate) fn handle_inputs(&mut self) -> std::io::Result<()> {
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release { continue; }

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
                }
            }
        }
        Ok(())
    }

    fn handle_main_menu_input(&mut self, code: KeyCode) {
        match self.main_menu_screen.handle_input(code) {
            MainMenuAction::HostLobby => self.state = AppState::Lobby,
            MainMenuAction::JoinLobby => {}
            MainMenuAction::Settings => self.state = AppState::SettingsMenu,
            MainMenuAction::Exit => self.game_run = false,
            MainMenuAction::None => {}
        }
    }

    fn handle_lobby_input(&mut self, code: KeyCode) {
        let is_host = self.is_local_player_host();
        if self.lobby_screen.handle_input(
            &mut self.room_settings,
            &mut self.profile.skin,
            is_host,
            code
        ) {
            self.start_game();
        }
    }

    fn handle_in_game_input(&mut self, code: KeyCode) {
        self.trigger_game_action(code);
    }

    fn handle_pause_menu_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('z') => self.pause_cursor = self.pause_cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('s') => self.pause_cursor = (self.pause_cursor + 1).min(2),
            KeyCode::Enter => match self.pause_cursor {
                0 => self.state = AppState::InGame,
                1 => self.state = AppState::SettingsMenu,
                2 => self.state = AppState::MainMenu,
                _ => {}
            }
            _ => {}
        }
    }

    fn handle_settings_menu_input(&mut self, code: KeyCode) {
        if self.editing_name {
            match code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.editing_name = false;
                }
                KeyCode::Backspace => {
                    self.profile.name.pop();
                }
                KeyCode::Char(c) => {
                    if self.profile.name.len() < 12 {
                        self.profile.name.push(c);
                    }
                }
                _ => {}
            }
        } else {
            match code {
                KeyCode::Up | KeyCode::Char('z') => self.settings_cursor = self.settings_cursor.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('s') => self.settings_cursor = (self.settings_cursor + 1).min(3),
                KeyCode::Left | KeyCode::Char('q') | KeyCode::Right | KeyCode::Char('d') => {
                    match self.settings_cursor {
                        0 => {
                            self.profile.gauge_skin = match self.profile.gauge_skin {
                                GaugeSkin::NecroDancer => GaugeSkin::Simple,
                                GaugeSkin::Undertale => GaugeSkin::NecroDancer,
                                GaugeSkin::Simple => GaugeSkin::Undertale,
                            };
                        }
                        2 => {
                            self.profile.ascii_mode = !self.profile.ascii_mode;
                        }
                        _ => {}
                    }
                }
                KeyCode::Enter => {
                    match self.settings_cursor {
                        1 => self.editing_name = true,
                        2 => self.profile.ascii_mode = !self.profile.ascii_mode,
                        3 => self.state = AppState::MainMenu,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn handle_esc(&mut self) {
        match self.state {
            AppState::InGame | AppState::Lobby => self.state = AppState::PauseMenu,
            AppState::PauseMenu => self.state = AppState::InGame,
            AppState::SettingsMenu => self.state = AppState::MainMenu,
            AppState::MainMenu => self.game_run = false,
        }
    }
}
