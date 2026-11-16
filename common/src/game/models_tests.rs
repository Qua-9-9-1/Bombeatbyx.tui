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
