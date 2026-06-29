use std::time::Duration;
use crossterm::event::{self, Event, KeyCode};
use common::game::{GameState, Cell};
use crate::tui::Tui;
use crate::game::{ui, input::InputState, physics, bombs};

pub const CELL_W: i32 = 2;
pub const CELL_H: i32 = 1;

pub struct App {
    pub game_state: GameState,
    pub should_quit: bool,
    inputs: InputState,
    tick_count: u32,
}

impl App {
    pub fn new() -> Self {
        let mut game_state = GameState::new(15, 15);
        game_state.grid[2 * 15 + 3] = Cell::Brick;
        game_state.grid[3 * 15 + 1] = Cell::Brick;

        game_state.players.push(common::game::Player {
            id: 1, sub_x: 1 * CELL_W, sub_y: 1 * CELL_H, is_alive: true, max_bombs: 2, active_bombs: 0, bomb_range: 3,
        });

        Self { game_state, should_quit: false, inputs: InputState::default(), tick_count: 0 }
    }

    pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        while !self.should_quit {
            tui.draw(|frame| ui::draw(frame, self))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    let ttl = 6; 
                    match key.code {
                        KeyCode::Esc => self.should_quit = true,
                        KeyCode::Up | KeyCode::Char('z') => { self.inputs.up = ttl; self.inputs.down = 0; },
                        KeyCode::Down | KeyCode::Char('s') => { self.inputs.down = ttl; self.inputs.up = 0; },
                        KeyCode::Left | KeyCode::Char('q') | KeyCode::Char('a') => { self.inputs.left = ttl; self.inputs.right = 0; },
                        KeyCode::Right | KeyCode::Char('d') => { self.inputs.right = ttl; self.inputs.left = 0; },
                        KeyCode::Char(' ') => bombs::try_place_bomb(&mut self.game_state),
                        _ => {}
                    }
                }
            }

            self.tick_count = self.tick_count.wrapping_add(1);

            physics::update_player_movement(&mut self.game_state, &self.inputs, self.tick_count);
            bombs::tick_bombs_and_explosions(&mut self.game_state);
            bombs::check_deaths(&mut self.game_state);

            self.inputs.tick_decay();
        }
        Ok(())
    }
}