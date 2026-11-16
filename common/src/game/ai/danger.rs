use crate::game::models::Cell;
use crate::game::state::GameState;

/// A bomb the bot is considering placing, used to simulate its blast.
#[derive(Debug, Clone, Copy)]
pub struct HypotheticalBomb {
    pub x: usize,
    pub y: usize,
    pub timer: u8,
    pub range: u8,
    pub pierce: bool,
}

#[derive(Debug, Clone, Copy)]
struct BombInfo {
    x: usize,
    y: usize,
    timer: u8,
    range: u8,
    pierce: bool,
}

/// For every cell, the number of beats before fire reaches it
/// (`Some(0)` = burning now, `None` = safe). Chain reactions are included.
pub struct DangerMap {
    width: usize,
    timers: Vec<Option<u8>>,
}

impl DangerMap {
    pub fn compute(state: &GameState) -> Self {
        Self::compute_with(state, None)
    }

    pub fn compute_with(state: &GameState, extra: Option<HypotheticalBomb>) -> Self {
        let mut bombs = collect_bombs(state, extra);
        propagate_chain_timers(state, &mut bombs);

        let mut timers = vec![None; state.width * state.height];
        for (idx, cell) in state.grid.iter().enumerate() {
            if matches!(cell, Cell::Explosion { .. }) {
                timers[idx] = Some(0);
            }
        }
        for bomb in &bombs {
            for (x, y) in blast_cells(state, &bombs, bomb.x, bomb.y, bomb.range, bomb.pierce) {
                let slot = &mut timers[y * state.width + x];
                *slot = Some(slot.map_or(bomb.timer, |t| t.min(bomb.timer)));
            }
        }
        Self {
            width: state.width,
            timers,
        }
    }

    /// Same map seen one beat later (used to plan moves after placing a bomb).
    pub fn shifted(mut self, beats: u8) -> Self {
        for timer in self.timers.iter_mut().flatten() {
            *timer = timer.saturating_sub(beats);
        }
        self
    }

    pub fn timer(&self, x: usize, y: usize) -> Option<u8> {
        self.timers.get(y * self.width + x).copied().flatten()
    }

    pub fn is_safe(&self, x: usize, y: usize) -> bool {
        self.timer(x, y).is_none()
    }

    /// Whether a cell reached after `step` moves is survivable.
    pub fn can_stand_at_step(&self, x: usize, y: usize, step: u32) -> bool {
        match self.timer(x, y) {
            None => true,
            Some(timer) => timer as u32 > step || (timer as u32) + 1 < step,
        }
    }
}

fn collect_bombs(state: &GameState, extra: Option<HypotheticalBomb>) -> Vec<BombInfo> {
    let mut bombs: Vec<BombInfo> = state
        .grid
        .iter()
        .enumerate()
        .filter_map(|(idx, cell)| match *cell {
            Cell::Bomb {
                ticks_left,
                range,
                pierce,
                ..
            } => Some(BombInfo {
                x: idx % state.width,
                y: idx / state.width,
                timer: ticks_left,
                range,
                pierce,
            }),
            _ => None,
        })
        .collect();
    if let Some(b) = extra {
        bombs.push(BombInfo {
            x: b.x,
            y: b.y,
            timer: b.timer,
            range: b.range,
            pierce: b.pierce,
        });
    }
    bombs
}

/// A bomb caught in another blast explodes at the same time.
fn propagate_chain_timers(state: &GameState, bombs: &mut [BombInfo]) {
    for _ in 0..bombs.len() {
        let mut has_changed = false;
        for i in 0..bombs.len() {
            let source = bombs[i];
            for (x, y) in blast_cells(state, bombs, source.x, source.y, source.range, source.pierce) {
                if let Some(target) = bombs.iter_mut().find(|b| b.x == x && b.y == y) {
                    if target.timer > source.timer {
                        target.timer = source.timer;
                        has_changed = true;
                    }
                }
            }
        }
        if !has_changed {
            return;
        }
    }
}

/// Cells reached by a blast from (x, y), mirroring `GameState::explode_bomb`.
fn blast_cells(
    state: &GameState,
    bombs: &[BombInfo],
    x: usize,
    y: usize,
    range: u8,
    pierce: bool,
) -> Vec<(usize, usize)> {
    let mut cells = vec![(x, y)];
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        for i in 1..=range as i32 {
            let (tx, ty) = (x as i32 + dx * i, y as i32 + dy * i);
            let cell = state.get_cell(tx, ty);
            if cell == Cell::Wall {
                break;
            }
            let (ux, uy) = (tx as usize, ty as usize);
            cells.push((ux, uy));
            let is_bomb = bombs.iter().any(|b| b.x == ux && b.y == uy);
            if is_bomb || (cell == Cell::Brick && !pierce) {
                break;
            }
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bomb(ticks_left: u8, range: u8) -> Cell {
        Cell::Bomb {
            owner_id: 1,
            ticks_left,
            range,
            pierce: false,
        }
    }

    #[test]
    fn compute_marks_blast_cells_with_bomb_timer() {
        let mut state = GameState::new(9, 3);
        state.grid[1 * 9 + 1] = bomb(3, 2);

        let danger = DangerMap::compute(&state);

        assert_eq!(danger.timer(3, 1), Some(3));
        assert!(danger.is_safe(4, 1));
    }

    #[test]
    fn compute_propagates_chain_reaction_timer() {
        let mut state = GameState::new(9, 3);
        state.grid[1 * 9 + 1] = bomb(1, 2);
        state.grid[1 * 9 + 3] = bomb(4, 2);

        let danger = DangerMap::compute(&state);

        assert_eq!(danger.timer(5, 1), Some(1));
    }

    #[test]
    fn compute_blast_is_stopped_by_brick() {
        let mut state = GameState::new(9, 3);
        state.grid[1 * 9 + 1] = bomb(2, 4);
        state.grid[1 * 9 + 2] = Cell::Brick;

        let danger = DangerMap::compute(&state);

        assert!(danger.is_safe(3, 1));
    }

    #[test]
    fn compute_with_includes_hypothetical_bomb() {
        let state = GameState::new(9, 3);
        let extra = HypotheticalBomb {
            x: 1,
            y: 1,
            timer: 4,
            range: 1,
            pierce: false,
        };

        let danger = DangerMap::compute_with(&state, Some(extra));

        assert_eq!(danger.timer(2, 1), Some(4));
    }
}
