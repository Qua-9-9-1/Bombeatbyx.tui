use crate::game::models::{Cell, Direction};
use crate::game::state::GameState;
use std::collections::VecDeque;

pub const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

/// First move to take and number of moves needed to reach a goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathStep {
    pub first_move: Direction,
    pub distance: u32,
}

pub fn is_walkable(state: &GameState, x: i32, y: i32, bot_id: u32) -> bool {
    if !matches!(state.get_cell(x, y), Cell::Empty | Cell::Bonus { .. }) {
        return false;
    }
    !state.players.iter().any(|p| {
        p.id != bot_id && p.is_alive && !p.is_spectator && p.sub_x / 2 == x && p.sub_y == y
    })
}

/// Breadth-first search from `start` (the start cell itself is never a goal).
/// `can_enter(x, y, step)` filters cells by the number of moves needed to reach them.
pub fn find_path(
    state: &GameState,
    bot_id: u32,
    start: (usize, usize),
    max_depth: u32,
    can_enter: impl Fn(usize, usize, u32) -> bool,
    is_goal: impl Fn(usize, usize) -> bool,
) -> Option<PathStep> {
    let mut visited = vec![false; state.width * state.height];
    visited[start.1 * state.width + start.0] = true;
    let mut queue = VecDeque::new();
    queue.push_back((start.0, start.1, 0_u32, None::<Direction>));

    while let Some((x, y, depth, first_move)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        for direction in DIRECTIONS {
            let (dx, dy) = direction.delta();
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if !is_walkable(state, nx, ny, bot_id) {
                continue;
            }
            let (ux, uy) = (nx as usize, ny as usize);
            let idx = uy * state.width + ux;
            let step = depth + 1;
            if visited[idx] || !can_enter(ux, uy, step) {
                continue;
            }
            visited[idx] = true;
            let first = first_move.unwrap_or(direction);
            if is_goal(ux, uy) {
                return Some(PathStep {
                    first_move: first,
                    distance: step,
                });
            }
            queue.push_back((ux, uy, step, Some(first)));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_path_returns_first_move_towards_goal() {
        let state = GameState::new(9, 3);

        let path = find_path(&state, 1, (1, 1), 10, |_, _, _| true, |x, _| x == 4);

        assert_eq!(
            path,
            Some(PathStep {
                first_move: Direction::Right,
                distance: 3
            })
        );
    }

    #[test]
    fn find_path_returns_none_when_goal_blocked_by_brick() {
        let mut state = GameState::new(9, 3);
        state.grid[1 * 9 + 2] = Cell::Brick;

        let path = find_path(&state, 1, (1, 1), 10, |_, _, _| true, |x, _| x == 4);

        assert_eq!(path, None);
    }

    #[test]
    fn find_path_respects_can_enter_filter() {
        let state = GameState::new(9, 3);

        let path = find_path(&state, 1, (1, 1), 10, |x, _, _| x != 3, |x, _| x == 4);

        assert_eq!(path, None);
    }
}
