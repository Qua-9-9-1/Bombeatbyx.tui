use crate::local::{
    controls::{ControlMode, ControlsManager},
    settings::{ClientSettings, GaugeSkin},
};
use crate::tui::Tui;
use crate::ui;
use common::game::{BeatAccuracy, GameAction, GameState, RhythmEngine};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::{Duration, Instant};

pub const CELL_W: u16 = 2;
pub const CELL_H: u16 = 1;

pub struct App {
    pub game_state: GameState,
    pub rhythm: RhythmEngine,
    pub game_run: bool,
    pub controls: ControlsManager,
    pub consumed_this_beat: bool,
    pub last_feedback: &'static str,
    pub settings: ClientSettings,
}

impl App {
    pub fn new() -> Self {
        let mut game_state = GameState::new(15, 15);

        game_state.players.push(common::game::Player {
            id: 1,
            sub_x: 2,
            sub_y: 2,
            is_alive: true,
            max_bombs: 3,
            active_bombs: 0,
            bomb_range: 2,
            score: 0,
        });

        Self {
            game_state,
            rhythm: RhythmEngine::new(80.0),
            game_run: true,
            controls: ControlsManager::new(),
            consumed_this_beat: false,
            last_feedback: "WAITING...",
            settings: ClientSettings {
                player_name: "Newbie".to_string(),
                key_up: 'z',
                key_down: 's',
                key_left: 'q',
                key_right: 'd',
                gauge_skin: GaugeSkin::Undertale,
            },
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        let _ = tui.init();
        let mut last_time = Instant::now();
        let tick_rate = Duration::from_millis(16);
        let mut lag = Duration::ZERO;

        while self.game_run {
            let current_time = Instant::now();
            lag += current_time.duration_since(last_time);
            last_time = current_time;

            self.handle_inputs()?;
            self.update_physics(tick_rate, &mut lag);

            tui.draw(|frame| ui::draw(frame, self))?;
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn handle_inputs(&mut self) -> std::io::Result<()> {
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                if self.controls.mode == ControlMode::Menu {
                    self.controls.handle_event(key);
                    continue;
                }

                if self.controls.mode == ControlMode::Game {
                    if key.code == KeyCode::Esc {
                        self.game_run = false;
                        return Ok(());
                    }

                    if let Some(action) = self.map_key_to_action(key.code) {
                        self.process_game_action(action);
                    }
                }
            }
        }
        Ok(())
    }

    fn map_key_to_action(&self, code: KeyCode) -> Option<GameAction> {
        match code {
            KeyCode::Left | KeyCode::Char('q') => Some(GameAction::MoveLeft),
            KeyCode::Right | KeyCode::Char('d') => Some(GameAction::MoveRight),
            KeyCode::Up | KeyCode::Char('z') => Some(GameAction::MoveUp),
            KeyCode::Down | KeyCode::Char('s') => Some(GameAction::MoveDown),
            KeyCode::Char(' ') => Some(GameAction::PlaceBomb),
            KeyCode::Char('e') => Some(GameAction::TriggerSpell),
            _ => None,
        }
    }

    fn process_game_action(&mut self, action: GameAction) {
        let accuracy = self.rhythm.evaluate_accuracy();

        if accuracy != BeatAccuracy::Miss {
            if !self.consumed_this_beat {
                self.game_state.handle_action(1, action, accuracy.clone());
                self.consumed_this_beat = true;
                self.last_feedback = accuracy.as_str();
            }
        } else {
            self.last_feedback = "MISS!";
        }
    }

    fn update_physics(&mut self, tick_rate: Duration, lag: &mut Duration) {
        while *lag >= tick_rate {
            if self.rhythm.tick_logic() {
                self.game_state.tick_bombs_and_explosions();
                self.game_state.check_deaths();
                self.consumed_this_beat = false;
            }
            *lag -= tick_rate;
        }
    }
}
