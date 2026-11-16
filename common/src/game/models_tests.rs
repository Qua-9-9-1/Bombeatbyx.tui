use super::*;

#[test]
fn combo_multiplier_grows_every_step_and_is_capped() {
    let mut player = Player::new(1, "Player 1", "🤖", 3);

    let multipliers: Vec<u32> = [0, COMBO_STEP, COMBO_STEP * 2, COMBO_STEP * 50]
        .iter()
        .map(|&combo| {
            player.combo = combo;
            player.combo_multiplier()
        })
        .collect();

    assert_eq!(multipliers, vec![1, 2, 3, MAX_COMBO_MULTIPLIER]);
}

#[test]
fn effective_bomb_range_adds_fever_bonus() {
    let mut player = Player::new(1, "Player 1", "🤖", 3);
    player.bomb_range = 2;
    player.combo = FEVER_COMBO;

    let range = player.effective_bomb_range();

    assert_eq!(range, 2 + FEVER_BONUS_RANGE);
}

#[test]
fn try_consume_action_lockout_allows_action_when_no_previous_action() {
    let mut player = Player::new(1, "Player 1", "🤖", 3);

    let allowed = player.try_consume_action_lockout();

    assert!(allowed);
    assert!(player.last_action_time.is_some());
}

#[test]
fn try_consume_action_lockout_blocks_action_when_spam_lockout_active() {
    let mut player = Player::new(1, "Player 1", "🤖", 3);
    player.last_action_time = Some(Instant::now());
    player.spam_lockout_until = Some(Instant::now() + Duration::from_millis(500));

    let allowed = player.try_consume_action_lockout();

    assert!(!allowed);
}

#[test]
fn sanitized_clamps_dangerous_values_into_valid_ranges() {
    let hostile = RoomSettings {
        width: 1,
        height: 10_000,
        bpm: 0.0,
        bonus_every: 0,
        lives: 0,
        target_score: 0,
        time_limit_mins: Some(0),
        bot_count: 200,
        ..RoomSettings::default()
    };

    let safe = hostile.sanitized();

    assert_eq!(safe.width, *MAP_SIZE_RANGE.start());
    assert_eq!(safe.height, *MAP_SIZE_RANGE.end());
    assert_eq!(safe.bpm, *BPM_RANGE.start());
    assert_eq!(safe.bonus_every, 1);
    assert_eq!(safe.lives, 1);
    assert_eq!(safe.time_limit_mins, Some(1));
    assert_eq!(safe.bot_count, MAX_BOTS);
}

#[test]
fn sanitized_makes_map_size_odd_and_replaces_nan_bpm() {
    let settings = RoomSettings {
        width: 16,
        bpm: f64::NAN,
        ..RoomSettings::default()
    };

    let safe = settings.sanitized();

    assert_eq!(safe.width, 15);
    assert_eq!(safe.bpm, RoomSettings::default().bpm);
}