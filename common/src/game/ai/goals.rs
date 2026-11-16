use super::BotProfile;
use super::danger::{DangerMap, HypotheticalBomb};
use super::pathfinding::{find_path, PathStep};
use crate::game::actions::GameAction;
use crate::game::models::{BOMB_FUSE_BEATS, Cell, Direction, Player, SecondItem};
use crate::game::state::GameState;

/// How far (in moves) a bot looks for bonuses, bricks or enemies.
const SEARCH_DEPTH: u32 = 12;
/// A bot moves towards enemies until it is this close (Manhattan distance).
const CHASE_DISTANCE: i32 = 2;

fn enemy_positions(state: &GameState, bot_id: u32) -> Vec<(i32, i32)> {
    state
        .players
        .iter()
        .filter(|p| p.id != bot_id && p.is_alive && !p.is_spectator)
        .map(|p| (p.sub_x / 2, p.sub_y))
        .collect()
}

/// What a bomb placed at (x, y) would hit: (bricks, enemies).
fn blast_targets(state: &GameState, bot: &Player, x: usize, y: usize) -> (usize, usize) {
    let enemies = enemy_positions(state, bot.id);
    let range = bot.effective_bomb_range() as i32;
    let (mut bricks, mut hit_enemies) = (0, 0);
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        for i in 1..=range {
            let (tx, ty) = (x as i32 + dx * i, y as i32 + dy * i);
            if enemies.contains(&(tx, ty)) {
                hit_enemies += 1;
            }
            match state.get_cell(tx, ty) {
                Cell::Wall | Cell::Bomb { .. } => break,
                Cell::Brick => {
                    bricks += 1;
                    if !bot.pierce {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    (bricks, hit_enemies)
}

/// Uses the detonator when one of the bot's bombs would catch an enemy.
pub fn use_offensive_item(
    state: &GameState,
    bot: &Player,
    position: (usize, usize),
) -> Option<GameAction> {
    if bot.second_item != Some(SecondItem::Detonator) || bot.active_bombs == 0 {
        return None;
    }
    let mut own_bombs_only = state.clone();
    for cell in own_bombs_only.grid.iter_mut() {
        if let Cell::Bomb {
            owner_id,
            ticks_left,
            ..
        } = cell
        {
            if *owner_id == bot.id {
                *ticks_left = 0;
            }
        }
    }
    let danger = DangerMap::compute(&own_bombs_only);
    let catches_enemy = enemy_positions(state, bot.id)
        .iter()
        .any(|&(x, y)| danger.timer(x as usize, y as usize) == Some(0));
    let is_bot_safe = danger.timer(position.0, position.1) != Some(0);
    (catches_enemy && is_bot_safe).then_some(GameAction::TriggerSpell)
}

/// Places a bomb only if it hits something and an escape route exists.
pub fn should_place_bomb(
    state: &GameState,
    bot: &Player,
    position: (usize, usize),
    profile: &BotProfile,
) -> bool {
    if bot.active_bombs >= bot.max_bombs {
        return false;
    }
    if state.get_cell(position.0 as i32, position.1 as i32) != Cell::Empty {
        return false;
    }
    let (bricks, enemies) = blast_targets(state, bot, position.0, position.1);
    let is_worth_it = bricks > 0 || (profile.hunts_players && enemies > 0);
    is_worth_it && has_escape_after_bomb(state, bot, position)
}

fn has_escape_after_bomb(state: &GameState, bot: &Player, position: (usize, usize)) -> bool {
    let bomb = HypotheticalBomb {
        x: position.0,
        y: position.1,
        timer: BOMB_FUSE_BEATS,
        range: bot.effective_bomb_range().min(u8::MAX as usize) as u8,
        pierce: bot.pierce,
    };
    // Moves start one beat after placing the bomb.
    let simulated = DangerMap::compute_with(state, Some(bomb)).shifted(1);
    let moves_available = BOMB_FUSE_BEATS as u32 - 1;
    // Enemies can block a corridor or drop a bomb in it: avoid relying on
    // escape routes that brush past them.
    let enemies = enemy_positions(state, bot.id);
    let is_near_enemy = |x: usize, y: usize| {
        enemies
            .iter()
            .any(|&(ex, ey)| (ex - x as i32).abs() + (ey - y as i32).abs() <= 1)
    };
    find_path(
        state,
        bot.id,
        position,
        moves_available,
        |x, y, step| simulated.can_stand_at_step(x, y, step) && !is_near_enemy(x, y),
        |x, y| simulated.is_safe(x, y),
    )
    .is_some()
}

/// Picks the next move towards the most interesting reachable target.
pub fn seek_goal(
    state: &GameState,
    bot: &Player,
    position: (usize, usize),
    danger: &DangerMap,
    profile: &BotProfile,
) -> Option<Direction> {
    let safe_path = |is_goal: &dyn Fn(usize, usize) -> bool| -> Option<PathStep> {
        find_path(
            state,
            bot.id,
            position,
            SEARCH_DEPTH,
            |x, y, _| danger.is_safe(x, y),
            is_goal,
        )
    };

    let is_bonus = |x: usize, y: usize| matches!(state.get_cell(x as i32, y as i32), Cell::Bonus { .. });
    if let Some(step) = safe_path(&is_bonus) {
        return Some(step.first_move);
    }

    let can_bomb = bot.active_bombs < bot.max_bombs;
    if can_bomb && profile.hunts_players {
        let hits_enemy = |x: usize, y: usize| blast_targets(state, bot, x, y).1 > 0;
        if let Some(step) = safe_path(&hits_enemy) {
            return Some(step.first_move);
        }
    }
    if can_bomb {
        let hits_brick = |x: usize, y: usize| blast_targets(state, bot, x, y).0 > 0;
        if let Some(step) = safe_path(&hits_brick) {
            return Some(step.first_move);
        }
    }

    let enemies = enemy_positions(state, bot.id);
    let near_enemy = |x: usize, y: usize| {
        enemies
            .iter()
            .any(|&(ex, ey)| (ex - x as i32).abs() + (ey - y as i32).abs() <= CHASE_DISTANCE)
    };
    safe_path(&near_enemy).map(|step| step.first_move)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ai::profile;
    use crate::game::models::BotDifficulty;

    fn bot_at(id: u32, x: i32, y: i32) -> Player {
        let mut bot = Player::new(id, "Bot", "🤖", 3);
        bot.sub_x = x * 2;
        bot.sub_y = y;
        bot
    }

    #[test]
    fn should_place_bomb_true_next_to_brick_with_escape() {
        let mut state = GameState::new(9, 5);
        state.grid[1 * 9 + 2] = Cell::Brick;
        let bot = bot_at(1, 1, 1);
        state.players = vec![bot.clone()];

        let decision = should_place_bomb(&state, &bot, (1, 1), &profile(BotDifficulty::Normal));

        assert!(decision);
    }

    #[test]
    fn should_place_bomb_false_when_trapped_in_dead_end() {
        let mut state = GameState::new(9, 5);
        state.grid[1 * 9 + 2] = Cell::Brick;
        state.grid[2 * 9 + 1] = Cell::Brick;
        let bot = bot_at(1, 1, 1);
        state.players = vec![bot.clone()];

        let decision = should_place_bomb(&state, &bot, (1, 1), &profile(BotDifficulty::Normal));

        assert!(!decision);
    }

    #[test]
    fn should_place_bomb_false_when_nothing_to_hit() {
        let state = GameState::new(9, 5);
        let bot = bot_at(1, 1, 1);

        let decision = should_place_bomb(&state, &bot, (1, 1), &profile(BotDifficulty::Hard));

        assert!(!decision);
    }

    #[test]
    fn seek_goal_moves_towards_bonus() {
        let mut state = GameState::new(9, 3);
        state.grid[1 * 9 + 5] = Cell::Bonus {
            bonus_type: crate::game::models::BonusType::BombQty,
            ticks_left: 10,
        };
        let bot = bot_at(1, 1, 1);
        state.players = vec![bot.clone()];
        let danger = DangerMap::compute(&state);

        let direction = seek_goal(&state, &bot, (1, 1), &danger, &profile(BotDifficulty::Easy));

        assert_eq!(direction, Some(Direction::Right));
    }
}
