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

        let players = vec![
            Player {
                id: self.current_player_id,
                is_host: true,
                name: self.profile.name.clone(),
                skin: self.profile.skin.clone(),
                color: "green".to_string(),
                sub_x: 0,
                sub_y: 0,
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
            },
            // Mocked players
            Player {
                id: 2,
                is_host: false,
                name: "GigaPlayer".to_string(),
                skin: "🐱".to_string(),
                color: "magenta".to_string(),
                sub_x: 0,
                sub_y: 0,
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
            },
            Player {
                id: 3,
                is_host: false,
                name: "Ribbit".to_string(),
                skin: "🐸".to_string(),
                color: "yellow".to_string(),
                sub_x: 0,
                sub_y: 0,
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
            },
            Player {
                id: 4,
                is_host: false,
                name: "Chad".to_string(),
                skin: "😎".to_string(),
                color: "blue".to_string(),
                sub_x: 0,
                sub_y: 0,
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
            },
            Player {
                id: 5,
                is_host: false,
                name: "NoobMaster69".to_string(),
                skin: "👾".to_string(),
                color: "red".to_string(),
                sub_x: 0,
                sub_y: 0,
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
            },
            Player {
                id: 6,
                is_host: false,
                name: "NoLifeGuy".to_string(),
                skin: "🧴".to_string(),
                color: "cyan".to_string(),
                sub_x: 0,
                sub_y: 0,
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
            },
            Player {
                id: 7,
                is_host: false,
                name: "PixelPanda".to_string(),
                skin: "🐼".to_string(),
                color: "green".to_string(),
                sub_x: 0,
                sub_y: 0,
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
            },
        ];

        ctx.state.spawn_players(players);

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
