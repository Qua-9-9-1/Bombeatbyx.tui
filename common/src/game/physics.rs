use crate::game::models::{BonusType, Cell, SecondItem};
use crate::game::state::GameState;

impl GameState {
    pub fn move_player(&mut self, player_id: u32, move_x: i32, move_y: i32) -> bool {
        let target_pos = if let Some(player) = self.players.iter().find(|p| p.id == player_id) {
            if !player.is_alive {
                return false;
            }
            Some((player.sub_x + move_x, player.sub_y + move_y))
        } else {
            None
        };

        if let Some((next_x, next_y)) = target_pos {
            if self.is_position_valid(player_id, next_x, next_y) {
                if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                    player.sub_x = next_x;
                    player.sub_y = next_y;

                    let cell_x = (next_x / 2) as usize;
                    let cell_y = next_y as usize;
                    let idx = cell_y * self.width + cell_x;
                    if let Cell::Bonus(b_type) = self.grid[idx] {
                        match b_type {
                            BonusType::BombQty => {
                                player.max_bombs = player.max_bombs.saturating_add(1);
                                player.collected_bonuses.push("💣".to_string());
                            }
                            BonusType::BombRange => {
                                player.bomb_range = player.bomb_range.saturating_add(1);
                                player.collected_bonuses.push("🔥".to_string());
                            }
                            BonusType::Shield => {
                                player.second_item = Some(SecondItem::Shield);
                            }
                        }
                        self.grid[idx] = Cell::Empty;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn is_position_valid(&self, player_id: u32, sx: i32, sy: i32) -> bool {
        let cell_x = sx / 2;
        let cell_y = sy / 1;

        if cell_x < 0 || cell_x >= self.width as i32 || cell_y < 0 || cell_y >= self.height as i32 {
            return false;
        }

        let cell = self.get_cell(cell_x, cell_y);

        if let Cell::Bomb { .. } = cell {
            if let Some(player) = self.players.iter().find(|p| p.id == player_id) {
                let p_grid_x = player.sub_x / 2;
                let p_grid_y = player.sub_y / 1;
                if cell_x == p_grid_x && cell_y == p_grid_y {
                    return true;
                }
            }
            return false;
        }

        if matches!(cell, Cell::Wall | Cell::Brick) {
            return false;
        }

        for other in &self.players {
            if other.id != player_id && other.is_alive && !other.is_spectator {
                let other_grid_x = other.sub_x / 2;
                let other_grid_y = other.sub_y / 1;
                if cell_x == other_grid_x && cell_y == other_grid_y {
                    return false;
                }
            }
        }

        true
    }
}
