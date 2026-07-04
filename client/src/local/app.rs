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

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    MainMenu,
    Lobby,
    SettingsMenu,
    InGame,
    PauseMenu,
    HostModal,
    JoinRoomMenu,
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
}

impl App {
    pub fn new() -> Self {
        let current_player_id = 1;
        let profile = ClientSettings::default();
        let room_settings = RoomSettings::default();

        let mut ctx =
            GameContext::new(room_settings.width, room_settings.height, room_settings.bpm);

        ctx.state.players = vec![
            Player {
                id: 1,
                is_host: true,
                name: profile.name.clone(),
                skin: profile.skin.clone(),
                color: "green".to_string(),
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
            },
        ];

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

        let udp_socket = std::net::UdpSocket::bind("0.0.0.0:3001").ok();
        if let Some(ref s) = udp_socket {
            let _ = s.set_nonblocking(true);
        }

        while self.game_run {
            let current_time = Instant::now();
            lag += current_time.duration_since(last_time);
            last_time = current_time;

            if self.state == AppState::JoinRoomMenu {
                if let Some(ref s) = udp_socket {
                    let mut buf = [0; 1024];
                    while let Ok((amt, src)) = s.recv_from(&mut buf) {
                        if let Ok(msg) = std::str::from_utf8(&buf[..amt]) {
                            if msg.starts_with("BOMBEAT_LAN_ROOM:") {
                                let parts: Vec<&str> = msg.split(':').collect();
                                if parts.len() == 4 {
                                    let code = parts[1].to_string();
                                    let host_name = parts[2].to_string();
                                    let count: usize = parts[3].parse().unwrap_or(1);
                                    if let Some(pos) = self.network.lan_rooms.iter().position(|r| r.0 == code) {
                                        self.network.lan_rooms[pos] = (code, host_name, count, src, Instant::now());
                                    } else {
                                        self.network.lan_rooms.push((code, host_name, count, src, Instant::now()));
                                    }
                                }
                            }
                        }
                    }
                }
                self.network.lan_rooms.retain(|r| r.4.elapsed() < Duration::from_secs(3));
            }

            if self.network.is_multiplayer {
                if let Some(mut rx) = self.network.server_rx.take() {
                    while let Ok(msg) = rx.try_recv() {
                        self.handle_server_message(msg);
                    }
                    self.network.server_rx = Some(rx);
                }
            }

            if self.state == AppState::InGame {
                if !self.network.is_multiplayer {
                    if let Some(ref mut ctx) = self.game_ctx {
                        ctx.tick_game_logic();
                        let alive_count = ctx.state.players.iter().filter(|p| p.lives > 0).count();
                        if alive_count == 0 {
                            self.state = AppState::Lobby;
                        }
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
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_local_server();
    }
}
