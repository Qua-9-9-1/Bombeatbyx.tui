use crate::game::models::{
    BONUS_PICKUP_POINTS, BonusType, Cell, DASH_DISTANCE, Direction, MAX_LIVES, Player, SecondItem,
};
use crate::game::state::GameState;

fn direction_from_delta(move_x: i32, move_y: i32) -> Option<Direction> {
    match (move_x.signum(), move_y.signum()) {
        (0, -1) => Some(Direction::Up),
        (0, 1) => Some(Direction::Down),
        (-1, 0) => Some(Direction::Left),
        (1, 0) => Some(Direction::Right),
        _ => None,
    }
}

fn apply_bonus(player: &mut Player, bonus: BonusType) {
    player.score = player.score.saturating_add(BONUS_PICKUP_POINTS);
    match bonus {
        BonusType::BombQty => {
            player.max_bombs = player.max_bombs.saturating_add(1);
            player.collected_bonuses.push("💣".to_string());
        }
        BonusType::BombRange => {
            player.bomb_range = player.bomb_range.saturating_add(1);
            player.collected_bonuses.push("🔥".to_string());
        }
        BonusType::Pierce => {
            if !player.pierce {
                player.collected_bonuses.push("🔪".to_string());
            }
            player.pierce = true;
        }
        BonusType::ExtraLife => {
            if player.lives < MAX_LIVES {
                player.lives += 1;
            }
        }
        BonusType::Shield => player.second_item = Some(SecondItem::Shield),
        BonusType::Detonator => player.second_item = Some(SecondItem::Detonator),
        BonusType::Dash => player.second_item = Some(SecondItem::Dash),
    }
}

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

        if let Some(direction) = direction_from_delta(move_x, move_y) {
            if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                player.facing = direction;
            }
        }

        let Some((next_x, next_y)) = target_pos else {
            return false;
        };
        if !self.is_position_valid(player_id, next_x, next_y) {
            return false;
        }

        let idx = (next_y as usize) * self.width + (next_x / 2) as usize;
        let picked_bonus = match self.grid[idx] {
            Cell::Bonus { bonus_type, .. } => {
                self.grid[idx] = Cell::Empty;
                Some(bonus_type)
            }
            _ => None,
        };

        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            player.sub_x = next_x;
            player.sub_y = next_y;
            if let Some(bonus) = picked_bonus {
                apply_bonus(player, bonus);
            }
        }
        true
    }

    /// Moves up to `DASH_DISTANCE` cells in the facing direction, stopping at
    /// the first obstacle. Returns `true` if the player moved at least once.
    pub fn dash_player(&mut self, player_id: u32) -> bool {
        let Some(player) = self.players.iter().find(|p| p.id == player_id) else {
            return false;
        };
        let (dx, dy) = player.facing.delta();
        let mut has_moved = false;
        for _ in 0..DASH_DISTANCE {
            if !self.move_player(player_id, dx * 2, dy) {
                break;
            }
            has_moved = true;
        }
        has_moved
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::models::Player;

    fn create_test_player(id: u32, sub_x: i32, sub_y: i32) -> Player {
        let mut player = Player::new(id, format!("Player {}", id), "🤖", 3);
        player.sub_x = sub_x;
        player.sub_y = sub_y;
        player
    }

    #[test]
    fn move_player_updates_coordinates_when_valid() {
        let mut state = GameState::new(5, 5);
        state.players = vec![create_test_player(1, 2, 1)];

        let moved = state.move_player(1, 2, 0);

        assert!(moved);
        assert_eq!(state.players[0].sub_x, 4);
        assert_eq!(state.players[0].sub_y, 1);
    }

    #[test]
    fn move_player_fails_when_collision_with_wall() {
        let mut state = GameState::new(5, 5);
        state.grid[1 * 5 + 2] = Cell::Wall;
        state.players = vec![create_test_player(1, 2, 1)];

        let moved = state.move_player(1, 2, 0);

        assert!(!moved);
        assert_eq!(state.players[0].sub_x, 2);
    }

    #[test]
    fn move_player_updates_facing_even_when_blocked() {
        let mut state = GameState::new(5, 5);
        state.players = vec![create_test_player(1, 2, 1)];

        state.move_player(1, 0, -1);

        assert_eq!(state.players[0].facing, Direction::Up);
    }

    #[test]
    fn move_player_extra_life_bonus_is_capped() {
        let mut state = GameState::new(5, 5);
        state.grid[1 * 5 + 2] = Cell::Bonus {
            bonus_type: BonusType::ExtraLife,
            ticks_left: 10,
        };
        let mut player = create_test_player(1, 2, 1);
        player.lives = MAX_LIVES;
        state.players = vec![player];

        state.move_player(1, 2, 0);

        assert_eq!(state.players[0].lives, MAX_LIVES);
    }

    #[test]
    fn move_player_extra_life_never_lowers_lives_above_cap() {
        let mut state = GameState::new(5, 5);
        state.grid[1 * 5 + 2] = Cell::Bonus {
            bonus_type: BonusType::ExtraLife,
            ticks_left: 10,
        };
        let mut player = create_test_player(1, 2, 1);
        player.lives = MAX_LIVES + 3;
        state.players = vec![player];

        state.move_player(1, 2, 0);

        assert_eq!(state.players[0].lives, MAX_LIVES + 3);
    }

    #[test]
    fn move_player_fails_when_collision_with_other_player() {
        let mut state = GameState::new(5, 5);
        state.players = vec![
            create_test_player(1, 2, 1),
            create_test_player(2, 4, 1),
        ];

        let moved = state.move_player(1, 2, 0);

        assert!(!moved);
    }
}
