use common::game::{GameState, Cell};
use crate::game::controls::InputState;

pub fn update_player_movement(game_state: &mut GameState, inputs: &InputState) {
    let (mut current_x, mut current_y, is_alive) = {
        let player = &game_state.players[0];
        (player.sub_x, player.sub_y, player.is_alive)
    };

    if !is_alive { return; }

    let mut move_x = 0;
    let mut move_y = 0;

    if inputs.left { move_x = -2; }
    else if inputs.right { move_x = 2; }

    if inputs.up { move_y = -1; }
    else if inputs.down { move_y = 1; }

    if move_x != 0 {
        let next_x = current_x + move_x;
        if is_position_valid(game_state, next_x, current_y) {
            current_x = next_x;
        }
    }

    if move_y != 0 {
        let next_y = current_y + move_y;
        
        if is_position_valid(game_state, current_x, next_y) {
            current_y = next_y;
        } else if current_x % 4 != 0 {
            if is_position_valid(game_state, current_x - 2, current_y) {
                current_x -= 2;
            } else if is_position_valid(game_state, current_x + 2, current_y) {
                current_x += 2;
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
    let points = [
        (sx, sy),
        (sx + 3, sy),
        (sx, sy + 1),
        (sx + 3, sy + 1),
    ];
    
    let p_grid_x = (game_state.players[0].sub_x + 2) / 4;
    let p_grid_y = (game_state.players[0].sub_y + 1) / 2;

    for (px, py) in points {
        let cell_x = px / 4;
        let cell_y = py / 2;
        
        if cell_x < 0 || cell_x >= game_state.width as i32 || cell_y < 0 || cell_y >= game_state.height as i32 {
            return false;
        }

        let cell = game_state.get_cell(cell_x, cell_y);
        if let Cell::Bomb { .. } = cell {
            if cell_x == p_grid_x && cell_y == p_grid_y { continue; }
            return false;
        }
        if is_cell_blocked(cell) { return false; }
    }
    true
}