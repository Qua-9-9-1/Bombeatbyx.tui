use crate::local::
    settings::{ClientSettings, GaugeSkin};

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
    pub is_paused: bool,
    pub consumed_this_beat: bool,
    pub last_feedback: &'static str,
    pub settings: ClientSettings,
    pub last_physical_input: Instant,
    pub spam_lockout_until: Instant,
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
            last_acted_beat: None,
            last_action_time: None,
            spam_lockout_until: None,
        });

        Self {
            game_state,
            rhythm: RhythmEngine::new(80.0),
            game_run: true,
            is_paused: false,
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
            last_physical_input: Instant::now() - Duration::from_secs(10),
            spam_lockout_until: Instant::now() - Duration::from_secs(10),
        }   }

pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        let _ = tui.init();
        let mut last_time = Instant::now();
        let mut last_render = Instant::now();
        let tick_rate = Duration::from_millis(16);
        let render_rate = Duration::from_millis(16);
        let mut lag = Duration::ZERO;

        while self.game_run {
            let current_time = Instant::now();
            lag += current_time.duration_since(last_time);
            last_time = current_time;

            self.handle_inputs()?;
            self.update_physics(tick_rate, &mut lag);

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
                    
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }

                    let now = Instant::now();
                    let delay = now.duration_since(self.last_physical_input);
                    self.last_physical_input = now;

                    if delay < Duration::from_millis(100) {
                        self.spam_lockout_until = now + Duration::from_millis(300);
                    }

                    if now < self.spam_lockout_until {
                        continue;
                    }

                    if key.code == KeyCode::Esc {
                        self.game_run = false;
                        return Ok(());
                    }

                    if let Some(action) = self.map_key_to_action(key.code) {
                        self.process_game_action(action);
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
        
        let current_beat = self.rhythm.beat_count; 

        self.game_state.handle_action(1, action, accuracy.clone(), current_beat);
        
        if accuracy != BeatAccuracy::Miss {
            if let Some(player) = self.game_state.players.iter().find(|p| p.id == 1) {
                if player.last_acted_beat == Some(current_beat) && !self.consumed_this_beat {
                    self.last_feedback = accuracy.as_str();
                    self.consumed_this_beat = true;
                }
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
