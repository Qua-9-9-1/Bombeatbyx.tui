use super::actions::GameAction;
use super::models::{Cell, Player};
use super::rhythm::BeatAccuracy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
    pub players: Vec<Player>,
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
            grid,
            players: Vec::new(),
        }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Cell {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return Cell::Wall;
        }
        self.grid[(y as usize) * self.width + (x as usize)]
    }

    pub fn handle_action(&mut self, player_id: u32, action: GameAction, accuracy: BeatAccuracy) {
        if matches!(accuracy, BeatAccuracy::Miss) {
            return;
        }

        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            player.score += accuracy.bonus_points();
        }

        match action {
            GameAction::MoveLeft => self.move_player(player_id, -2, 0),
            GameAction::MoveRight => self.move_player(player_id, 2, 0),
            GameAction::MoveUp => self.move_player(player_id, 0, -1),
            GameAction::MoveDown => self.move_player(player_id, 0, 1),
            GameAction::PlaceBomb => self.try_place_bomb(player_id, accuracy),
            GameAction::TriggerSpell => self.trigger_action_2(player_id),
        }
    }

    pub fn trigger_action_2(&mut self, player_id: u32) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if player.is_alive && player.bomb_range < 6 {
                player.bomb_range += 1;
            }
        }
    }
}
