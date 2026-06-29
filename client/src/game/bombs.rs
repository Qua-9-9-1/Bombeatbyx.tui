use common::game::{GameState, Cell};
use crate::game::app::{CELL_H, CELL_W};

pub fn try_place_bomb(game_state: &mut GameState) {
    let player = &game_state.players[0];
    if !player.is_alive { return; }

    if player.active_bombs < player.max_bombs {
        let grid_x = ((player.sub_x + 1) / CELL_W) as usize;
        let grid_y = (player.sub_y / CELL_H) as usize;
        let idx = grid_y * game_state.width + grid_x;

        if game_state.grid[idx] == Cell::Empty {
            game_state.grid[idx] = Cell::Bomb {
                owner_id: player.id,
                ticks_left: 150,
            };
            game_state.players[0].active_bombs += 1;
        }
    }
}

pub fn tick_bombs_and_explosions(game_state: &mut GameState) {
    let width = game_state.width;
    let height = game_state.height;
    let mut bombs_to_explode = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            match game_state.grid[idx] {
                Cell::Bomb { owner_id, mut ticks_left } => {
                    if ticks_left > 0 { ticks_left -= 1; }
                    game_state.grid[idx] = Cell::Bomb { owner_id, ticks_left };
                    if ticks_left == 0 { bombs_to_explode.push((x, y, owner_id)); }
                }
                Cell::Explosion { mut ticks_left } => {
                    if ticks_left > 0 { ticks_left -= 1; }
                    game_state.grid[idx] = if ticks_left == 0 { Cell::Empty } else { Cell::Explosion { ticks_left } };
                }
                _ => {}
            }
        }
    }

    for (bx, by, owner_id) in bombs_to_explode {
        explode_bomb(game_state, bx, by, owner_id);
    }
}

fn explode_bomb(game_state: &mut GameState, bx: usize, by: usize, owner_id: u32) {
    let width = game_state.width;
    let height = game_state.height;
    let range = game_state.players.iter().find(|p| p.id == owner_id).map(|p| p.bomb_range).unwrap_or(2);

    if let Some(p) = game_state.players.iter_mut().find(|p| p.id == owner_id) {
        if p.active_bombs > 0 { p.active_bombs -= 1; }
    }

    game_state.grid[by * width + bx] = Cell::Explosion { ticks_left: 30 };
    let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for (dx, dy) in directions {
        for i in 1..=range {
            let tx = bx as i32 + dx * (i as i32);
            let ty = by as i32 + dy * (i as i32);

            if tx < 0 || tx >= width as i32 || ty < 0 || ty >= height as i32 { break; }
            let idx = (ty as usize) * width + (tx as usize);

            match game_state.grid[idx] {
                Cell::Wall => break,
                Cell::Brick => {
                    game_state.grid[idx] = Cell::Explosion { ticks_left: 30 };
                    break;
                }
                Cell::Empty | Cell::Explosion { .. } => {
                    game_state.grid[idx] = Cell::Explosion { ticks_left: 30 };
                }
                Cell::Bomb { owner_id: other_owner, .. } => {
                    game_state.grid[idx] = Cell::Bomb { owner_id: other_owner, ticks_left: 0 };
                }
            }
        }
    }
}

pub fn check_deaths(game_state: &mut GameState) {
    for i in 0..game_state.players.len() {
        let (cx, cy, is_alive) = {
            let player = &game_state.players[i];
            ((player.sub_x + 1) / CELL_W, player.sub_y / CELL_H, player.is_alive)
        };

        if !is_alive { continue; }

        if let Cell::Explosion { .. } = game_state.get_cell(cx, cy) {
            game_state.players[i].is_alive = false;
        }
    }
}