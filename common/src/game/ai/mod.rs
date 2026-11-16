//! Bot brain: pure functions that pick one action per beat from a `GameState`.

pub mod danger;
mod goals;
pub mod pathfinding;
#[cfg(test)]
mod simulation_tests;

use crate::game::actions::GameAction;
use crate::game::models::{BotDifficulty, Direction, Player, SecondItem};
use crate::game::rhythm::BeatAccuracy;
use crate::game::spawns::GameRng;
use crate::game::state::GameState;
use danger::DangerMap;
use pathfinding::find_path;

/// Tuning knobs for each difficulty level (percentages).
pub struct BotProfile {
    /// Chance to skip a beat while safe.
    pub idle_percent: u32,
    /// Chance to freeze while standing in a blast zone.
    pub panic_percent: u32,
    pub wander_percent: u32,
    pub hunts_players: bool,
    pub uses_items: bool,
    /// Weights for (Perfect, Great, Ok, Miss).
    pub accuracy_weights: [u32; 4],
}

pub fn profile(difficulty: BotDifficulty) -> BotProfile {
    match difficulty {
        BotDifficulty::Easy => BotProfile {
            idle_percent: 30,
            panic_percent: 12,
            wander_percent: 25,
            hunts_players: false,
            uses_items: false,
            accuracy_weights: [20, 40, 30, 10],
        },
        BotDifficulty::Normal => BotProfile {
            idle_percent: 10,
            panic_percent: 2,
            wander_percent: 8,
            hunts_players: true,
            uses_items: true,
            accuracy_weights: [40, 40, 15, 5],
        },
        BotDifficulty::Hard => BotProfile {
            idle_percent: 2,
            panic_percent: 0,
            wander_percent: 0,
            hunts_players: true,
            uses_items: true,
            accuracy_weights: [70, 25, 5, 0],
        },
    }
}

pub fn roll_bot_accuracy(difficulty: BotDifficulty, rng: &mut GameRng) -> BeatAccuracy {
    let weights = profile(difficulty).accuracy_weights;
    let total: u32 = weights.iter().sum();
    let mut pick = rng.next_u32() % total.max(1);
    let outcomes = [
        BeatAccuracy::Perfect,
        BeatAccuracy::Great,
        BeatAccuracy::Ok,
        BeatAccuracy::Miss,
    ];
    for (outcome, weight) in outcomes.into_iter().zip(weights) {
        if pick < weight {
            return outcome;
        }
        pick -= weight;
    }
    BeatAccuracy::Ok
}

/// Chooses the bot's action for the current beat (`None` = skip the beat).
pub fn decide_bot_action(
    state: &GameState,
    bot_id: u32,
    difficulty: BotDifficulty,
    rng: &mut GameRng,
) -> Option<GameAction> {
    let bot = state
        .players
        .iter()
        .find(|p| p.id == bot_id && p.is_alive && !p.is_spectator)?;
    let profile = profile(difficulty);
    let position = ((bot.sub_x / 2) as usize, bot.sub_y as usize);
    let danger = DangerMap::compute(state);

    if let Some(timer) = danger.timer(position.0, position.1) {
        if rng.chance(profile.panic_percent) {
            return None;
        }
        return escape(state, bot, position, &danger, timer);
    }
    if rng.chance(profile.idle_percent) {
        return None;
    }
    if profile.uses_items {
        if let Some(action) = goals::use_offensive_item(state, bot, position) {
            return Some(action);
        }
    }
    if goals::should_place_bomb(state, bot, position, &profile) {
        return Some(GameAction::PlaceBomb);
    }
    if rng.chance(profile.wander_percent) {
        return random_safe_move(state, bot_id, position, &danger, rng);
    }
    goals::seek_goal(state, bot, position, &danger, &profile).map(move_action)
}

fn escape(
    state: &GameState,
    bot: &Player,
    position: (usize, usize),
    danger: &DangerMap,
    timer: u8,
) -> Option<GameAction> {
    let has_shield = bot.second_item == Some(SecondItem::Shield);
    if has_shield && timer <= 1 {
        return Some(GameAction::TriggerSpell);
    }
    let path = find_path(
        state,
        bot.id,
        position,
        timer.max(1) as u32 + 2,
        |x, y, step| danger.can_stand_at_step(x, y, step),
        |x, y| danger.is_safe(x, y),
    );
    match path {
        Some(step) => Some(move_action(step.first_move)),
        None if has_shield => Some(GameAction::TriggerSpell),
        None => None,
    }
}

fn random_safe_move(
    state: &GameState,
    bot_id: u32,
    position: (usize, usize),
    danger: &DangerMap,
    rng: &mut GameRng,
) -> Option<GameAction> {
    let options: Vec<Direction> = pathfinding::DIRECTIONS
        .into_iter()
        .filter(|direction| {
            let (dx, dy) = direction.delta();
            let (x, y) = (position.0 as i32 + dx, position.1 as i32 + dy);
            pathfinding::is_walkable(state, x, y, bot_id) && danger.is_safe(x as usize, y as usize)
        })
        .collect();
    if options.is_empty() {
        return None;
    }
    Some(move_action(options[rng.below(options.len())]))
}

pub fn move_action(direction: Direction) -> GameAction {
    match direction {
        Direction::Up => GameAction::MoveUp,
        Direction::Down => GameAction::MoveDown,
        Direction::Left => GameAction::MoveLeft,
        Direction::Right => GameAction::MoveRight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::models::Cell;

    fn bot_at(x: i32, y: i32) -> Player {
        let mut bot = Player::new(1, "Bot", "🤖", 3);
        bot.sub_x = x * 2;
        bot.sub_y = y;
        bot
    }

    #[test]
    fn decide_bot_action_flees_from_bomb_blast() {
        let mut state = GameState::new(9, 5);
        state.players = vec![bot_at(1, 1)];
        state.grid[1 * 9 + 1] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 3,
            range: 2,
            pierce: false,
        };

        let action = decide_bot_action(&state, 1, BotDifficulty::Hard, &mut GameRng::seeded(1));

        assert!(matches!(
            action,
            Some(GameAction::MoveRight) | Some(GameAction::MoveDown)
        ));
    }

    #[test]
    fn decide_bot_action_ignores_dead_bot() {
        let mut state = GameState::new(9, 5);
        let mut bot = bot_at(1, 1);
        bot.is_alive = false;
        state.players = vec![bot];

        let action = decide_bot_action(&state, 1, BotDifficulty::Hard, &mut GameRng::seeded(1));

        assert_eq!(action, None);
    }

    #[test]
    fn roll_bot_accuracy_hard_never_misses() {
        let mut rng = GameRng::seeded(42);

        let has_missed = (0..500)
            .any(|_| roll_bot_accuracy(BotDifficulty::Hard, &mut rng) == BeatAccuracy::Miss);

        assert!(!has_missed);
    }
}
