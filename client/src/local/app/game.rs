use crate::local::app::App;
use common::game::GameAction;
use common::game::ai::{decide_bot_action, roll_bot_accuracy};
use common::game::models::MAX_BOTS;
use crossterm::event::KeyCode;
use std::time::Duration;

impl App {
    #[allow(dead_code)]
    pub(crate) fn start_game(&mut self) {
        if let Some(ref ctx) = self.game_ctx {
            let players = ctx.state.players.clone();
            self.game_ctx = Some(common::game::GameContext::new_match(
                &self.room_settings,
                players,
            ));
        }

        self.state = crate::local::app::AppState::InGame;
    }

    /// Freezes the offline game while the pause menu (or settings opened from
    /// it) is shown, so beats do not fast-forward on resume.
    pub(crate) fn sync_solo_pause(&mut self) {
        let is_paused = self.paused_from == Some(crate::local::app::AppState::InGame)
            && self.state != crate::local::app::AppState::InGame;
        if is_paused {
            self.solo_paused_at.get_or_insert_with(std::time::Instant::now);
            return;
        }
        if let Some(paused_at) = self.solo_paused_at.take() {
            if self.state == crate::local::app::AppState::InGame {
                if let Some(ref mut ctx) = self.game_ctx {
                    ctx.shift_clock(paused_at.elapsed());
                }
            }
        }
    }

    pub(crate) fn update_physics(&mut self, tick_rate: Duration, lag: &mut Duration) {
        while *lag >= tick_rate {
            *lag -= tick_rate;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_feedback_and_combo(&self) -> (&'static str, u32) {
        if let Some(ref ctx) = self.game_ctx {
            if let Some(player) = ctx.state.players.iter().find(|p| p.id == 1) {
                return (player.last_accuracy.as_str(), player.combo);
            }
        }
        ("WAITING...", 0)
    }

    pub(crate) fn is_local_player_host(&self) -> bool {
        if let Some(ref ctx) = self.game_ctx {
            if let Some(player) = ctx
                .state
                .players
                .iter()
                .find(|p| p.id == self.current_player_id)
            {
                return player.is_host;
            }
        }
        true
    }

    pub(crate) fn is_local_player_spectator(&self) -> bool {
        if let Some(ref ctx) = self.game_ctx {
            if let Some(player) = ctx
                .state
                .players
                .iter()
                .find(|p| p.id == self.current_player_id)
            {
                return player.is_spectator;
            }
        }
        false
    }

    pub(crate) fn trigger_game_action(&mut self, code: KeyCode) {
        if let Some(action) = self.map_key_to_action(code) {
            if self.network.is_multiplayer {
                if let Some(ref tx) = self.network.server_tx {
                    let _ = tx.send(common::messages::ClientMessage::Action(action.clone()));
                }
                if let Some(ref mut ctx) = self.game_ctx {
                    let progress = ctx.rhythm.progress();
                    let target_beat = if progress > 0.5 {
                        ctx.rhythm.beat_count + 1
                    } else {
                        ctx.rhythm.beat_count
                    };
                    ctx.process_player_action(self.current_player_id, action.clone());
                    self.last_local_action = Some((target_beat, action));
                }
            } else {
                if let Some(ref mut ctx) = self.game_ctx {
                    ctx.process_player_action(self.current_player_id, action);
                }
            }
        }
    }

    fn map_key_to_action(&self, code: KeyCode) -> Option<GameAction> {
        let (key_up, key_down, key_left, key_right, key_bomb, key_spell) = self.profile.keys();
        if code == KeyCode::Left || code == KeyCode::Char(key_left) {
            Some(GameAction::MoveLeft)
        } else if code == KeyCode::Right || code == KeyCode::Char(key_right) {
            Some(GameAction::MoveRight)
        } else if code == KeyCode::Up || code == KeyCode::Char(key_up) {
            Some(GameAction::MoveUp)
        } else if code == KeyCode::Down || code == KeyCode::Char(key_down) {
            Some(GameAction::MoveDown)
        } else if code == KeyCode::Char(key_bomb) || (key_bomb == ' ' && code == KeyCode::Char(' '))
        {
            Some(GameAction::PlaceBomb)
        } else if code == KeyCode::Char(key_spell) {
            Some(GameAction::TriggerSpell)
        } else if code == KeyCode::F(1) {
            Some(GameAction::Emote(1))
        } else if code == KeyCode::F(2) {
            Some(GameAction::Emote(2))
        } else if code == KeyCode::F(3) {
            Some(GameAction::Emote(3))
        } else if code == KeyCode::F(4) {
            Some(GameAction::Emote(4))
        } else {
            None
        }
    }

    pub(crate) fn setup_local_lobby_players(&mut self) {
        if let Some(ref mut ctx) = self.game_ctx {
            let me = ctx
                .state
                .players
                .iter()
                .find(|p| p.id == self.current_player_id)
                .cloned()
                .unwrap_or_else(|| {
                    let mut p = common::game::Player::new(
                        1,
                        self.profile.name.clone(),
                        self.profile.skin.clone(),
                        self.room_settings.lives,
                    );
                    p.is_host = true;
                    p
                });

            let all_skins = common::game::models::ALL_SKINS;
            let mut available_skins: Vec<String> = all_skins
                .iter()
                .filter(|&&s| s != me.skin)
                .map(|&s| s.to_string())
                .collect();

            let mut players = vec![me];
            for index in 0..self.room_settings.bot_count.min(MAX_BOTS) {
                let id = index as u32 + 2;
                let name = BOT_NAMES[index as usize % BOT_NAMES.len()];
                let skin = available_skins.pop().unwrap_or_else(|| "🤖".to_string());
                let mut bot = common::game::Player::new(id, name, skin, self.room_settings.lives);
                bot.is_ready = true;
                players.push(bot);
            }
            ctx.state.players = players;
        }
    }

    /// Lets every AI opponent act once, right on the beat.
    pub(crate) fn update_bots(&mut self) {
        let Some(ref mut ctx) = self.game_ctx else {
            return;
        };
        if ctx.state.countdown.is_some() || ctx.state.game_over_countdown.is_some() {
            return;
        }
        let difficulty = self.room_settings.bot_difficulty;
        let bot_ids: Vec<u32> = ctx
            .state
            .players
            .iter()
            .filter(|p| p.id != self.current_player_id && p.is_alive && !p.is_spectator)
            .map(|p| p.id)
            .collect();

        for bot_id in bot_ids {
            let target_beat = ctx.rhythm.beat_count;
            if let Some(action) = decide_bot_action(&ctx.state, bot_id, difficulty, &mut self.bot_rng)
            {
                let accuracy = roll_bot_accuracy(difficulty, &mut self.bot_rng);
                ctx.state
                    .handle_action(bot_id, action, accuracy, target_beat);
            }
            if self.bot_rng.chance(BOT_EMOTE_PERCENT) {
                let emote = (self.bot_rng.below(4) + 1) as u8;
                ctx.state.trigger_emote(bot_id, emote);
            }
        }
    }
}

const BOT_NAMES: [&str; 7] = [
    "Bot Alpha",
    "Bot Beta",
    "Bot Gamma",
    "Bot Delta",
    "Bot Epsilon",
    "Bot Zeta",
    "Bot Eta",
];
const BOT_EMOTE_PERCENT: u32 = 1;
