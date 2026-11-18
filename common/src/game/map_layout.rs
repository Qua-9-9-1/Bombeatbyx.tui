//! Arena patterns (indestructible walls). Every layout is mirror-symmetric on
//! both axes, so the spawn points (corners and edge middles) are all equal,
//! and keeps every free cell reachable from every other one.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MapLayout {
    /// Pillar every other cell: the classic grid.
    #[default]
    Classic,
    /// No pillars at all: long blasts, nowhere to hide but behind bricks.
    Open,
    /// Pillar grid plus a walled ring around the center, open in the middle
    /// of each side: the center becomes a dangerous arena.
    Ring,
    /// Horizontal corridors separated by walls, joined by a few crossings:
    /// long lanes where blasts travel far.
    Lanes,
}

pub const ALL_LAYOUTS: [MapLayout; 4] = [
    MapLayout::Classic,
    MapLayout::Open,
    MapLayout::Ring,
    MapLayout::Lanes,
];

impl MapLayout {
    pub fn label(self) -> &'static str {
        match self {
            MapLayout::Classic => "Classic",
            MapLayout::Open => "Open",
            MapLayout::Ring => "Ring",
            MapLayout::Lanes => "Lanes",
        }
    }

    pub fn next(self) -> Self {
        let pos = ALL_LAYOUTS.iter().position(|&l| l == self).unwrap_or(0);
        ALL_LAYOUTS[(pos + 1) % ALL_LAYOUTS.len()]
    }

    pub fn previous(self) -> Self {
        let pos = ALL_LAYOUTS.iter().position(|&l| l == self).unwrap_or(0);
        ALL_LAYOUTS[(pos + ALL_LAYOUTS.len() - 1) % ALL_LAYOUTS.len()]
    }

    /// Whether `(x, y)` is an indestructible wall on a `width` x `height` map.
    pub fn is_wall(self, x: usize, y: usize, width: usize, height: usize) -> bool {
        if x == 0 || y == 0 || x + 1 >= width || y + 1 >= height {
            return true;
        }
        let pillar = x % 2 == 0 && y % 2 == 0;
        match self {
            MapLayout::Classic => pillar,
            MapLayout::Open => false,
            MapLayout::Ring => pillar || is_ring_wall(x, y, width, height),
            MapLayout::Lanes => {
                // Mirrored column index, so crossings are symmetric.
                let mirrored_x = x.min(width - 1 - x);
                y % 2 == 0 && mirrored_x % 4 != 1
            }
        }
    }
}

fn is_ring_wall(x: usize, y: usize, width: usize, height: usize) -> bool {
    let (cx, cy) = (width / 2, height / 2);
    // Keeps at least two free rows/columns between the ring and the border,
    // where the spawns are.
    let radius = (cx.min(cy) / 2).min(cx.min(cy).saturating_sub(2));
    if radius < 2 {
        return false;
    }
    let dx = x.abs_diff(cx);
    let dy = y.abs_diff(cy);
    let on_ring = dx.max(dy) == radius;
    let is_gate = dx <= 1 || dy <= 1;
    on_ring && !is_gate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rules::MAP_SIZE_RANGE;
    use crate::game::spawns::get_spawn_points;

    fn free_cells(layout: MapLayout, w: usize, h: usize) -> Vec<(usize, usize)> {
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| !layout.is_wall(x, y, w, h))
            .collect()
    }

    fn reachable_from(layout: MapLayout, w: usize, h: usize, start: (usize, usize)) -> usize {
        let mut seen = vec![false; w * h];
        let mut stack = vec![start];
        let mut count = 0;
        while let Some((x, y)) = stack.pop() {
            if seen[y * w + x] || layout.is_wall(x, y, w, h) {
                continue;
            }
            seen[y * w + x] = true;
            count += 1;
            stack.extend([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]);
        }
        count
    }

    fn odd_sizes() -> impl Iterator<Item = usize> {
        MAP_SIZE_RANGE.filter(|s| s % 2 == 1)
    }

    #[test]
    fn every_layout_is_fully_connected_and_spawns_are_free() {
        for layout in ALL_LAYOUTS {
            for w in odd_sizes() {
                for h in odd_sizes() {
                    let free = free_cells(layout, w, h);
                    let spawns = get_spawn_points(w, h, 8);
                    for &(sx, sy) in &spawns {
                        assert!(!layout.is_wall(sx, sy, w, h), "{layout:?} {w}x{h} spawn {sx},{sy}");
                    }
                    let reached = reachable_from(layout, w, h, spawns[0]);
                    assert_eq!(reached, free.len(), "{layout:?} {w}x{h} is split");
                }
            }
        }
    }

    #[test]
    fn every_spawn_has_an_escape_route() {
        for layout in ALL_LAYOUTS {
            for w in odd_sizes() {
                for h in odd_sizes() {
                    for (sx, sy) in get_spawn_points(w, h, 8) {
                        let exits = [(sx - 1, sy), (sx + 1, sy), (sx, sy - 1), (sx, sy + 1)]
                            .iter()
                            .filter(|&&(x, y)| !layout.is_wall(x, y, w, h))
                            .count();
                        assert!(exits >= 1, "{layout:?} {w}x{h} spawn {sx},{sy} is boxed in");
                    }
                }
            }
        }
    }

    #[test]
    fn layouts_are_mirror_symmetric() {
        for layout in ALL_LAYOUTS {
            let (w, h) = (15, 13);
            for (x, y) in (0..h).flat_map(|y| (0..w).map(move |x| (x, y))) {
                let wall = layout.is_wall(x, y, w, h);
                assert_eq!(wall, layout.is_wall(w - 1 - x, y, w, h));
                assert_eq!(wall, layout.is_wall(x, h - 1 - y, w, h));
            }
        }
    }

    #[test]
    fn ring_actually_draws_walls_on_standard_maps() {
        let classic = free_cells(MapLayout::Classic, 15, 15).len();
        let ring = free_cells(MapLayout::Ring, 15, 15).len();
        assert!(ring < classic);
    }

    #[test]
    fn next_and_previous_cycle_through_all_layouts() {
        for layout in ALL_LAYOUTS {
            assert_eq!(layout.next().previous(), layout);
        }
        assert_eq!(MapLayout::Lanes.next(), MapLayout::Classic);
    }
}
