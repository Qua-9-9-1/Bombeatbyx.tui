use super::GameState;
use crate::game::actions::GameAction;
use crate::game::models::SecondItem;
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
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if player.is_alive {
                if let Some(item) = player.second_item {
                    match item {
                        SecondItem::Shield => {
                            player.shield_until_beat = Some(current_beat);
                            player.second_item = None;
                            return true;
                        }
                    }
                }
            }
        }
        false
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
        Player {
            id,
            is_host: id == 1,
            name: format!("Player {}", id),
            skin: "🤖".to_string(),
            sub_x: 2,
            sub_y: 2,
            is_alive: true,
            score: 0,
            combo: 5,
            max_bombs: 1,
            active_bombs: 0,
            bomb_range: 1,
            last_acted_beat: None,
            last_accuracy: BeatAccuracy::Waiting,
            last_action_time: None,
            spam_lockout_until: None,
            active_emote: None,
            emote_until: None,
            lives: 3,
            death_pos: None,
            respawn_timer: None,
            collected_bonuses: Vec::new(),
            is_spectator: false,
            second_item: None,
            shield_until_beat: None,
            is_ready: false,
            death_beat: None,
        }
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
    fn trigger_action_2_activates_shield_when_equipped() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.second_item = Some(SecondItem::Shield);
        state.players = vec![player];

        let success = state.trigger_action_2(1, 10);

        assert!(success);
        assert_eq!(state.players[0].shield_until_beat, Some(10));
        assert!(state.players[0].second_item.is_none());
    }
}
