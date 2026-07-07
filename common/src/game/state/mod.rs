pub mod actions;
pub mod spawns;

use crate::game::models::{Cell, GameMode, Player};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub width: usize,
    pub height: usize,
    pub host_player_id: Option<u32>,
    pub bpm: f64,
    pub sudden_death: bool,
    pub bonus_every: u32,
    pub grid: Vec<Cell>,
    pub players: Vec<Player>,
    pub mode: GameMode,
    pub elapsed_time_secs: u32,
    pub target_score: u32,
    pub time_limit_mins: Option<u32>,
}

impl GameState {
    pub fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![Cell::Empty; width * height];
        for y in 0..height {
            for x in 0..width {
                if x == 0
                    || x == width - 1
                    || y == 0
                    || y == height - 1
                    || (x % 2 == 0 && y % 2 == 0)
                {
                    grid[y * width + x] = Cell::Wall;
                }
            }
        }
        Self {
            width,
            height,
            host_player_id: None,
            bpm: 60.0,
            sudden_death: false,
            bonus_every: 10,
            grid,
            players: Vec::new(),
            mode: GameMode::Deathmatch,
            elapsed_time_secs: 0,
            target_score: 1000,
            time_limit_mins: None,
        }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Cell {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return Cell::Wall;
        }
        self.grid[(y as usize) * self.width + (x as usize)]
    }

    pub fn tick_beat(&mut self, beat_count: u64) {
        if self.bonus_every > 0 && beat_count > 0 && beat_count % (self.bonus_every as u64) == 0 {
            self.spawn_random_bonus();
        }
    }
}
