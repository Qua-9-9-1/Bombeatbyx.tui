use super::actions::GameAction;
use super::models::{Cell, Player, GameMode, BonusType, SecondItem};
use super::rhythm::BeatAccuracy;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

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
        }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Cell {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return Cell::Wall;
        }
        self.grid[(y as usize) * self.width + (x as usize)]
    }

    pub fn handle_action(
        &mut self,
        player_id: u32,
        action: GameAction,
        accuracy: BeatAccuracy,
        current_beat: u64,
    ) {
        if let GameAction::Emote(index) = action {
            self.trigger_emote(player_id, index);
            return;
        }

        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            let now = Instant::now();

            if let Some(lockout) = player.spam_lockout_until {
                if now < lockout {
                    player.spam_lockout_until = Some(now + Duration::from_millis(300));
                    return;
                }
            }

            if let Some(last_time) = player.last_action_time {
                let delay = now.duration_since(last_time);

                if delay < Duration::from_millis(100) {
                    player.spam_lockout_until = Some(now + Duration::from_millis(300));
                    player.last_action_time = Some(now);
                    return;
                }
            }

            player.last_action_time = Some(now);

            if let Some(last_beat) = player.last_acted_beat {
                if last_beat == current_beat {
                    return;
                }
            }

            player.last_acted_beat = Some(current_beat);
            player.last_accuracy = accuracy.clone();

            if matches!(accuracy, BeatAccuracy::Miss) {
                player.combo = 0;
                return;
            }

            player.combo = (player.combo + 1).min(9999);
            player.score += accuracy.bonus_points();
            match action {
                GameAction::MoveLeft => self.move_player(player_id, -2, 0),
                GameAction::MoveRight => self.move_player(player_id, 2, 0),
                GameAction::MoveUp => self.move_player(player_id, 0, -1),
                GameAction::MoveDown => self.move_player(player_id, 0, 1),
                GameAction::PlaceBomb => self.try_place_bomb(player_id, accuracy),
                GameAction::TriggerSpell => self.trigger_action_2(player_id, current_beat),
                GameAction::Emote(_) => {}
            }
        }
    }

    pub fn trigger_action_2(&mut self, player_id: u32, current_beat: u64) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if player.is_alive {
                if let Some(item) = player.second_item {
                    match item {
                        SecondItem::Shield => {
                            player.shield_until_beat = Some(current_beat);
                            player.second_item = None;
                        }
                    }
                }
            }
        }
    }

    pub fn trigger_emote(&mut self, player_id: u32, index: u8) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            let emote = match index {
                1 => "👋",
                2 => "✌️",
                3 => "🖕",
                4 => "👍",
                _ => return,
            };
            player.active_emote = Some(emote.to_string());
            player.emote_until = Some(std::time::Instant::now() + std::time::Duration::from_millis(1500));
        }
    }

    pub fn spawn_players(&mut self, mut players: Vec<Player>) {
        let active_players_indices: Vec<usize> = players.iter()
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

        if candidates.is_empty() {
            return None;
        }

        let alive_players: Vec<&Player> = self.players.iter()
            .filter(|p| p.is_alive)
            .collect();

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

    pub fn tick_beat(&mut self, beat_count: u64) {
        if self.bonus_every > 0 && beat_count > 0 && beat_count % (self.bonus_every as u64) == 0 {
            self.spawn_random_bonus();
        }
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

fn get_pseudo_random_u32() -> u32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_nanos();
    let mut x = now as u64;
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    x as u32
}

pub fn get_spawn_points(width: usize, height: usize, count: usize) -> Vec<(usize, usize)> {
    let mut candidates = vec![
        (1, 1),
        (width - 2, 1),
        (1, height - 2),
        (width - 2, height - 2),
        (width / 2, 1),
        (width / 2, height - 2),
        (1, height / 2),
        (width - 2, height / 2),
    ];

    candidates.dedup();

    let count = count.min(candidates.len()).max(1);
    if count == candidates.len() {
        let mut result = candidates;
        let mut seed = get_pseudo_random_u32();
        let n = result.len();
        for i in (1..n).rev() {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (seed as usize) % (i + 1);
            result.swap(i, j);
        }
        return result;
    }

    let mut best_subsets = Vec::new();
    let mut max_min_dist_sq = -1.0;

    let n = candidates.len();
    for mask in 0..(1 << n) {
        if (mask as i32).count_ones() as usize == count {
            let mut subset = Vec::new();
            for i in 0..n {
                if (mask & (1 << i)) != 0 {
                    subset.push(candidates[i]);
                }
            }

            let mut min_dist_sq = f64::MAX;
            for i in 0..subset.len() {
                for j in (i + 1)..subset.len() {
                    let dx = subset[i].0 as f64 - subset[j].0 as f64;
                    let dy = subset[i].1 as f64 - subset[j].1 as f64;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                    }
                }
            }

            if min_dist_sq > max_min_dist_sq {
                max_min_dist_sq = min_dist_sq;
                best_subsets.clear();
                best_subsets.push(subset);
            } else if (min_dist_sq - max_min_dist_sq).abs() < 1e-5 {
                best_subsets.push(subset);
            }
        }
    }

    let mut seed = get_pseudo_random_u32();
    let chosen_idx = (seed as usize) % best_subsets.len();
    let mut selected_subset = best_subsets[chosen_idx].clone();

    let n = selected_subset.len();
    for i in (1..n).rev() {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let j = (seed as usize) % (i + 1);
        selected_subset.swap(i, j);
    }

    selected_subset
}
