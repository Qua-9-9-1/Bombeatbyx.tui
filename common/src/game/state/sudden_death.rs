use super::GameState;
use crate::game::models::Cell;

pub const SUDDEN_DEATH_START_SECS: u32 = 120;
/// Number of innermost rings that are never closed (a 3x3 area on odd maps).
const SUDDEN_DEATH_KEPT_RINGS: usize = 2;

impl GameState {
    /// Sudden death starts after 2 minutes, or at half the time limit if shorter.
    pub fn sudden_death_start_secs(&self) -> u32 {
        match self.time_limit_mins {
            Some(mins) => (mins * 60 / 2).min(SUDDEN_DEATH_START_SECS),
            None => SUDDEN_DEATH_START_SECS,
        }
    }

    pub fn is_sudden_death_active(&self) -> bool {
        self.sudden_death && self.elapsed_time_secs >= self.sudden_death_start_secs()
    }

    /// Closes the next cell of the shrinking spiral (one per beat).
    pub fn tick_sudden_death(&mut self, current_beat: u64) {
        if !self.is_sudden_death_active() {
            return;
        }
        let order = self.sudden_death_order();
        while let Some(&idx) = order.get(self.sudden_death_progress) {
            self.sudden_death_progress += 1;
            if self.grid[idx] != Cell::Wall {
                self.close_cell(idx, current_beat);
                return;
            }
        }
    }

    /// Inner cells ordered ring by ring (outermost first), clockwise.
    fn sudden_death_order(&self) -> Vec<usize> {
        let mut order = Vec::new();
        let max_ring = (self.width.min(self.height).saturating_sub(1)) / 2;
        let last_closed_ring = max_ring.saturating_sub(SUDDEN_DEATH_KEPT_RINGS);

        for ring in 1..=last_closed_ring {
            let (left, top) = (ring, ring);
            let (right, bottom) = (self.width - 1 - ring, self.height - 1 - ring);
            for x in left..=right {
                order.push(top * self.width + x);
            }
            for y in top + 1..=bottom {
                order.push(y * self.width + right);
            }
            for x in (left..right).rev() {
                order.push(bottom * self.width + x);
            }
            for y in (top + 1..bottom).rev() {
                order.push(y * self.width + left);
            }
        }
        order
    }

    fn close_cell(&mut self, idx: usize, current_beat: u64) {
        if let Cell::Bomb { owner_id, .. } = self.grid[idx] {
            if let Some(owner) = self.players.iter_mut().find(|p| p.id == owner_id) {
                owner.active_bombs = owner.active_bombs.saturating_sub(1);
            }
        }
        self.pending_drops.retain(|&(drop_idx, _)| drop_idx != idx);
        self.grid[idx] = Cell::Wall;

        let (x, y) = ((idx % self.width) as i32, (idx / self.width) as i32);
        let crushed: Vec<usize> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_alive && !p.is_spectator && p.sub_x / 2 == x && p.sub_y == y)
            .map(|(i, _)| i)
            .collect();
        for index in crushed {
            self.kill_player(index, None, current_beat);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::models::Player;

    fn sudden_death_state() -> GameState {
        let mut state = GameState::new(9, 9);
        state.sudden_death = true;
        state.elapsed_time_secs = SUDDEN_DEATH_START_SECS;
        state
    }

    #[test]
    fn tick_sudden_death_does_nothing_before_start() {
        let mut state = sudden_death_state();
        state.elapsed_time_secs = SUDDEN_DEATH_START_SECS - 1;

        state.tick_sudden_death(1);

        assert_eq!(state.get_cell(1, 1), Cell::Empty);
    }

    #[test]
    fn tick_sudden_death_closes_outer_ring_first() {
        let mut state = sudden_death_state();

        state.tick_sudden_death(1);

        assert_eq!(state.get_cell(1, 1), Cell::Wall);
    }

    #[test]
    fn tick_sudden_death_crushes_player_on_closed_cell() {
        let mut state = sudden_death_state();
        let mut player = Player::new(1, "Player 1", "🤖", 3);
        player.sub_x = 2;
        player.sub_y = 1;
        state.players = vec![player];

        state.tick_sudden_death(1);

        assert!(!state.players[0].is_alive);
        assert_eq!(state.players[0].lives, 2);
    }

    #[test]
    fn tick_sudden_death_keeps_center_area_open() {
        let mut state = sudden_death_state();

        for beat in 0..500 {
            state.tick_sudden_death(beat);
        }

        assert_eq!(state.get_cell(3, 3), Cell::Empty);
        assert_eq!(state.get_cell(5, 5), Cell::Empty);
        assert_eq!(state.get_cell(1, 1), Cell::Wall);
    }

    #[test]
    fn sudden_death_start_secs_uses_half_of_short_time_limit() {
        let mut state = sudden_death_state();
        state.time_limit_mins = Some(2);

        let start = state.sudden_death_start_secs();

        assert_eq!(start, 60);
    }
}
