use crate::local::settings::{ClientSettings, GaugeSkin};
use crate::tui::Tui;
use crate::ui;
use common::game::{BeatAccuracy, GameAction, GameContext, Player, RoomSettings};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::{Duration, Instant};
use crate::screens::{lobby::LobbyScreen, main_menu::{MainMenuScreen, MainMenuAction}};

pub const CELL_W: u16 = 2;
pub const CELL_H: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    MainMenu,
    Lobby,
    SettingsMenu,
    InGame,
    PauseMenu,
}

pub struct App {
    pub state: AppState,
    pub game_ctx: Option<GameContext>,
    pub current_player_id: u32,
    pub game_run: bool,
    pub profile: ClientSettings,
    pub room_settings: RoomSettings,

    pub main_menu_screen: MainMenuScreen,
    pub lobby_screen: LobbyScreen,

    pub pause_cursor: usize,
    pub settings_cursor: usize,
}

impl App {
    pub fn new() -> Self {
        let current_player_id = 1;
        let profile = ClientSettings::default();
        let room_settings = RoomSettings::default();

        Self {
            state: AppState::MainMenu,
            profile,
            game_ctx: None,
            current_player_id,
            game_run: true,
            main_menu_screen: MainMenuScreen::new(),
            lobby_screen: LobbyScreen::new(),
            room_settings,
            pause_cursor: 0,
            settings_cursor: 0,
        }
    }

    pub fn start_game(&mut self) {
        let mut ctx = GameContext::new(
            self.room_settings.width,
            self.room_settings.height,
            self.room_settings.bpm,
        );

        ctx.state.players.push(Player {
            id: self.current_player_id,
            is_host: true,
            name: self.profile.name.clone(),
            skin: self.profile.skin.clone(),
            color: "cyan".to_string(),
            sub_x: 2,
            sub_y: 2,
            is_alive: true,
            score: 0,
            combo: 0,
            last_acted_beat: None,
            last_accuracy: BeatAccuracy::Waiting,
            max_bombs: 1,
            active_bombs: 0,
            bomb_range: 1,
            last_action_time: None,
            spam_lockout_until: None,
        });

        self.game_ctx = Some(ctx);
        self.state = AppState::InGame;
    }

    pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        let _ = tui.init();
        let mut last_time = Instant::now();
        let mut last_render = Instant::now();
        let render_rate = Duration::from_millis(16);
        let tick_rate = Duration::from_millis(16);
        let mut lag = Duration::ZERO;

        while self.game_run {
            let current_time = Instant::now();
            lag += current_time.duration_since(last_time);
            last_time = current_time;

            if self.state == AppState::InGame {
                if let Some(ref mut ctx) = self.game_ctx {
                    ctx.tick_game_logic();
                }
            }
            self.handle_inputs()?;
            if self.state == AppState::InGame {
                self.update_physics(tick_rate, &mut lag);
            }

            if current_time.duration_since(last_render) >= render_rate {
                tui.draw(|frame| ui::draw(frame, self))?;
                last_render = current_time;
            }

            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn handle_inputs(&mut self) -> std::io::Result<()> {
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release { continue; }

                if key.code == KeyCode::Esc {
                    self.handle_esc();
                    return Ok(());
                }

                match self.state {
                    AppState::MainMenu => {
                        match self.main_menu_screen.handle_input(key.code) {
                            MainMenuAction::HostLobby => self.state = AppState::Lobby,
                            MainMenuAction::JoinLobby => {} // placeholder
                            MainMenuAction::Settings => self.state = AppState::SettingsMenu,
                            MainMenuAction::Exit => self.game_run = false,
                            MainMenuAction::None => {}
                        }
                    }
                    AppState::Lobby => {
                        let is_host = self.is_local_player_host();
                        if self.lobby_screen.handle_input(
                            &mut self.room_settings,
                            &mut self.profile.skin,
                            is_host,
                            key.code
                        ) {
                            self.start_game();
                        }
                    }
                    AppState::InGame => self.trigger_game_action(key.code),
                    AppState::PauseMenu => {
                        match key.code {
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
                    AppState::SettingsMenu => {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('z') => self.settings_cursor = self.settings_cursor.saturating_sub(1),
                            KeyCode::Down | KeyCode::Char('s') => self.settings_cursor = (self.settings_cursor + 1).min(2),
                            KeyCode::Left | KeyCode::Char('q') => {
                                match self.settings_cursor {
                                    0 => {
                                        self.profile.gauge_skin = match self.profile.gauge_skin {
                                            GaugeSkin::NecroDancer => GaugeSkin::Simple,
                                            GaugeSkin::Undertale => GaugeSkin::NecroDancer,
                                            GaugeSkin::Simple => GaugeSkin::Undertale,
                                        };
                                    }
                                    1 => {
                                        let names = vec!["Bomber", "Beast", "ProPlayer", "Newbie", "RhythmGod"];
                                        let curr = names.iter().position(|&n| n == self.profile.name).unwrap_or(0);
                                        let next = (curr + names.len() - 1) % names.len();
                                        self.profile.name = names[next].to_string();
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Right | KeyCode::Char('d') => {
                                match self.settings_cursor {
                                    0 => {
                                        self.profile.gauge_skin = match self.profile.gauge_skin {
                                            GaugeSkin::NecroDancer => GaugeSkin::Undertale,
                                            GaugeSkin::Undertale => GaugeSkin::Simple,
                                            GaugeSkin::Simple => GaugeSkin::NecroDancer,
                                        };
                                    }
                                    1 => {
                                        let names = vec!["Bomber", "Beast", "ProPlayer", "Newbie", "RhythmGod"];
                                        let curr = names.iter().position(|&n| n == self.profile.name).unwrap_or(0);
                                        let next = (curr + 1) % names.len();
                                        self.profile.name = names[next].to_string();
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Enter => {
                                self.state = AppState::MainMenu;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_esc(&mut self) {
        match self.state {
            AppState::InGame | AppState::Lobby => self.state = AppState::PauseMenu,
            AppState::PauseMenu => self.state = AppState::InGame,
            AppState::SettingsMenu => self.state = AppState::MainMenu,
            AppState::MainMenu => self.game_run = false,
        }
    }

    fn update_physics(&mut self, tick_rate: Duration, lag: &mut Duration) {
        while *lag >= tick_rate {
            *lag -= tick_rate;
        }
    }

    #[allow(dead_code)]
    pub fn get_feedback_and_combo(&self) -> (&'static str, u32) {
        if let Some(ref ctx) = self.game_ctx {
            if let Some(player) = ctx.state.players.iter().find(|p| p.id == 1) {
                if player.last_acted_beat == Some(ctx.rhythm.beat_count) {
                    return (player.last_accuracy.as_str(), player.combo);
                }
                return ("WAITING...", player.combo);
            }
        }
        ("WAITING...", 0)
    }

    pub fn is_local_player_host(&self) -> bool {
        if let Some(ref ctx) = self.game_ctx {
            if let Some(player) = ctx.state.players.iter().find(|p| p.id == self.current_player_id) {
                return player.is_host;
            }
        }
        true
    }

    pub fn trigger_game_action(&mut self, code: KeyCode) {
        if let Some(action) = self.map_key_to_action(code) {
            if let Some(ref mut ctx) = self.game_ctx {
                ctx.process_player_action(self.current_player_id, action);
            }
        }
    }

    fn map_key_to_action(&self, code: KeyCode) -> Option<GameAction> {
        if code == KeyCode::Left || code == KeyCode::Char(self.profile.key_left) {
            Some(GameAction::MoveLeft)
        } else if code == KeyCode::Right || code == KeyCode::Char(self.profile.key_right) {
            Some(GameAction::MoveRight)
        } else if code == KeyCode::Up || code == KeyCode::Char(self.profile.key_up) {
            Some(GameAction::MoveUp)
        } else if code == KeyCode::Down || code == KeyCode::Char(self.profile.key_down) {
            Some(GameAction::MoveDown)
        } else if code == KeyCode::Char(' ') {
            Some(GameAction::PlaceBomb)
        } else if code == KeyCode::Char('e') {
            Some(GameAction::TriggerSpell)
        } else {
            None
        }
    }
}
