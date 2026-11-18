pub mod actions;
pub mod combat;
pub mod match_flow;
pub mod spawns;
pub mod sudden_death;

use crate::game::models::{BonusType, Cell, GameMode, Player};
use crate::game::spawns::GameRng;
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
    pub countdown: Option<u32>,
    pub game_over_countdown: Option<u32>,
    #[serde(default)]
    pub current_beat: u64,
    #[serde(default)]
    pub sudden_death_progress: usize,
    /// Bonuses hidden in destroyed bricks, revealed when the fire on their cell dies out.
    #[serde(skip)]
    pub pending_drops: Vec<(usize, BonusType)>,
    #[serde(skip)]
    pub rng: GameRng,
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
            countdown: None,
            game_over_countdown: None,
            current_beat: 0,
            sudden_death_progress: 0,
            pending_drops: Vec::new(),
            rng: GameRng::from_time(),
        }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Cell {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return Cell::Wall;
        }
        self.grid[(y as usize) * self.width + (x as usize)]
    }

    /// `true` when the state can be rendered and simulated safely. Clients use it
    /// to reject malformed snapshots from a buggy or malicious server.
    pub fn is_well_formed(&self) -> bool {
        let sizes = crate::game::rules::MAP_SIZE_RANGE;
        sizes.contains(&self.width)
            && sizes.contains(&self.height)
            && self.grid.len() == self.width * self.height
    }

    pub fn tick_beat(&mut self, beat_count: u64) {
        if self.bonus_every > 0 && beat_count > 0 && beat_count % (self.bonus_every as u64) == 0 {
            self.spawn_random_bonus();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_well_formed_rejects_grid_size_mismatch_and_out_of_range_maps() {
        let valid = GameState::new(9, 9);
        let mut short_grid = GameState::new(9, 9);
        short_grid.grid.pop();
        let mut huge = GameState::new(9, 9);
        huge.width = 5000;

        assert!(valid.is_well_formed());
        assert!(!short_grid.is_well_formed());
        assert!(!huge.is_well_formed());
    }

    #[test]
    fn new_game_state_generates_boundary_walls() {
        let state = GameState::new(5, 5);

        for x in 0..5 {
            assert_eq!(state.get_cell(x as i32, 0), Cell::Wall);
            assert_eq!(state.get_cell(x as i32, 4), Cell::Wall);
        }
        for y in 0..5 {
            assert_eq!(state.get_cell(0, y as i32), Cell::Wall);
            assert_eq!(state.get_cell(4, y as i32), Cell::Wall);
        }
        assert_eq!(state.get_cell(1, 1), Cell::Empty);
    }

    #[test]
    fn get_cell_returns_wall_for_out_of_bounds() {
        let state = GameState::new(5, 5);

        assert_eq!(state.get_cell(-1, 2), Cell::Wall);
        assert_eq!(state.get_cell(5, 2), Cell::Wall);
        assert_eq!(state.get_cell(2, -1), Cell::Wall);
        assert_eq!(state.get_cell(2, 5), Cell::Wall);
    }
}
