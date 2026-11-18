//! How each arena cell looks: symbol and colors. Bomb timers and danger are
//! shown with colors only, so emojis never have to share their two columns
//! with a digit.

use common::game::Cell;
use common::game::ai::danger::DangerMap;
use common::game::state::GameState;
use ratatui::style::Color;

/// Floor about to burn on the next beat (two tones, like the checkerboard).
const DANGER_DARK: Color = Color::Rgb(75, 18, 18);
const DANGER_LIGHT: Color = Color::Rgb(95, 24, 24);
const BRICK_FG: Color = Color::Rgb(205, 133, 63);
const BRICK_BG: Color = Color::Rgb(139, 69, 19);

/// Per-frame facts the cells need, computed once for the whole map.
pub struct MapView {
    danger: DangerMap,
    cracked: Vec<bool>,
    width: usize,
}

impl MapView {
    pub fn new(state: &GameState) -> Self {
        let mut cracked = vec![false; state.grid.len()];
        if state.brick_hints {
            for &idx in &state.cracked_bricks {
                if let Some(slot) = cracked.get_mut(idx) {
                    *slot = true;
                }
            }
        }
        Self {
            danger: DangerMap::compute(state),
            cracked,
            width: state.width,
        }
    }

    /// Fire reaches this cell on the next beat (bomb fuse at its last beat).
    pub fn burns_next_beat(&self, x: usize, y: usize) -> bool {
        self.danger.timer(x, y) == Some(1)
    }

    pub fn is_cracked(&self, x: usize, y: usize) -> bool {
        self.cracked.get(y * self.width + x).copied().unwrap_or(false)
    }
}

/// Bomb background: darker red when just placed, glowing red on its last beat.
pub fn bomb_heat(ticks_left: u8) -> Color {
    match ticks_left {
        0 | 1 => Color::Rgb(235, 25, 10),
        2 => Color::Rgb(180, 30, 15),
        3 => Color::Rgb(125, 30, 20),
        _ => Color::Rgb(80, 28, 24),
    }
}

/// Floor color, tinted red where the next beat's blasts will land.
pub fn floor(view: &MapView, x: usize, y: usize, beat: u64) -> Color {
    if !view.burns_next_beat(x, y) {
        return super::beat_fx::floor_color(x, y, beat);
    }
    if (x + y + beat as usize) % 2 == 0 {
        DANGER_DARK
    } else {
        DANGER_LIGHT
    }
}

pub fn brick(cracked: bool, ascii: bool) -> (&'static str, Color, Color) {
    let symbol = match (cracked, ascii) {
        (false, false) => "░░",
        (true, false) => "▒▒",
        (false, true) => "[]",
        (true, true) => "{}",
    };
    let fg = if cracked { Color::Rgb(255, 200, 90) } else { BRICK_FG };
    (symbol, fg, BRICK_BG)
}

pub fn bomb_symbol(ascii: bool) -> &'static str {
    if ascii { "()" } else { "💣" }
}

/// A player standing on a bomb alternates with it, half a beat each.
pub fn shows_bomb_over_player(beat_progress: f64) -> bool {
    beat_progress >= 0.5
}

pub fn is_bomb(cell: Cell) -> Option<u8> {
    match cell {
        Cell::Bomb { ticks_left, .. } => Some(ticks_left),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bombs_get_redder_as_they_tick_down() {
        let reds: Vec<u8> = (1..=4)
            .rev()
            .map(|ticks| match bomb_heat(ticks) {
                Color::Rgb(r, _, _) => r,
                _ => 0,
            })
            .collect();

        assert!(reds.windows(2).all(|pair| pair[0] < pair[1]), "{reds:?}");
    }

    #[test]
    fn floor_turns_red_only_where_fire_lands_next_beat() {
        let mut state = GameState::new(9, 3);
        state.grid[1 * 9 + 1] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
            range: 2,
            pierce: false,
        };
        state.grid[1 * 9 + 7] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 3,
            range: 1,
            pierce: false,
        };
        let view = MapView::new(&state);

        assert!(view.burns_next_beat(3, 1));
        assert!(!view.burns_next_beat(4, 1));
        assert!(!view.burns_next_beat(6, 1));
        assert!(matches!(floor(&view, 3, 1, 0), Color::Rgb(..)));
    }

    #[test]
    fn cracked_bricks_need_the_room_setting() {
        let mut state = GameState::new(9, 3);
        state.cracked_bricks = vec![1 * 9 + 3];
        assert!(!MapView::new(&state).is_cracked(3, 1));

        state.brick_hints = true;
        assert!(MapView::new(&state).is_cracked(3, 1));
    }
}
