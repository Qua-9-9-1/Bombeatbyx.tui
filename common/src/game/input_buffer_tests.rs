use super::*;
use crate::game::models::Player;
use crate::game::rhythm::BeatAccuracy;
use std::time::Instant;

const BEAT: u64 = 8;

/// Player standing on cell (1, 1), free to move down; clock at `progress` of beat 8.
fn context_at(progress: f64) -> GameContext {
    let mut ctx = GameContext::new(5, 5, 60.0);
    let mut player = Player::new(1, "P1", "🤖", 3);
    player.sub_x = 2;
    player.sub_y = 1;
    player.combo = 5;
    ctx.state.players = vec![player];
    set_progress(&mut ctx, progress);
    ctx
}

fn set_progress(ctx: &mut GameContext, progress: f64) {
    let interval = ctx.rhythm.beat_interval;
    ctx.rhythm.beat_count = BEAT;
    ctx.rhythm.last_beat_time = Instant::now() - interval.mul_f64(progress);
    ctx.rhythm.next_beat_time = ctx.rhythm.last_beat_time + interval;
}

#[test]
fn off_beat_input_waits_for_next_beat() {
    let mut ctx = context_at(0.5);

    ctx.process_player_action(1, GameAction::MoveDown);

    assert!(ctx.has_buffered_action(1));
    assert_eq!(ctx.state.players[0].sub_y, 1);
}

#[test]
fn buffered_input_runs_on_beat_without_combo() {
    let mut ctx = context_at(0.5);
    ctx.process_player_action(1, GameAction::MoveDown);

    ctx.flush_buffered_actions(BEAT + 1);

    let player = &ctx.state.players[0];
    assert_eq!(player.sub_y, 2);
    assert_eq!(player.combo, 0);
    assert_eq!(player.score, 0);
    assert_eq!(player.last_acted_beat, Some(BEAT + 1));
    assert_eq!(player.last_accuracy, BeatAccuracy::Miss);
    assert!(!ctx.has_buffered_action(1));
}

#[test]
fn buffered_input_is_not_run_before_its_beat() {
    let mut ctx = context_at(0.5);
    ctx.process_player_action(1, GameAction::MoveDown);

    ctx.flush_buffered_actions(BEAT);

    assert!(ctx.has_buffered_action(1));
    assert_eq!(ctx.state.players[0].sub_y, 1);
}

#[test]
fn only_first_off_beat_input_is_kept() {
    let mut ctx = context_at(0.4);
    ctx.process_player_action(1, GameAction::MoveDown);
    ctx.state.players[0].last_action_time = None;

    ctx.process_player_action(1, GameAction::PlaceBomb);

    let buffered = ctx.buffered_actions[&1];
    assert_eq!(buffered.action, GameAction::MoveDown);
}

#[test]
fn on_beat_input_is_ignored_while_slot_is_full() {
    let mut ctx = context_at(0.5);
    ctx.process_player_action(1, GameAction::MoveDown);
    ctx.state.players[0].last_action_time = None;
    set_progress(&mut ctx, 0.97);

    ctx.process_player_action(1, GameAction::PlaceBomb);

    assert_eq!(ctx.state.players[0].active_bombs, 0);
    assert_eq!(ctx.buffered_actions[&1].action, GameAction::MoveDown);
}

#[test]
fn on_beat_input_runs_immediately_and_keeps_combo() {
    let mut ctx = context_at(0.02);

    ctx.process_player_action(1, GameAction::MoveDown);

    let player = &ctx.state.players[0];
    assert_eq!(player.sub_y, 2);
    assert_eq!(player.combo, 6);
    assert!(!ctx.has_buffered_action(1));
}

#[test]
fn dead_player_buffered_input_is_dropped() {
    let mut ctx = context_at(0.5);
    ctx.process_player_action(1, GameAction::MoveDown);
    ctx.state.players[0].is_alive = false;

    ctx.flush_buffered_actions(BEAT + 1);

    assert_eq!(ctx.state.players[0].sub_y, 1);
    assert!(!ctx.has_buffered_action(1));
}
