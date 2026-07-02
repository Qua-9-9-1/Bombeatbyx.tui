use crate::local::app::App;
use common::game::{BeatAccuracy, GameAction, GameContext, Player};
use crossterm::event::KeyCode;
use std::time::Duration;

impl App {
    pub(crate) fn start_game(&mut self) {
        let mut ctx = GameContext::new(
            self.room_settings.width,
            self.room_settings.height,
            self.room_settings.bpm,
        );

        ctx.state.players.push(Player {
            id: self.current_player_id,
            is_host: true,
            name: self.profile.name.clone(),
            skin: self.profile.skin.clone(),
            color: "cyan".to_string(),
            sub_x: 2,
            sub_y: 2,
            is_alive: true,
            score: 0,
            combo: 0,
            last_acted_beat: None,
            last_accuracy: BeatAccuracy::Waiting,
            max_bombs: 1,
            active_bombs: 0,
            bomb_range: 1,
            last_action_time: None,
            spam_lockout_until: None,
        });

        self.game_ctx = Some(ctx);
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
                if player.last_acted_beat == Some(ctx.rhythm.beat_count) {
                    return (player.last_accuracy.as_str(), player.combo);
                }
                return ("WAITING...", player.combo);
            }
        }
        ("WAITING...", 0)
    }

    pub(crate) fn is_local_player_host(&self) -> bool {
        if let Some(ref ctx) = self.game_ctx {
            if let Some(player) = ctx.state.players.iter().find(|p| p.id == self.current_player_id) {
                return player.is_host;
            }
        }
        true
    }

    pub(crate) fn trigger_game_action(&mut self, code: KeyCode) {
        if let Some(action) = self.map_key_to_action(code) {
            if let Some(ref mut ctx) = self.game_ctx {
                ctx.process_player_action(self.current_player_id, action);
            }
        }
    }

    fn map_key_to_action(&self, code: KeyCode) -> Option<GameAction> {
        if code == KeyCode::Left || code == KeyCode::Char(self.profile.key_left) {
            Some(GameAction::MoveLeft)
        } else if code == KeyCode::Right || code == KeyCode::Char(self.profile.key_right) {
            Some(GameAction::MoveRight)
        } else if code == KeyCode::Up || code == KeyCode::Char(self.profile.key_up) {
            Some(GameAction::MoveUp)
        } else if code == KeyCode::Down || code == KeyCode::Char(self.profile.key_down) {
            Some(GameAction::MoveDown)
        } else if code == KeyCode::Char(' ') {
            Some(GameAction::PlaceBomb)
        } else if code == KeyCode::Char('e') {
            Some(GameAction::TriggerSpell)
        } else {
            None
        }
    }
}
