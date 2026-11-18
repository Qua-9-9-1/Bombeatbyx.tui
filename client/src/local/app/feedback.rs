use super::App;
use common::game::GameAction;
use common::game::rhythm::{BeatAccuracy, BeatSide, beat_side};
use std::time::Instant;

/// How the local player's last input landed, shown right on the arena.
#[derive(Debug, Clone, PartialEq)]
pub enum HitKind {
    /// Rating, plus the side of the beat when it was not perfect.
    Rated(BeatAccuracy, Option<BeatSide>),
    /// A second input in a beat that was already used.
    AlreadyActed,
    /// Refused by the anti-spam lockout (presses too close together).
    TooFast,
}

#[derive(Debug, Clone)]
pub struct LocalHit {
    pub kind: HitKind,
    pub at: Instant,
}

impl App {
    /// Judges an input locally, before it is applied, to give instant feedback.
    pub(super) fn record_local_hit(&mut self, action: &GameAction) {
        if matches!(action, GameAction::Emote(_)) {
            return;
        }
        let Some(ctx) = self.game_ctx.as_ref() else {
            return;
        };
        if ctx.state.countdown.is_some() || ctx.state.game_over_countdown.is_some() {
            return;
        }
        let now = Instant::now();
        let Some(player) = ctx.state.players.iter().find(|p| p.id == self.current_player_id) else {
            return;
        };
        // Dead players' inputs do nothing: no feedback rather than a fake rating.
        if !player.is_alive {
            return;
        }
        let (target_beat, accuracy) = ctx.rhythm.timing_at(now);
        let beat_used =
            accuracy != BeatAccuracy::Miss && player.last_acted_beat == Some(target_beat);
        let already_acted = beat_used || ctx.has_buffered_action(self.current_player_id);
        let kind = if player.would_be_throttled(now) {
            HitKind::TooFast
        } else if already_acted {
            HitKind::AlreadyActed
        } else {
            let side = (accuracy != BeatAccuracy::Perfect).then(|| beat_side(ctx.rhythm.progress()));
            HitKind::Rated(accuracy, side)
        };
        self.last_hit = Some(LocalHit { kind, at: now });
    }
}
