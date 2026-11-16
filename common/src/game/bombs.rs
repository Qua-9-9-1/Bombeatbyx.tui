use crate::game::models::{
    BOMB_FUSE_BEATS, BRICK_BONUS_DROP_PERCENT, BRICK_BONUS_LIFETIME_TICKS, BRICK_POINTS, BonusType,
    Cell,
};
use crate::game::rhythm::BeatAccuracy;
use crate::game::state::GameState;

impl GameState {
    pub fn try_place_bomb(&mut self, player_id: u32, _accuracy: BeatAccuracy) -> bool {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if !player.is_alive {
                return false;
            }

            if player.active_bombs < player.max_bombs {
                let grid_x = (player.sub_x / 2) as usize;
                let grid_y = (player.sub_y / 1) as usize;
                let idx = grid_y * self.width + grid_x;

                if self.grid[idx] == Cell::Empty {
                    self.grid[idx] = Cell::Bomb {
                        owner_id: player.id,
                        ticks_left: BOMB_FUSE_BEATS,
                        range: player.effective_bomb_range().min(u8::MAX as usize) as u8,
                        pierce: player.pierce,
                    };
                    player.active_bombs += 1;
                    return true;
                }
            }
        }
        false
    }

    pub fn tick_bombs_and_explosions(&mut self) {
        let mut bombs_to_explode = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                match self.grid[idx] {
                    Cell::Bomb {
                        owner_id,
                        mut ticks_left,
                        range,
                        pierce,
                    } => {
                        ticks_left = ticks_left.saturating_sub(1);
                        self.grid[idx] = Cell::Bomb {
                            owner_id,
                            ticks_left,
                            range,
                            pierce,
                        };
                        if ticks_left == 0 {
                            bombs_to_explode.push((x, y));
                        }
                    }
                    Cell::Explosion {
                        mut ticks_left,
                        owner_id,
                    } => {
                        if ticks_left > 0 {
                            ticks_left -= 1;
                        }
                        self.grid[idx] = if ticks_left == 0 {
                            self.take_pending_drop(idx)
                        } else {
                            Cell::Explosion {
                                ticks_left,
                                owner_id,
                            }
                        };
                    }
                    Cell::Bonus {
                        bonus_type,
                        mut ticks_left,
                    } => {
                        if ticks_left > 0 {
                            ticks_left -= 1;
                        }
                        self.grid[idx] = if ticks_left == 0 {
                            Cell::Empty
                        } else {
                            Cell::Bonus {
                                bonus_type,
                                ticks_left,
                            }
                        };
                    }
                    _ => {}
                }
            }
        }

        self.detonate_chain(bombs_to_explode);
    }

    /// Explodes every listed bomb and, in the same tick, every bomb caught in
    /// one of the resulting blasts (chain reaction).
    pub fn detonate_chain(&mut self, mut pending: Vec<(usize, usize)>) {
        while let Some((bx, by)) = pending.pop() {
            pending.extend(self.explode_bomb(bx, by));
        }
    }

    fn take_pending_drop(&mut self, idx: usize) -> Cell {
        match self.pending_drops.iter().position(|&(i, _)| i == idx) {
            Some(pos) => {
                let (_, bonus_type) = self.pending_drops.swap_remove(pos);
                Cell::Bonus {
                    bonus_type,
                    ticks_left: BRICK_BONUS_LIFETIME_TICKS,
                }
            }
            None => Cell::Empty,
        }
    }

    /// Returns the positions of the other bombs reached by this blast.
    fn explode_bomb(&mut self, bx: usize, by: usize) -> Vec<(usize, usize)> {
        let (owner_id, range, pierce) = match self.grid[by * self.width + bx] {
            Cell::Bomb {
                owner_id,
                range,
                pierce,
                ..
            } => (owner_id, range as i32, pierce),
            _ => return Vec::new(),
        };

        if let Some(p) = self.players.iter_mut().find(|p| p.id == owner_id) {
            p.active_bombs = p.active_bombs.saturating_sub(1);
        }

        let fire = Cell::Explosion {
            ticks_left: 1,
            owner_id,
        };
        self.grid[by * self.width + bx] = fire;
        let mut chained = Vec::new();
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for (dx, dy) in directions {
            for i in 1..=range {
                let tx = bx as i32 + dx * i;
                let ty = by as i32 + dy * i;

                if tx < 0 || tx >= self.width as i32 || ty < 0 || ty >= self.height as i32 {
                    break;
                }
                let idx = (ty as usize) * self.width + (tx as usize);

                match self.grid[idx] {
                    Cell::Wall => break,
                    Cell::Brick => {
                        self.grid[idx] = fire;
                        self.award_points(owner_id, BRICK_POINTS);
                        if self.rng.chance(BRICK_BONUS_DROP_PERCENT) {
                            let bonus = BonusType::roll(&mut self.rng);
                            self.pending_drops.push((idx, bonus));
                        }
                        if !pierce {
                            break;
                        }
                    }
                    Cell::Empty | Cell::Explosion { .. } | Cell::Bonus { .. } => {
                        self.grid[idx] = fire;
                    }
                    Cell::Bomb { .. } => {
                        chained.push((tx as usize, ty as usize));
                        break;
                    }
                }
            }
        }
        chained
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::models::Player;
    use crate::game::rhythm::BeatAccuracy;

    fn create_test_player(id: u32) -> Player {
        let mut player = Player::new(id, format!("Player {}", id), "🤖", 3);
        player.sub_x = 2;
        player.sub_y = 1;
        player
    }

    #[test]
    fn try_place_bomb_increases_active_bombs_when_cell_empty() {
        let mut state = GameState::new(5, 5);
        state.players = vec![create_test_player(1)];

        let placed = state.try_place_bomb(1, BeatAccuracy::Perfect);

        assert!(placed);
        assert_eq!(state.players[0].active_bombs, 1);
        let cell_idx = 1 * 5 + 1;
        assert!(matches!(state.grid[cell_idx], Cell::Bomb { owner_id: 1, .. }));
    }

    #[test]
    fn try_place_bomb_fails_when_max_bombs_reached() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.active_bombs = 1;
        state.players = vec![player];

        let placed = state.try_place_bomb(1, BeatAccuracy::Perfect);

        assert!(!placed);
    }

    #[test]
    fn tick_bombs_and_explosions_triggers_explosion_on_zero_ticks() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.active_bombs = 1;
        player.bomb_range = 1;
        state.players = vec![player];

        let cell_idx = 2 * 5 + 2;
        state.grid[cell_idx] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
            range: 1,
            pierce: false,
        };

        state.tick_bombs_and_explosions();

        assert_eq!(state.players[0].active_bombs, 0);
        assert!(matches!(state.grid[cell_idx], Cell::Explosion { .. }));
        assert!(matches!(state.grid[2 * 5 + 1], Cell::Explosion { .. }));
        assert!(matches!(state.grid[2 * 5 + 3], Cell::Explosion { .. }));
        assert!(matches!(state.grid[1 * 5 + 2], Cell::Explosion { .. }));
        assert!(matches!(state.grid[3 * 5 + 2], Cell::Explosion { .. }));
    }

    #[test]
    fn tick_bombs_and_explosions_reveals_hidden_bonus_when_fire_dies_out() {
        use crate::game::models::BonusType;
        let mut state = GameState::new(5, 5);
        let idx = 1 * 5 + 3;
        state.grid[idx] = Cell::Explosion {
            ticks_left: 1,
            owner_id: 1,
        };
        state.pending_drops.push((idx, BonusType::BombRange));

        state.tick_bombs_and_explosions();

        assert!(matches!(
            state.grid[idx],
            Cell::Bonus {
                bonus_type: BonusType::BombRange,
                ..
            }
        ));
        assert!(state.pending_drops.is_empty());
    }

    #[test]
    fn tick_bombs_and_explosions_pierce_bomb_goes_through_bricks() {
        let mut state = GameState::new(9, 3);
        state.players = vec![create_test_player(1)];
        state.grid[1 * 9 + 2] = Cell::Brick;
        state.grid[1 * 9 + 3] = Cell::Brick;
        state.grid[1 * 9 + 1] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
            range: 3,
            pierce: true,
        };

        state.tick_bombs_and_explosions();

        assert!(matches!(state.grid[1 * 9 + 3], Cell::Explosion { .. }));
        assert!(matches!(state.grid[1 * 9 + 4], Cell::Explosion { .. }));
    }

    #[test]
    fn tick_bombs_and_explosions_awards_points_for_destroyed_brick() {
        let mut state = GameState::new(7, 3);
        state.players = vec![create_test_player(1)];
        state.grid[1 * 7 + 2] = Cell::Brick;
        state.grid[1 * 7 + 1] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
            range: 2,
            pierce: false,
        };

        state.tick_bombs_and_explosions();

        assert_eq!(state.players[0].score, BRICK_POINTS);
        assert!(matches!(state.grid[1 * 7 + 3], Cell::Empty));
    }

    #[test]
    fn try_place_bomb_locks_owner_range_at_placement() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.bomb_range = 3;
        state.players = vec![player];

        state.try_place_bomb(1, BeatAccuracy::Perfect);

        assert!(matches!(state.grid[1 * 5 + 1], Cell::Bomb { range: 3, .. }));
    }

    #[test]
    fn tick_bombs_and_explosions_uses_bomb_range_not_current_owner_range() {
        let mut state = GameState::new(9, 3);
        let mut player = create_test_player(1);
        player.bomb_range = 1;
        player.active_bombs = 1;
        state.players = vec![player];
        state.grid[1 * 9 + 1] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
            range: 3,
            pierce: false,
        };

        state.tick_bombs_and_explosions();

        assert!(matches!(state.grid[1 * 9 + 4], Cell::Explosion { .. }));
    }

    #[test]
    fn tick_bombs_and_explosions_chains_other_bombs_in_same_tick() {
        let mut state = GameState::new(9, 3);
        state.players = vec![create_test_player(1)];
        state.grid[1 * 9 + 1] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
            range: 2,
            pierce: false,
        };
        state.grid[1 * 9 + 3] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 4,
            range: 2,
            pierce: false,
        };

        state.tick_bombs_and_explosions();

        assert!(matches!(state.grid[1 * 9 + 3], Cell::Explosion { .. }));
        assert!(matches!(state.grid[1 * 9 + 5], Cell::Explosion { .. }));
    }

    #[test]
    fn tick_bombs_and_explosions_blast_stops_at_chained_bomb() {
        let mut state = GameState::new(9, 3);
        state.players = vec![create_test_player(1)];
        state.grid[1 * 9 + 1] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
            range: 5,
            pierce: false,
        };
        state.grid[1 * 9 + 2] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 4,
            range: 1,
            pierce: false,
        };

        state.tick_bombs_and_explosions();

        assert_eq!(state.grid[1 * 9 + 4], Cell::Empty);
    }

    #[test]
    fn tick_bombs_and_explosions_despawns_bonus_when_ticks_left_expires() {
        use crate::game::models::BonusType;
        let mut state = GameState::new(5, 5);
        let cell_idx = 1 * 5 + 1;
        state.grid[cell_idx] = Cell::Bonus {
            bonus_type: BonusType::BombQty,
            ticks_left: 1,
        };

        state.tick_bombs_and_explosions();

        assert_eq!(state.grid[cell_idx], Cell::Empty);
    }

    #[test]
    fn move_player_picks_up_bonus_with_ticks_left() {
        use crate::game::models::BonusType;
        let mut state = GameState::new(5, 5);
        let cell_idx = 1 * 5 + 2;
        state.grid[cell_idx] = Cell::Bonus {
            bonus_type: BonusType::BombQty,
            ticks_left: 10,
        };
        state.players = vec![create_test_player(1)];

        let moved = state.move_player(1, 2, 0);

        assert!(moved);
        assert_eq!(state.players[0].max_bombs, 2);
        assert_eq!(state.grid[cell_idx], Cell::Empty);
    }
}
