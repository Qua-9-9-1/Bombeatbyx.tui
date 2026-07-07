mod game;
mod inputs;
mod local_server;

use crate::local::settings::ClientSettings;
use crate::screens::{lobby::LobbyScreen, main_menu::MainMenuScreen};
use crate::tui::Tui;
use crate::ui;
use common::game::{BeatAccuracy, GameContext, Player, RoomSettings};
use std::time::{Duration, Instant};

pub const CELL_W: u16 = 2;
pub const CELL_H: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostAction {
    Transfer,
    Ban,
}

#[derive(Debug, Clone)]
pub struct ConfirmationPopup {
    pub title: String,
    pub message: String,
    pub action: HostAction,
    pub target_id: u32,
}

#[derive(Debug, Clone)]
pub struct NotificationPopup {
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    MainMenu,
    Lobby,
    SettingsMenu,
    InGame,
    PauseMenu,
    HostModal,
    JoinRoomMenu,
    VictoryScreen,
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
    pub editing_name: bool,

    pub last_local_action: Option<(u64, common::game::GameAction)>,

    pub network: crate::network::NetworkContext,

    pub server_process: Option<std::process::Child>,
    pub host_cursor: usize,
    pub host_mode: usize,
    pub host_visibility: usize,
    pub join_cursor: usize,
    pub join_filter_mode: usize,
    pub paused_from: Option<AppState>,
    pub capturing_key: bool,

    pub active_confirmation: Option<ConfirmationPopup>,
    pub active_notification: Option<NotificationPopup>,
    pub is_local_dev_bots: bool,
    pub victory_final_state: Option<common::game::GameState>,
    pub victory_start_time: Option<Instant>,
}

impl App {
    pub fn new() -> Self {
        let current_player_id = 1;
        let profile = ClientSettings::load();
        let room_settings = profile.last_room_settings.clone();

        let mut ctx =
            GameContext::new(room_settings.width, room_settings.height, room_settings.bpm);

        ctx.state.players = vec![Player {
            id: 1,
            is_host: true,
            name: profile.name.clone(),
            skin: profile.skin.clone(),
            sub_x: 0,
            sub_y: 0,
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
            active_emote: None,
            emote_until: None,
            lives: room_settings.lives,
            death_pos: None,
            respawn_timer: None,
            collected_bonuses: Vec::new(),
            is_spectator: false,
            second_item: None,
            shield_until_beat: None,
            is_ready: false,
            death_beat: None,
        }];

        Self {
            state: AppState::MainMenu,
            profile,
            game_ctx: Some(ctx),
            current_player_id,
            game_run: true,
            main_menu_screen: MainMenuScreen::new(),
            lobby_screen: LobbyScreen::new(),
            room_settings,
            pause_cursor: 0,
            settings_cursor: 0,
            editing_name: false,
            last_local_action: None,

            network: crate::network::NetworkContext::new(),

            server_process: None,
            host_cursor: 0,
            host_mode: 0,
            host_visibility: 0,
            join_cursor: 0,
            join_filter_mode: 0,
            paused_from: None,
            capturing_key: false,

            active_confirmation: None,
            active_notification: None,
            is_local_dev_bots: false,
            victory_final_state: None,
            victory_start_time: None,
        }
    }

    pub async fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        let _ = tui.init();
        let mut last_time = Instant::now();
        let mut last_render = Instant::now();
        let render_rate = Duration::from_millis(16);
        let tick_rate = Duration::from_millis(16);
        let mut lag = Duration::ZERO;
        let mut prev_state = self.state.clone();

        let udp_socket = std::net::UdpSocket::bind("0.0.0.0:27315").ok();
        if let Some(ref s) = udp_socket {
            let _ = s.set_nonblocking(true);
        }

        while self.game_run {
            let current_time = Instant::now();
            lag += current_time.duration_since(last_time);
            last_time = current_time;

            self.handle_network_tick(&udp_socket);

            if self.state == AppState::InGame {
                if !self.network.is_multiplayer {
                    let mut has_beat_ticked = false;
                    let mut should_end = false;
                    if let Some(ref mut ctx) = self.game_ctx {
                        has_beat_ticked = ctx.tick_game_logic();

                        if let Some(limit_mins) = self.room_settings.time_limit_mins {
                            if ctx.state.elapsed_time_secs >= limit_mins * 60 {
                                should_end = true;
                            }
                        }

                        if self.room_settings.mode == common::game::models::GameMode::Score {
                            if ctx
                                .state
                                .players
                                .iter()
                                .any(|p| p.score >= self.room_settings.target_score)
                            {
                                should_end = true;
                            }
                        } else {
                            let total_players = ctx.state.players.len();
                            let alive_count =
                                ctx.state.players.iter().filter(|p| p.lives > 0).count();
                            if total_players > 1 && alive_count <= 1 {
                                should_end = true;
                            } else if total_players <= 1 && alive_count == 0 {
                                should_end = true;
                            }
                        }
                    }
                    if has_beat_ticked && self.is_local_dev_bots {
                        self.update_bots();
                    }
                    if should_end && self.game_ctx.is_some() {
                        let final_state = self.game_ctx.as_ref().unwrap().state.clone();
                        let ctx = common::game::GameContext::new(
                            self.room_settings.width,
                            self.room_settings.height,
                            self.room_settings.bpm,
                        );
                        self.game_ctx = Some(ctx);
                        self.setup_local_lobby_players();
                        self.victory_final_state = Some(final_state);
                        self.victory_start_time = Some(Instant::now());
                        self.state = AppState::VictoryScreen;
                    }
                } else {
                    if let Some(ref mut ctx) = self.game_ctx {
                        ctx.rhythm.tick_logic();
                    }
                }
            }
            self.handle_inputs()?;
            if self.state == AppState::InGame && !self.network.is_multiplayer {
                self.update_physics(tick_rate, &mut lag);
            }

            if current_time.duration_since(last_render) >= render_rate {
                if self.state != prev_state {
                    let _ = tui.clear();
                    prev_state = self.state.clone();
                }
                tui.draw(|frame| ui::draw(frame, self))?;
                last_render = current_time;
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        Ok(())
    }

    pub fn execute_confirmed_host_action(&mut self, action: HostAction, target_id: u32) {
        self.lobby_screen.selected_player_id = None;
        if self.network.is_multiplayer {
            if let Some(ref tx) = self.network.server_tx {
                match action {
                    HostAction::Transfer => {
                        let _ = tx.send(common::messages::ClientMessage::TransferHost(target_id));
                    }
                    HostAction::Ban => {
                        let _ = tx.send(common::messages::ClientMessage::BanPlayer(target_id));
                    }
                }
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_local_server();
    }
}
