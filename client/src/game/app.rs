use common::game::{GameState, Player};
use crossterm::event::{self, Event};
use std::time::Duration;

use crate::game::{controls, controls::Action, ui};
use crate::tui::Tui;

pub struct App {
    pub game_state: GameState,
    pub is_running: bool,
}

impl App {
    pub fn new() -> Self {
        let mut game_state = GameState::new(15, 15);
        game_state.players.push(Player {
            id: 1,
            x: 1,
            y: 1,
            is_alive: true,
            max_bombs: 1,
            active_bombs: 0,
            bomb_range: 2,
        });

        Self {
            game_state,
            is_running: true,
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        while self.is_running {
            tui.draw(|frame| ui::draw(frame, self))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    let action = controls::handle_key(key);
                    self.update(action);
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.is_running = false,
            Action::MoveUp => {
                if self.game_state.players[0].y > 0 {
                    self.game_state.players[0].y -= 1;
                }
            }
            Action::MoveDown => {
                if self.game_state.players[0].y < self.game_state.height - 1 {
                    self.game_state.players[0].y += 1;
                }
            }
            Action::MoveLeft => {
                if self.game_state.players[0].x > 0 {
                    self.game_state.players[0].x -= 1;
                }
            }
            Action::MoveRight => {
                if self.game_state.players[0].x < self.game_state.width - 1 {
                    self.game_state.players[0].x += 1;
                }
            }
            Action::None => {}
        }
    }
}
