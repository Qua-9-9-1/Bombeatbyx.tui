use super::*;

fn create_test_player(id: u32, is_spectator: bool, is_alive: bool) -> Player {
    let mut player = Player::new(id, format!("Player {}", id), "🤖", 3);
    player.is_spectator = is_spectator;
    player.is_alive = is_alive;
    player
}

#[test]
fn spawn_players_positions_active_and_spectators() {
    let mut state = GameState::new(15, 15);
    let players = vec![
        create_test_player(1, false, false),
        create_test_player(2, true, false),
    ];

    state.spawn_players(players);

    assert!(state.players[0].is_alive);
    assert!(state.players[0].sub_x >= 0);
    assert!(!state.players[1].is_alive);
    assert_eq!(state.players[1].sub_x, -100);
    assert_eq!(state.players[1].sub_y, -100);
}

#[test]
fn generate_bricks_keeps_spawn_area_free() {
    let mut state = GameState::new(15, 15);
    let mut player = create_test_player(1, false, true);
    player.sub_x = 2;
    player.sub_y = 1;
    state.players = vec![player];

    state.generate_bricks(&mut GameRng::seeded(7), 100);

    assert_eq!(state.get_cell(1, 1), Cell::Empty);
    assert_eq!(state.get_cell(2, 1), Cell::Empty);
    assert_eq!(state.get_cell(3, 1), Cell::Empty);
    assert_eq!(state.get_cell(1, 3), Cell::Empty);
    assert_eq!(state.get_cell(5, 5), Cell::Brick);
}

#[test]
fn generate_bricks_never_replaces_walls() {
    let mut state = GameState::new(9, 9);

    state.generate_bricks(&mut GameRng::seeded(3), 100);

    assert_eq!(state.get_cell(0, 0), Cell::Wall);
    assert_eq!(state.get_cell(2, 2), Cell::Wall);
}

#[test]
fn select_furthest_spawn_returns_furthest_coordinate() {
    let mut state = GameState::new(15, 15);
    let mut p1 = create_test_player(1, false, true);
    p1.sub_x = 2;
    p1.sub_y = 1;
    state.players = vec![p1];

    let candidates = vec![(1, 2), (10, 10)];

    let chosen = state.select_furthest_spawn(candidates);

    assert_eq!(chosen, Some((10, 10)));
}

#[test]
fn tick_respawns_resurrects_dead_players_when_timer_expires() {
    let mut state = GameState::new(15, 15);
    let mut player = create_test_player(1, false, false);
    player.lives = 2;
    player.respawn_timer = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
    player.active_bombs = 3;
    state.players = vec![player];

    state.tick_respawns();

    assert!(state.players[0].is_alive);
    // Bombs placed before dying are still ticking on the map.
    assert_eq!(state.players[0].active_bombs, 3);
    assert!(state.players[0].respawn_timer.is_none());
    assert!(state.players[0].death_pos.is_none());
}

#[test]
fn tick_respawns_retries_when_no_cell_is_free() {
    let mut state = GameState::new(7, 7);
    state.grid.fill(Cell::Wall);
    let mut player = create_test_player(1, false, false);
    player.lives = 2;
    player.respawn_timer = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
    state.players = vec![player];

    state.tick_respawns();

    assert!(!state.players[0].is_alive);
    assert!(state.players[0].respawn_timer.is_some());
}

#[test]
fn tick_respawns_grants_temporary_spawn_protection() {
    let mut state = GameState::new(15, 15);
    state.current_beat = 40;
    let mut player = create_test_player(1, false, false);
    player.respawn_timer = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
    state.players = vec![player];

    state.tick_respawns();

    assert!(state.players[0].is_protected(40 + RESPAWN_PROTECTION_BEATS));
    assert!(!state.players[0].is_protected(41 + RESPAWN_PROTECTION_BEATS));
}

#[test]
fn cracked_bricks_are_public_only_with_brick_hints() {
    let mut hidden = GameState::new(15, 15);
    hidden.generate_bricks(&mut GameRng::seeded(11), 100);
    let mut shown = GameState::new(15, 15);
    shown.brick_hints = true;
    shown.generate_bricks(&mut GameRng::seeded(11), 100);

    assert!(!hidden.hidden_bonuses.is_empty());
    assert!(hidden.cracked_bricks.is_empty());
    assert_eq!(shown.cracked_bricks, shown.hidden_bonuses);
    assert!(shown.hidden_bonuses.iter().all(|&i| shown.grid[i] == Cell::Brick));
}