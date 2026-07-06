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
