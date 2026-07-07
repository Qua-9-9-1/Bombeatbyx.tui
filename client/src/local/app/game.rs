use crate::local::app::App;
use common::game::GameAction;
use crossterm::event::KeyCode;
use std::time::Duration;

impl App {
    #[allow(dead_code)]
    pub(crate) fn start_game(&mut self) {
        if let Some(ref mut ctx) = self.game_ctx {
            let mut players = ctx.state.players.clone();
            for p in &mut players {
                p.lives = self.room_settings.lives;
                p.is_alive = !p.is_spectator;
                p.death_pos = None;
                p.respawn_timer = None;
                p.active_bombs = 0;
                p.max_bombs = 1;
                p.bomb_range = 1;
                p.collected_bonuses.clear();
                p.second_item = if p.id == 2 {
                    Some(common::game::models::SecondItem::Shield)
                } else {
                    None
                };
                p.shield_until_beat = None;
                p.combo = 0;
            }

            let mut new_state =
                common::game::GameState::new(self.room_settings.width, self.room_settings.height);
            new_state.bpm = self.room_settings.bpm;
            new_state.sudden_death = self.room_settings.sudden_death;
            new_state.bonus_every = self.room_settings.bonus_every;
            new_state.mode = self.room_settings.mode;
            new_state.target_score = self.room_settings.target_score;
            new_state.time_limit_mins = self.room_settings.time_limit_mins;

            new_state.spawn_players(players);

            ctx.state = new_state;
            ctx.rhythm = common::game::RhythmEngine::new(self.room_settings.bpm);
            ctx.last_closed_window_beat = None;
            ctx.start_time = std::time::Instant::now();
        }

        self.state = crate::local::app::AppState::InGame;
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
                .unwrap_or_else(|| common::game::Player {
                    id: 1,
                    is_host: true,
                    name: self.profile.name.clone(),
                    skin: self.profile.skin.clone(),
                    sub_x: 0,
                    sub_y: 0,
                    is_alive: true,
                    score: 0,
                    combo: 0,
                    last_acted_beat: None,
                    last_accuracy: common::game::BeatAccuracy::Waiting,
                    max_bombs: 1,
                    active_bombs: 0,
                    bomb_range: 1,
                    last_action_time: None,
                    spam_lockout_until: None,
                    active_emote: None,
                    emote_until: None,
                    lives: self.room_settings.lives,
                    death_pos: None,
                    respawn_timer: None,
                    collected_bonuses: Vec::new(),
                    is_spectator: false,
                    second_item: None,
                    shield_until_beat: None,
                    is_ready: false,
                    death_beat: None,
                });

            let all_skins = common::game::models::ALL_SKINS;
            let mut available_skins: Vec<String> = all_skins
                .iter()
                .filter(|&&s| s != me.skin)
                .map(|&s| s.to_string())
                .collect();

            let skin2 = available_skins.pop().unwrap_or_else(|| "🦊".to_string());
            let skin3 = available_skins.pop().unwrap_or_else(|| "🐧".to_string());
            let skin4 = available_skins.pop().unwrap_or_else(|| "🐸".to_string());

            ctx.state.players = vec![
                me,
                common::game::Player {
                    id: 2,
                    is_host: false,
                    name: "Bot Alpha 🦊".to_string(),
                    skin: skin2,
                    sub_x: 0,
                    sub_y: 0,
                    is_alive: true,
                    score: 0,
                    combo: 0,
                    last_acted_beat: None,
                    last_accuracy: common::game::BeatAccuracy::Waiting,
                    max_bombs: 1,
                    active_bombs: 0,
                    bomb_range: 1,
                    last_action_time: None,
                    spam_lockout_until: None,
                    active_emote: None,
                    emote_until: None,
                    lives: self.room_settings.lives,
                    death_pos: None,
                    respawn_timer: None,
                    collected_bonuses: Vec::new(),
                    is_spectator: false,
                    second_item: None,
                    shield_until_beat: None,
                    is_ready: true,
                    death_beat: None,
                },
                common::game::Player {
                    id: 3,
                    is_host: false,
                    name: "Bot Beta 🐧".to_string(),
                    skin: skin3,
                    sub_x: 0,
                    sub_y: 0,
                    is_alive: true,
                    score: 0,
                    combo: 0,
                    last_acted_beat: None,
                    last_accuracy: common::game::BeatAccuracy::Waiting,
                    max_bombs: 1,
                    active_bombs: 0,
                    bomb_range: 1,
                    last_action_time: None,
                    spam_lockout_until: None,
                    active_emote: None,
                    emote_until: None,
                    lives: self.room_settings.lives,
                    death_pos: None,
                    respawn_timer: None,
                    collected_bonuses: Vec::new(),
                    is_spectator: false,
                    second_item: None,
                    shield_until_beat: None,
                    is_ready: true,
                    death_beat: None,
                },
                common::game::Player {
                    id: 4,
                    is_host: false,
                    name: "Bot Gamma 🐸".to_string(),
                    skin: skin4,
                    sub_x: 0,
                    sub_y: 0,
                    is_alive: true,
                    score: 0,
                    combo: 0,
                    last_acted_beat: None,
                    last_accuracy: common::game::BeatAccuracy::Waiting,
                    max_bombs: 1,
                    active_bombs: 0,
                    bomb_range: 1,
                    last_action_time: None,
                    spam_lockout_until: None,
                    active_emote: None,
                    emote_until: None,
                    lives: self.room_settings.lives,
                    death_pos: None,
                    respawn_timer: None,
                    collected_bonuses: Vec::new(),
                    is_spectator: false,
                    second_item: None,
                    shield_until_beat: None,
                    is_ready: true,
                    death_beat: None,
                },
            ];
        }
    }

    pub(crate) fn update_bots(&mut self) {
        if let Some(ref mut ctx) = self.game_ctx {
            let bot_ids: Vec<u32> = ctx
                .state
                .players
                .iter()
                .filter(|p| p.id != self.current_player_id && p.is_alive && !p.is_spectator)
                .map(|p| p.id)
                .collect();

            for bot_id in bot_ids {
                let seed = common::game::spawns::get_pseudo_random_u32();
                if seed % 100 > 60 {
                    continue;
                }

                let action_roll = (seed / 100) % 100;
                let action = if action_roll < 15 {
                    common::game::GameAction::PlaceBomb
                } else if action_roll < 20 {
                    common::game::GameAction::TriggerSpell
                } else if action_roll < 25 {
                    let emote_id = ((seed / 10000) % 4 + 1) as u8;
                    common::game::GameAction::Emote(emote_id)
                } else {
                    let dir_roll = (seed / 10000) % 4;
                    match dir_roll {
                        0 => common::game::GameAction::MoveLeft,
                        1 => common::game::GameAction::MoveRight,
                        2 => common::game::GameAction::MoveUp,
                        _ => common::game::GameAction::MoveDown,
                    }
                };

                let accuracy = match seed % 3 {
                    0 => common::game::BeatAccuracy::Perfect,
                    1 => common::game::BeatAccuracy::Great,
                    _ => common::game::BeatAccuracy::Ok,
                };

                let target_beat = ctx.rhythm.beat_count;

                ctx.state
                    .handle_action(bot_id, action, accuracy, target_beat);
            }
        }
    }
}
