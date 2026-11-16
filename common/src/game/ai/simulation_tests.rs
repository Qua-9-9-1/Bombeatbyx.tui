//! Headless, deterministic bot-vs-bot matches checking the AI plays sensibly.

use super::{decide_bot_action, roll_bot_accuracy};
use crate::game::models::{BRICK_DENSITY_PERCENT, BotDifficulty, Cell, Player, RoomSettings};
use crate::game::spawns::GameRng;
use crate::game::state::GameState;

const SIMULATED_BEATS: u64 = 300;
const BOT_COUNT: u32 = 4;

struct SimulationStats {
    bricks_start: usize,
    bricks_left: usize,
    deaths: u32,
}

fn count_bricks(state: &GameState) -> usize {
    state.grid.iter().filter(|c| **c == Cell::Brick).count()
}

fn arena_with_bots(seed: u64) -> GameState {
    let settings = RoomSettings::default();
    let (w, h) = (settings.width, settings.height);
    let corners = [(1, 1), (w - 2, 1), (1, h - 2), (w - 2, h - 2)];
    let mut state = GameState::new(w, h);
    state.players = (1..=BOT_COUNT)
        .map(|id| {
            let mut bot = Player::new(id, format!("Bot {}", id), "🤖", 99);
            let (x, y) = corners[id as usize - 1];
            bot.sub_x = x as i32 * 2;
            bot.sub_y = y as i32;
            bot
        })
        .collect();
    state.rng = GameRng::seeded(seed);
    state.generate_bricks(&mut GameRng::seeded(seed), BRICK_DENSITY_PERCENT);
    state
}

/// Mirrors the client loop (tick, deaths, then bots act) without wall-clock
/// timers: respawns are instant and the anti-spam lockout is cleared each beat.
fn simulate(difficulty: BotDifficulty, seed: u64) -> SimulationStats {
    let mut state = arena_with_bots(seed);
    let mut rng = GameRng::seeded(seed ^ 0xABCD);
    let bricks_start = count_bricks(&state);
    let lives_start: u32 = state.players.iter().map(|p| p.lives as u32).sum();

    for beat in 1..=SIMULATED_BEATS {
        state.current_beat = beat;
        state.tick_bombs_and_explosions();
        state.check_deaths(beat);
        for bot in state.players.iter_mut().filter(|p| !p.is_alive) {
            bot.respawn_timer = Some(std::time::Instant::now());
        }
        state.tick_respawns();

        for id in 1..=BOT_COUNT {
            if let Some(bot) = state.players.iter_mut().find(|p| p.id == id) {
                bot.last_action_time = None;
                bot.spam_lockout_until = None;
            }
            if let Some(action) = decide_bot_action(&state, id, difficulty, &mut rng) {
                let accuracy = roll_bot_accuracy(difficulty, &mut rng);
                state.handle_action(id, action, accuracy, beat);
            }
        }
    }

    let lives_end: u32 = state.players.iter().map(|p| p.lives as u32).sum();
    SimulationStats {
        bricks_start,
        bricks_left: count_bricks(&state),
        deaths: lives_start - lives_end,
    }
}

#[test]
fn hard_bots_clear_most_bricks() {
    for seed in [1, 2, 3] {
        let stats = simulate(BotDifficulty::Hard, seed);

        assert!(
            stats.bricks_left * 10 < stats.bricks_start,
            "seed {}: {} of {} bricks left",
            seed,
            stats.bricks_left,
            stats.bricks_start
        );
    }
}

#[test]
fn hard_bots_rarely_die() {
    for seed in [1, 2, 3] {
        let stats = simulate(BotDifficulty::Hard, seed);

        // 4 bots over 5 minutes at 60 BPM: at most ~2 deaths each.
        assert!(stats.deaths <= 2 * BOT_COUNT, "seed {}: {} deaths", seed, stats.deaths);
    }
}

#[test]
fn easy_bots_die_more_than_hard_bots() {
    let easy: u32 = [1, 2, 3].iter().map(|&s| simulate(BotDifficulty::Easy, s).deaths).sum();
    let hard: u32 = [1, 2, 3].iter().map(|&s| simulate(BotDifficulty::Hard, s).deaths).sum();

    assert!(easy > hard, "easy {} vs hard {}", easy, hard);
}
