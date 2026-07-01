use crate::game::rhythm::{GaugeSkin, RhythmManager};
use crate::game::{controls::ControlMode, controls::ControlsManager, ui};
use crate::tui::Tui;
use common::game::{Cell, GameState};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::{Duration, Instant};

pub const CELL_W: u16 = 2;
pub const CELL_H: u16 = 1;

pub struct App {
    pub game_state: GameState,
    pub rhythm: RhythmManager,
    pub game_run: bool,
    pub controls: ControlsManager,
    pub consumed_this_beat: bool,
    pub last_feedback: &'static str,
}

impl App {
    pub fn new() -> Self {
        let mut game_state = GameState::new(15, 15);
        game_state.grid[2 * 15 + 3] = Cell::Brick;
        game_state.grid[3 * 15 + 1] = Cell::Brick;

        game_state.players.push(common::game::Player {
            id: 1,
            sub_x: 1 * CELL_W as i32,
            sub_y: 1 * CELL_H as i32,
            is_alive: true,
            max_bombs: 2,
            active_bombs: 0,
            bomb_range: 3,
        });

        Self {
            game_state,
            rhythm: RhythmManager::new(80.0, GaugeSkin::Undertale),
            game_run: true,
            controls: ControlsManager::new(),
            consumed_this_beat: false,
            last_feedback: "ATTENTE",
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        let _ = tui.init();
        let mut last_time = Instant::now();
        let tick_rate = Duration::from_millis(16);
        let mut lag = Duration::ZERO;

        while self.game_run {
            let current_time = Instant::now();
            let elapsed = current_time.duration_since(last_time);
            last_time = current_time;
            lag += elapsed;

            while event::poll(Duration::ZERO)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }

                    if self.controls.mode == ControlMode::Menu {
                        self.controls.handle_event(key);
                    } else if self.controls.mode == ControlMode::Game {
                        if key.code == KeyCode::Esc {
                            self.game_run = false;
                            continue;
                        }

                        let progress = self.rhythm.progress();
                        let on_beat = progress < 0.15 || progress > 0.85;

                        if on_beat {
                            if !self.consumed_this_beat {
                                let mut action_executed = false;
                                let p_id = 1;

                                match key.code {
                                    KeyCode::Left | KeyCode::Char('q') => {
                                        self.game_state.move_player(p_id, -2, 0);
                                        action_executed = true;
                                    }
                                    KeyCode::Right | KeyCode::Char('d') => {
                                        self.game_state.move_player(p_id, 2, 0);
                                        action_executed = true;
                                    }
                                    KeyCode::Up | KeyCode::Char('z') => {
                                        self.game_state.move_player(p_id, 0, -1);
                                        action_executed = true;
                                    }
                                    KeyCode::Down | KeyCode::Char('s') => {
                                        self.game_state.move_player(p_id, 0, 1);
                                        action_executed = true;
                                    }
                                    KeyCode::Char(' ') => {
                                        self.game_state.try_place_bomb(p_id);
                                        action_executed = true;
                                    }
                                    KeyCode::Char('e') | KeyCode::Char('E') => {
                                        self.game_state.trigger_action_2(p_id);
                                        action_executed = true;
                                    }
                                    _ => {}
                                }

                                if action_executed {
                                    self.consumed_this_beat = true;
                                    self.last_feedback = "PERFECT!";
                                }
                            }
                        } else {
                            if matches!(
                                key.code,
                                KeyCode::Left
                                    | KeyCode::Right
                                    | KeyCode::Up
                                    | KeyCode::Down
                                    | KeyCode::Char(' ')
                                    | KeyCode::Char('e')
                                    | KeyCode::Char('q')
                                    | KeyCode::Char('s')
                                    | KeyCode::Char('d')
                                    | KeyCode::Char('z')
                            ) {
                                self.last_feedback = "MISS!";
                            }
                        }
                    }
                }
            }

            while lag >= tick_rate {
                if self.rhythm.tick_logic() {
                    self.game_state.tick_bombs_and_explosions();
                    self.game_state.check_deaths();
                    self.consumed_this_beat = false;
                }
                lag -= tick_rate;
            }

            tui.draw(|frame| ui::draw(frame, self))?;
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}
