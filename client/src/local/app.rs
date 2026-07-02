mod inputs;
mod game;

use crate::local::settings::ClientSettings;
use crate::tui::Tui;
use crate::ui;
use common::game::{GameContext, RoomSettings, Player, BeatAccuracy};
use std::time::{Duration, Instant};
use crate::screens::{lobby::LobbyScreen, main_menu::MainMenuScreen};

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
    pub editing_name: bool,
}

impl App {
    pub fn new() -> Self {
        let current_player_id = 1;
        let profile = ClientSettings::default();
        let room_settings = RoomSettings::default();

        let mut ctx = GameContext::new(
            room_settings.width,
            room_settings.height,
            room_settings.bpm,
        );

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
            },
            Player {
                id: 2,
                is_host: false,
                name: "GigaPlayer".to_string(),
                skin: "🐱".to_string(),
                color: "magenta".to_string(),
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
            },
            Player {
                id: 3,
                is_host: false,
                name: "Ribbit".to_string(),
                skin: "🐸".to_string(),
                color: "yellow".to_string(),
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
            },
            Player {
                id: 4,
                is_host: false,
                name: "Chad".to_string(),
                skin: "😎".to_string(),
                color: "blue".to_string(),
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
            },
            Player {
                id: 5,
                is_host: false,
                name: "NoobMaster69".to_string(),
                skin: "👾".to_string(),
                color: "red".to_string(),
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
            },
            Player {
                id: 6,
                is_host: false,
                name: "NoLifeGuy".to_string(),
                skin: "🧴".to_string(),
                color: "cyan".to_string(),
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
            },
            Player {
                id: 7,
                is_host: false,
                name: "PixelPanda".to_string(),
                skin: "🐼".to_string(),
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
            },
            Player {
                id: 8,
                is_host: false,
                name: "Aqua".to_string(),
                skin: "💧".to_string(),
                color: "cyan".to_string(),
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
        }
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
}
