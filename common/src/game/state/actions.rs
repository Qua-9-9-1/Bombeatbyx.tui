use super::GameState;
use crate::game::actions::GameAction;
use crate::game::models::{Cell, SHIELD_DURATION_BEATS, SecondItem};

const MAX_COMBO: u32 = 9999;
const EMOTE_DURATION: std::time::Duration = std::time::Duration::from_millis(1500);
/// Minimum delay between two emotes of the same player.
const EMOTE_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(500);
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

            if !player.is_alive {
                return;
            }

            let success =
                self.apply_player_action(player_id, action, accuracy.clone(), current_beat);
            // An on-beat action keeps the groove even if it fails (e.g. bumping
            // a wall), but it consumes the beat: one action per beat.
            if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
                p.last_acted_beat = Some(current_beat);
                p.last_accuracy = accuracy.clone();
                if accuracy != BeatAccuracy::Ok {
                    p.combo = (p.combo + 1).min(MAX_COMBO);
                }
                if success {
                    let points = accuracy.bonus_points() * p.combo_multiplier();
                    p.score = p.score.saturating_add(points);
                }
            }
        }
    }

    /// Runs an input buffered after an off-beat press. It happens on the beat,
    /// but earns no rhythm points and breaks the combo.
    pub fn handle_buffered_action(&mut self, player_id: u32, action: GameAction, beat: u64) {
        if self.countdown.is_some() || self.game_over_countdown.is_some() {
            return;
        }
        let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) else {
            return;
        };
        if !player.is_alive || player.last_acted_beat == Some(beat) {
            return;
        }
        player.combo = 0;
        player.last_acted_beat = Some(beat);
        player.last_accuracy = BeatAccuracy::Miss;
        self.apply_player_action(player_id, action, BeatAccuracy::Miss, beat);
    }

    /// Resets the combo of every active player who let a beat pass without acting.
    pub fn reset_idle_combos(&mut self, current_beat: u64) {
        for player in &mut self.players {
            let is_active = player.is_alive && !player.is_spectator;
            if is_active && player.last_acted_beat != Some(current_beat) {
                player.combo = 0;
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
            // Emotes ignore the beat, so they get their own anti-flood cooldown
            // (each one changes the state and triggers a broadcast).
            let now = std::time::Instant::now();
            let on_cooldown = player
                .emote_until
                .is_some_and(|until| until > now + EMOTE_DURATION - EMOTE_COOLDOWN);
            if player.is_spectator || on_cooldown {
                return;
            }
            let emote = match index {
                1 => "👋",
                2 => "✌️",
                3 => "🖕",
                4 => "👍",
                _ => return,
            };
            player.active_emote = Some(emote.to_string());
            player.emote_until = Some(now + EMOTE_DURATION);
        }
    }
}

#[cfg(test)]
#[path = "actions_tests.rs"]
mod tests;
