use crate::local::app::App;
use common::game::GameAction;
use crossterm::event::KeyCode;
use std::time::Duration;

impl App {
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
                p.second_item = if p.id == 2 { Some(common::game::models::SecondItem::Shield) } else { None };
                p.shield_until_beat = None;
                p.combo = 0;
            }

            let mut new_state = common::game::GameState::new(
                self.room_settings.width,
                self.room_settings.height,
            );
            new_state.bpm = self.room_settings.bpm;
            new_state.sudden_death = self.room_settings.sudden_death;
            new_state.bonus_every = self.room_settings.bonus_every;
            new_state.mode = self.room_settings.mode;

            new_state.spawn_players(players);

            ctx.state = new_state;
            ctx.rhythm = common::game::RhythmEngine::new(self.room_settings.bpm);
            ctx.last_closed_window_beat = None;
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
        } else if code == KeyCode::Char('1') {
            Some(GameAction::Emote(1))
        } else if code == KeyCode::Char('2') {
            Some(GameAction::Emote(2))
        } else if code == KeyCode::Char('3') {
            Some(GameAction::Emote(3))
        } else if code == KeyCode::Char('4') {
            Some(GameAction::Emote(4))
        } else {
            None
        }
    }
}
