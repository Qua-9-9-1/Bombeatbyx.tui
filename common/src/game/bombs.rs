use crate::game::models::{
    BOMB_FUSE_BEATS, BRICK_BONUS_LIFETIME_TICKS, BRICK_POINTS, BonusType, Cell,
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
                        if self.take_hidden_bonus(idx) {
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
#[path = "bombs_tests.rs"]
mod tests;
