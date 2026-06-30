use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use common::game::{GameState, Cell};
use crate::tui::Tui;
use crate::game::{ui, controls::ControlsManager, controls::ControlMode, physics, bombs};

pub const CELL_W: i32 = 4;
pub const CELL_H: i32 = 2;

pub struct App {
    pub game_state: GameState,
    pub should_quit: bool,
    pub controls: ControlsManager,
}

impl App {
    pub fn new() -> Self {
        let mut game_state = GameState::new(15, 15);
        game_state.grid[2 * 15 + 3] = Cell::Brick;
        game_state.grid[3 * 15 + 1] = Cell::Brick;

        game_state.players.push(common::game::Player {
            id: 1, sub_x: 1 * CELL_W, sub_y: 1 * CELL_H, is_alive: true, max_bombs: 2, active_bombs: 0, bomb_range: 3,
        });

        Self { game_state, should_quit: false, controls: ControlsManager::new() }
    }

    pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        let _ = tui.init(); 

        let mut last_time = Instant::now();
        let tick_rate = Duration::from_millis(16);
        let mut lag = Duration::ZERO;

        while !self.should_quit {
            let current_time = Instant::now();
            let elapsed = current_time.duration_since(last_time);
            last_time = current_time;
            lag += elapsed;

            while event::poll(Duration::ZERO)? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Esc && key.kind != KeyEventKind::Release && self.controls.mode == ControlMode::Game {
                        self.should_quit = true;
                    }
                    self.controls.handle_event(key);
                }
            }

            while lag >= tick_rate {
                if self.controls.mode == ControlMode::Game {
                    physics::update_player_movement(&mut self.game_state, &self.controls.state);
                    
                    if self.controls.state.space_pressed {
                        bombs::try_place_bomb(&mut self.game_state);
                        self.controls.state.space_pressed = false;
                    }
                }

                bombs::tick_bombs_and_explosions(&mut self.game_state);
                bombs::check_deaths(&mut self.game_state);
                
                self.controls.tick_input_decay();

                lag -= tick_rate;
            }

            tui.draw(|frame| ui::draw(frame, self))?;
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}