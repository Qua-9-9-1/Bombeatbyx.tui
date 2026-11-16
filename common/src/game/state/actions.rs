use super::GameState;
use crate::game::actions::GameAction;
use crate::game::models::{Cell, SHIELD_DURATION_BEATS, SecondItem};
use crate::game::rhythm::BeatAccuracy;

impl GameState {
    pub fn handle_action(
        &mut self,
        player_id: u32,
        action: GameAction,
        accuracy: BeatAccuracy,
        current_beat: u64,
    ) {
        if self.countdown.is_some() || self.game_over_countdown.is_some() {
            return;
        }

        if let GameAction::Emote(index) = action {
            self.trigger_emote(player_id, index);
            return;
        }

        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if !player.try_consume_action_lockout() {
                return;
            }

            if let Some(last_beat) = player.last_acted_beat {
                if last_beat == current_beat {
                    return;
                }
            }

            if matches!(accuracy, BeatAccuracy::Miss) {
                player.last_acted_beat = Some(current_beat);
                player.last_accuracy = BeatAccuracy::Miss;
                player.combo = 0;
                return;
            }

            let success =
                self.apply_player_action(player_id, action, accuracy.clone(), current_beat);
            if success {
                if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
                    p.last_acted_beat = Some(current_beat);
                    p.last_accuracy = accuracy.clone();
                    if accuracy != BeatAccuracy::Ok {
                        p.combo = (p.combo + 1).min(9999);
                    }
                    p.score += accuracy.bonus_points();
                }
            }
        }
    }

    fn apply_player_action(
        &mut self,
        player_id: u32,
        action: GameAction,
        accuracy: BeatAccuracy,
        current_beat: u64,
    ) -> bool {
        match action {
            GameAction::MoveLeft => self.move_player(player_id, -2, 0),
            GameAction::MoveRight => self.move_player(player_id, 2, 0),
            GameAction::MoveUp => self.move_player(player_id, 0, -1),
            GameAction::MoveDown => self.move_player(player_id, 0, 1),
            GameAction::PlaceBomb => self.try_place_bomb(player_id, accuracy),
            GameAction::TriggerSpell => self.trigger_action_2(player_id, current_beat),
            GameAction::Emote(_) => false,
        }
    }

    pub fn trigger_action_2(&mut self, player_id: u32, current_beat: u64) -> bool {
        let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) else {
            return false;
        };
        if !player.is_alive {
            return false;
        }
        let Some(item) = player.second_item else {
            return false;
        };

        let is_used = match item {
            SecondItem::Shield => {
                player.shield_until_beat = Some(current_beat + SHIELD_DURATION_BEATS);
                true
            }
            SecondItem::Detonator => self.detonate_player_bombs(player_id),
            SecondItem::Dash => self.dash_player(player_id),
        };

        if is_used {
            if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
                p.second_item = None;
            }
        }
        is_used
    }

    /// Instantly explodes every bomb owned by the player.
    /// Returns `false` (item kept) when the player has no bomb on the map.
    pub fn detonate_player_bombs(&mut self, player_id: u32) -> bool {
        let mut own_bombs = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if let Cell::Bomb { owner_id, .. } = self.grid[y * self.width + x] {
                    if owner_id == player_id {
                        own_bombs.push((x, y));
                    }
                }
            }
        }
        if own_bombs.is_empty() {
            return false;
        }
        self.detonate_chain(own_bombs);
        true
    }

    /// Clears expired emotes so that clients (which never receive `emote_until`)
    /// stop displaying them.
    pub fn tick_emotes(&mut self) {
        let now = std::time::Instant::now();
        for player in &mut self.players {
            if player.emote_until.is_some_and(|until| now >= until) {
                player.active_emote = None;
                player.emote_until = None;
            }
        }
    }

    pub fn trigger_emote(&mut self, player_id: u32, index: u8) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            let emote = match index {
                1 => "👋",
                2 => "✌️",
                3 => "🖕",
                4 => "👍",
                _ => return,
            };
            player.active_emote = Some(emote.to_string());
            player.emote_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(1500));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::models::{Player, SecondItem};
    use crate::game::rhythm::BeatAccuracy;

    fn create_test_player(id: u32) -> Player {
        let mut player = Player::new(id, format!("Player {}", id), "🤖", 3);
        player.sub_x = 2;
        player.sub_y = 2;
        player.combo = 5;
        player
    }

    #[test]
    fn handle_action_resets_combo_on_miss() {
        let mut state = GameState::new(5, 5);
        state.players = vec![create_test_player(1)];

        state.handle_action(1, GameAction::MoveLeft, BeatAccuracy::Miss, 1);

        assert_eq!(state.players[0].combo, 0);
        assert_eq!(state.players[0].last_accuracy, BeatAccuracy::Miss);
    }

    #[test]
    fn trigger_action_2_detonator_explodes_own_bombs_immediately() {
        let mut state = GameState::new(7, 5);
        let mut player = create_test_player(1);
        player.second_item = Some(SecondItem::Detonator);
        player.active_bombs = 1;
        state.players = vec![player];
        state.grid[3 * 7 + 5] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 4,
            range: 1,
            pierce: false,
        };

        let used = state.trigger_action_2(1, 3);

        assert!(used);
        assert!(matches!(state.grid[3 * 7 + 5], Cell::Explosion { .. }));
        assert_eq!(state.players[0].active_bombs, 0);
    }

    #[test]
    fn trigger_action_2_detonator_kept_when_no_bomb_placed() {
        let mut state = GameState::new(7, 5);
        let mut player = create_test_player(1);
        player.second_item = Some(SecondItem::Detonator);
        state.players = vec![player];

        let used = state.trigger_action_2(1, 3);

        assert!(!used);
        assert_eq!(state.players[0].second_item, Some(SecondItem::Detonator));
    }

    #[test]
    fn trigger_action_2_dash_moves_several_cells_in_facing_direction() {
        let mut state = GameState::new(11, 5);
        let mut player = create_test_player(1);
        player.sub_x = 2;
        player.sub_y = 1;
        player.facing = crate::game::models::Direction::Right;
        player.second_item = Some(SecondItem::Dash);
        state.players = vec![player];

        let used = state.trigger_action_2(1, 3);

        assert!(used);
        assert_eq!(state.players[0].sub_x, 8);
    }

    #[test]
    fn tick_emotes_clears_expired_emote() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.active_emote = Some("👋".to_string());
        player.emote_until = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        state.players = vec![player];

        state.tick_emotes();

        assert!(state.players[0].active_emote.is_none());
    }

    #[test]
    fn trigger_action_2_activates_shield_when_equipped() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.second_item = Some(SecondItem::Shield);
        state.players = vec![player];

        let success = state.trigger_action_2(1, 10);

        assert!(success);
        assert_eq!(
            state.players[0].shield_until_beat,
            Some(10 + SHIELD_DURATION_BEATS)
        );
        assert!(state.players[0].second_item.is_none());
    }
}
