use common::game::{GameState, Cell};
use crate::game::app::{CELL_H, CELL_W};
use crate::game::input::InputState;

pub fn update_player_movement(game_state: &mut GameState, inputs: &InputState, tick_count: u32) {
    let (mut current_x, mut current_y, is_alive) = {
        let player = &game_state.players[0];
        (player.sub_x, player.sub_y, player.is_alive)
    };

    if !is_alive { return; }

    let mut move_x = 0;
    let mut move_y = 0;

    if inputs.left > 0 && tick_count % 4 == 0 { move_x = -1; }
    else if inputs.right > 0 && tick_count % 4 == 0 { move_x = 1; }

    if inputs.up > 0 && tick_count % 8 == 0 { move_y = -1; }
    else if inputs.down > 0 && tick_count % 8 == 0 { move_y = 1; }

    if move_x != 0 {
        let next_x = current_x + move_x;
        if is_position_valid(game_state, next_x, current_y) {
            current_x = next_x;
        }
    }

    if move_y != 0 {
        let next_y = current_y + move_y;
        let cell_left = current_x / CELL_W;
        let cell_right = (current_x + 1) / CELL_W;

        if cell_left != cell_right {
            let target_grid_y = next_y / CELL_H;
            let left_is_blocked = is_cell_blocked(game_state.get_cell(cell_left, target_grid_y));
            let right_is_blocked = is_cell_blocked(game_state.get_cell(cell_right, target_grid_y));

            if left_is_blocked && !right_is_blocked {
                if is_position_valid(game_state, current_x + 1, current_y) { current_x += 1; }
            } else if right_is_blocked && !left_is_blocked {
                if is_position_valid(game_state, current_x - 1, current_y) { current_x -= 1; }
            } else if !left_is_blocked && !right_is_blocked {
                current_y = next_y;
            }
        } else {
            if is_position_valid(game_state, current_x, next_y) {
                current_y = next_y;
            }
        }
    }

    let player = &mut game_state.players[0];
    player.sub_x = current_x;
    player.sub_y = current_y;
}

fn is_cell_blocked(cell: Cell) -> bool {
    matches!(cell, Cell::Wall | Cell::Brick | Cell::Bomb { .. })
}

fn is_position_valid(game_state: &GameState, sx: i32, sy: i32) -> bool {
    let points = [(sx, sy), (sx + 1, sy)];
    
    let p_grid_x = (game_state.players[0].sub_x + 1) / CELL_W;
    let p_grid_y = game_state.players[0].sub_y / CELL_H;

    for (px, py) in points {
        let cell_x = px / CELL_W;
        let cell_y = py / CELL_H;
        let cell = game_state.get_cell(cell_x, cell_y);

        if let Cell::Bomb { .. } = cell {
            if cell_x == p_grid_x && cell_y == p_grid_y {
                continue;
            }
            return false;
        }

        if matches!(cell, Cell::Wall | Cell::Brick) {
            return false;
        }
    }
    true
}