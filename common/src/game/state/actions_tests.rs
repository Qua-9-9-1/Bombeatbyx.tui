use super::*;
use crate::game::models::{Player, SecondItem};
use crate::game::rhythm::BeatAccuracy;

fn create_test_player(id: u32) -> Player {
    let mut player = Player::new(id, format!("Player {}", id), "🤖", 3);
    player.sub_x = 2;
    player.sub_y = 2;
    player.combo = 5;
    player
}

#[test]
fn handle_action_resets_combo_on_miss() {
    let mut state = GameState::new(5, 5);
    state.players = vec![create_test_player(1)];

    state.handle_action(1, GameAction::MoveLeft, BeatAccuracy::Miss, 1);

    assert_eq!(state.players[0].combo, 0);
    assert_eq!(state.players[0].last_accuracy, BeatAccuracy::Miss);
}

#[test]
fn handle_action_failed_move_on_beat_keeps_groove_and_consumes_beat() {
    let mut state = GameState::new(5, 5);
    state.players = vec![create_test_player(1)];

    state.handle_action(1, GameAction::MoveLeft, BeatAccuracy::Perfect, 7);

    assert_eq!(state.players[0].combo, 6);
    assert_eq!(state.players[0].last_acted_beat, Some(7));
    assert_eq!(state.players[0].score, 0);
}

#[test]
fn handle_action_applies_combo_multiplier_to_rhythm_points() {
    let mut state = GameState::new(5, 5);
    let mut player = create_test_player(1);
    player.combo = crate::game::models::COMBO_STEP - 1;
    state.players = vec![player];

    state.handle_action(1, GameAction::MoveUp, BeatAccuracy::Perfect, 7);

    assert_eq!(state.players[0].score, BeatAccuracy::Perfect.bonus_points() * 2);
}

#[test]
fn handle_action_ignored_for_dead_player() {
    let mut state = GameState::new(5, 5);
    let mut player = create_test_player(1);
    player.is_alive = false;
    state.players = vec![player];

    state.handle_action(1, GameAction::MoveUp, BeatAccuracy::Perfect, 7);

    assert_eq!(state.players[0].last_acted_beat, None);
}

#[test]
fn reset_idle_combos_resets_players_who_skipped_the_beat() {
    let mut state = GameState::new(5, 5);
    let mut idle = create_test_player(1);
    idle.last_acted_beat = Some(6);
    let mut active = create_test_player(2);
    active.last_acted_beat = Some(7);
    state.players = vec![idle, active];

    state.reset_idle_combos(7);

    assert_eq!(state.players[0].combo, 0);
    assert_eq!(state.players[1].combo, 5);
}

#[test]
fn trigger_action_2_detonator_explodes_own_bombs_immediately() {
    let mut state = GameState::new(7, 5);
    let mut player = create_test_player(1);
    player.second_item = Some(SecondItem::Detonator);
    player.active_bombs = 1;
    state.players = vec![player];
    state.grid[3 * 7 + 5] = Cell::Bomb {
        owner_id: 1,
        ticks_left: 4,
        range: 1,
        pierce: false,
    };

    let used = state.trigger_action_2(1, 3);

    assert!(used);
    assert!(matches!(state.grid[3 * 7 + 5], Cell::Explosion { .. }));
    assert_eq!(state.players[0].active_bombs, 0);
}

#[test]
fn trigger_action_2_detonator_kept_when_no_bomb_placed() {
    let mut state = GameState::new(7, 5);
    let mut player = create_test_player(1);
    player.second_item = Some(SecondItem::Detonator);
    state.players = vec![player];

    let used = state.trigger_action_2(1, 3);

    assert!(!used);
    assert_eq!(state.players[0].second_item, Some(SecondItem::Detonator));
}

#[test]
fn trigger_action_2_dash_moves_several_cells_in_facing_direction() {
    let mut state = GameState::new(11, 5);
    let mut player = create_test_player(1);
    player.sub_x = 2;
    player.sub_y = 1;
    player.facing = crate::game::models::Direction::Right;
    player.second_item = Some(SecondItem::Dash);
    state.players = vec![player];

    let used = state.trigger_action_2(1, 3);

    assert!(used);
    assert_eq!(state.players[0].sub_x, 8);
}

#[test]
fn tick_emotes_clears_expired_emote() {
    let mut state = GameState::new(5, 5);
    let mut player = create_test_player(1);
    player.active_emote = Some("👋".to_string());
    player.emote_until = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
    state.players = vec![player];

    state.tick_emotes();

    assert!(state.players[0].active_emote.is_none());
}

#[test]
fn trigger_action_2_activates_shield_when_equipped() {
    let mut state = GameState::new(5, 5);
    let mut player = create_test_player(1);
    player.second_item = Some(SecondItem::Shield);
    state.players = vec![player];

    let success = state.trigger_action_2(1, 10);

    assert!(success);
    assert_eq!(
        state.players[0].shield_until_beat,
        Some(10 + SHIELD_DURATION_BEATS)
    );
    assert!(state.players[0].second_item.is_none());
}
