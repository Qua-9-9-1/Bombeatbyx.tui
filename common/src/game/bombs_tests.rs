use super::*;
use crate::game::models::Player;
use crate::game::rhythm::BeatAccuracy;

fn create_test_player(id: u32) -> Player {
    let mut player = Player::new(id, format!("Player {}", id), "🤖", 3);
    player.sub_x = 2;
    player.sub_y = 1;
    player
}

#[test]
fn try_place_bomb_increases_active_bombs_when_cell_empty() {
    let mut state = GameState::new(5, 5);
    state.players = vec![create_test_player(1)];

    let placed = state.try_place_bomb(1, BeatAccuracy::Perfect);

    assert!(placed);
    assert_eq!(state.players[0].active_bombs, 1);
    let cell_idx = 1 * 5 + 1;
    assert!(matches!(state.grid[cell_idx], Cell::Bomb { owner_id: 1, .. }));
}

#[test]
fn try_place_bomb_fails_when_max_bombs_reached() {
    let mut state = GameState::new(5, 5);
    let mut player = create_test_player(1);
    player.active_bombs = 1;
    state.players = vec![player];

    let placed = state.try_place_bomb(1, BeatAccuracy::Perfect);

    assert!(!placed);
}

#[test]
fn tick_bombs_and_explosions_triggers_explosion_on_zero_ticks() {
    let mut state = GameState::new(5, 5);
    let mut player = create_test_player(1);
    player.active_bombs = 1;
    player.bomb_range = 1;
    state.players = vec![player];

    let cell_idx = 2 * 5 + 2;
    state.grid[cell_idx] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 1,
        range: 1,
        pierce: false,
    };

    state.tick_bombs_and_explosions();

    assert_eq!(state.players[0].active_bombs, 0);
    assert!(matches!(state.grid[cell_idx], Cell::Explosion { .. }));
    assert!(matches!(state.grid[2 * 5 + 1], Cell::Explosion { .. }));
    assert!(matches!(state.grid[2 * 5 + 3], Cell::Explosion { .. }));
    assert!(matches!(state.grid[1 * 5 + 2], Cell::Explosion { .. }));
    assert!(matches!(state.grid[3 * 5 + 2], Cell::Explosion { .. }));
}

#[test]
fn tick_bombs_and_explosions_reveals_hidden_bonus_when_fire_dies_out() {
    use crate::game::models::BonusType;
    let mut state = GameState::new(5, 5);
    let idx = 1 * 5 + 3;
    state.grid[idx] = Cell::Explosion {
        ticks_left: 1,
        owner_id: 1,
    };
    state.pending_drops.push((idx, BonusType::BombRange));

    state.tick_bombs_and_explosions();

    assert!(matches!(
        state.grid[idx],
        Cell::Bonus {
            bonus_type: BonusType::BombRange,
            ..
        }
    ));
    assert!(state.pending_drops.is_empty());
}

#[test]
fn tick_bombs_and_explosions_pierce_bomb_goes_through_bricks() {
    let mut state = GameState::new(9, 3);
    state.players = vec![create_test_player(1)];
    state.grid[1 * 9 + 2] = Cell::Brick;
    state.grid[1 * 9 + 3] = Cell::Brick;
    state.grid[1 * 9 + 1] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 1,
        range: 3,
        pierce: true,
    };

    state.tick_bombs_and_explosions();

    assert!(matches!(state.grid[1 * 9 + 3], Cell::Explosion { .. }));
    assert!(matches!(state.grid[1 * 9 + 4], Cell::Explosion { .. }));
}

#[test]
fn tick_bombs_and_explosions_awards_points_for_destroyed_brick() {
    let mut state = GameState::new(7, 3);
    state.players = vec![create_test_player(1)];
    state.grid[1 * 7 + 2] = Cell::Brick;
    state.grid[1 * 7 + 1] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 1,
        range: 2,
        pierce: false,
    };

    state.tick_bombs_and_explosions();

    assert_eq!(state.players[0].score, BRICK_POINTS);
    assert!(matches!(state.grid[1 * 7 + 3], Cell::Empty));
}

#[test]
fn try_place_bomb_locks_owner_range_at_placement() {
    let mut state = GameState::new(5, 5);
    let mut player = create_test_player(1);
    player.bomb_range = 3;
    state.players = vec![player];

    state.try_place_bomb(1, BeatAccuracy::Perfect);

    assert!(matches!(state.grid[1 * 5 + 1], Cell::Bomb { range: 3, .. }));
}

#[test]
fn tick_bombs_and_explosions_uses_bomb_range_not_current_owner_range() {
    let mut state = GameState::new(9, 3);
    let mut player = create_test_player(1);
    player.bomb_range = 1;
    player.active_bombs = 1;
    state.players = vec![player];
    state.grid[1 * 9 + 1] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 1,
        range: 3,
        pierce: false,
    };

    state.tick_bombs_and_explosions();

    assert!(matches!(state.grid[1 * 9 + 4], Cell::Explosion { .. }));
}

#[test]
fn tick_bombs_and_explosions_chains_other_bombs_in_same_tick() {
    let mut state = GameState::new(9, 3);
    state.players = vec![create_test_player(1)];
    state.grid[1 * 9 + 1] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 1,
        range: 2,
        pierce: false,
    };
    state.grid[1 * 9 + 3] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 4,
        range: 2,
        pierce: false,
    };

    state.tick_bombs_and_explosions();

    assert!(matches!(state.grid[1 * 9 + 3], Cell::Explosion { .. }));
    assert!(matches!(state.grid[1 * 9 + 5], Cell::Explosion { .. }));
}

#[test]
fn tick_bombs_and_explosions_blast_stops_at_chained_bomb() {
    let mut state = GameState::new(9, 3);
    state.players = vec![create_test_player(1)];
    state.grid[1 * 9 + 1] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 1,
        range: 5,
        pierce: false,
    };
    state.grid[1 * 9 + 2] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 4,
        range: 1,
        pierce: false,
    };

    state.tick_bombs_and_explosions();

    assert_eq!(state.grid[1 * 9 + 4], Cell::Empty);
}

#[test]
fn tick_bombs_and_explosions_despawns_bonus_when_ticks_left_expires() {
    use crate::game::models::BonusType;
    let mut state = GameState::new(5, 5);
    let cell_idx = 1 * 5 + 1;
    state.grid[cell_idx] = Cell::Bonus {
        bonus_type: BonusType::BombQty,
        ticks_left: 1,
    };

    state.tick_bombs_and_explosions();

    assert_eq!(state.grid[cell_idx], Cell::Empty);
}

#[test]
fn move_player_picks_up_bonus_with_ticks_left() {
    use crate::game::models::BonusType;
    let mut state = GameState::new(5, 5);
    let cell_idx = 1 * 5 + 2;
    state.grid[cell_idx] = Cell::Bonus {
        bonus_type: BonusType::BombQty,
        ticks_left: 10,
    };
    state.players = vec![create_test_player(1)];

    let moved = state.move_player(1, 2, 0);

    assert!(moved);
    assert_eq!(state.players[0].max_bombs, 2);
    assert_eq!(state.grid[cell_idx], Cell::Empty);
}


#[test]
fn blast_destroys_a_bonus_lying_on_the_floor() {
    use crate::game::models::BonusType;
    let mut state = GameState::new(9, 3);
    state.players = vec![create_test_player(1)];
    let bonus_idx = 1 * 9 + 3;
    state.grid[bonus_idx] = Cell::Bonus {
        bonus_type: BonusType::BombQty,
        ticks_left: 20,
    };
    state.grid[1 * 9 + 1] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 1,
        range: 3,
        pierce: false,
    };

    state.tick_bombs_and_explosions();
    assert!(matches!(state.grid[bonus_idx], Cell::Explosion { .. }));

    state.tick_bombs_and_explosions();
    assert_eq!(state.grid[bonus_idx], Cell::Empty);
}