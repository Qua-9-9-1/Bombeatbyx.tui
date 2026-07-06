use super::GameState;
use crate::game::models::{BonusType, Cell, Player};
use crate::game::spawns::{get_pseudo_random_u32, get_spawn_points};

impl GameState {
    pub fn spawn_players(&mut self, mut players: Vec<Player>) {
        let active_players_indices: Vec<usize> = players
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_spectator)
            .map(|(i, _)| i)
            .collect();

        let spawns = get_spawn_points(self.width, self.height, active_players_indices.len());
        for (i, &idx) in active_players_indices.iter().enumerate() {
            if i < spawns.len() {
                players[idx].sub_x = (spawns[i].0 * 2) as i32;
                players[idx].sub_y = spawns[i].1 as i32;
                players[idx].is_alive = true;
            }
        }

        for p in &mut players {
            if p.is_spectator {
                p.is_alive = false;
                p.sub_x = -100;
                p.sub_y = -100;
            }
        }

        self.players = players;
    }

    pub fn tick_respawns(&mut self) {
        let now = std::time::Instant::now();
        let mut respawns_to_process = Vec::new();
        for (i, player) in self.players.iter().enumerate() {
            if let Some(timer) = player.respawn_timer {
                if now >= timer {
                    respawns_to_process.push(i);
                }
            }
        }

        for idx in respawns_to_process {
            self.players[idx].respawn_timer = None;
            self.players[idx].death_pos = None;
            if self.players[idx].lives > 0 {
                if let Some((rx, ry)) = self.calculate_respawn_position() {
                    self.players[idx].sub_x = (rx * 2) as i32;
                    self.players[idx].sub_y = ry as i32;
                    self.players[idx].is_alive = true;
                    self.players[idx].active_bombs = 0;
                }
            }
        }
    }

    pub fn calculate_respawn_position(&self) -> Option<(usize, usize)> {
        let candidates = self.get_respawn_candidates();
        self.select_furthest_spawn(candidates)
    }

    fn get_respawn_candidates(&self) -> Vec<(usize, usize)> {
        let mut candidates = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if self.grid[idx] == Cell::Empty {
                    let mut solid_count = 0;
                    let neighbors = [
                        (x as i32 - 1, y as i32),
                        (x as i32 + 1, y as i32),
                        (x as i32, y as i32 - 1),
                        (x as i32, y as i32 + 1),
                    ];
                    for &(nx, ny) in &neighbors {
                        if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                            let n_idx = (ny as usize) * self.width + (nx as usize);
                            if self.grid[n_idx] == Cell::Wall || self.grid[n_idx] == Cell::Brick {
                                solid_count += 1;
                            }
                        } else {
                            solid_count += 1;
                        }
                    }
                    if solid_count < 3 {
                        candidates.push((x, y));
                    }
                }
            }
        }

        if candidates.is_empty() {
            for y in 0..self.height {
                for x in 0..self.width {
                    let idx = y * self.width + x;
                    if self.grid[idx] == Cell::Empty {
                        candidates.push((x, y));
                    }
                }
            }
        }
        candidates
    }

    fn select_furthest_spawn(&self, candidates: Vec<(usize, usize)>) -> Option<(usize, usize)> {
        if candidates.is_empty() {
            return None;
        }

        let alive_players: Vec<&Player> = self.players.iter().filter(|p| p.is_alive).collect();

        if alive_players.is_empty() {
            return Some(candidates[0]);
        }

        let mut best_candidate = candidates[0];
        let mut max_min_dist_sq = -1.0;

        for (cx, cy) in candidates {
            let mut min_dist_sq = f64::MAX;
            for p in &alive_players {
                let px = (p.sub_x / 2) as f64;
                let py = p.sub_y as f64;
                let dx = cx as f64 - px;
                let dy = cy as f64 - py;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                }
            }

            if min_dist_sq > max_min_dist_sq {
                max_min_dist_sq = min_dist_sq;
                best_candidate = (cx, cy);
            }
        }

        Some(best_candidate)
    }

    pub fn spawn_random_bonus(&mut self) {
        let mut empty_cells = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if self.grid[idx] == Cell::Empty {
                    let player_here = self.players.iter().any(|p| {
                        p.is_alive && (p.sub_x / 2) as usize == x && p.sub_y as usize == y
                    });
                    if !player_here {
                        empty_cells.push(idx);
                    }
                }
            }
        }
        if !empty_cells.is_empty() {
            let seed = get_pseudo_random_u32();
            let idx = empty_cells[(seed as usize) % empty_cells.len()];
            let bonus_type = match seed % 3 {
                0 => BonusType::BombQty,
                1 => BonusType::BombRange,
                _ => BonusType::Shield,
            };
            self.grid[idx] = Cell::Bonus(bonus_type);
        }
    }
}
