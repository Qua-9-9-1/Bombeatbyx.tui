use super::App;
use crate::tui::ring_bell;
use std::time::Duration;

/// A beat reached later than this (e.g. after an online clock realignment)
/// stays silent: a late tick would teach the wrong timing.
const MAX_BELL_DELAY: Duration = Duration::from_millis(40);

impl App {
    /// Plays the metronome once per new beat, according to the player's setting.
    pub(super) fn ring_beat_sound(&mut self) {
        let Some(rhythm) = self.game_ctx.as_ref().map(|ctx| &ctx.rhythm) else {
            return;
        };
        let beat = rhythm.beat_count;
        if beat == self.last_sound_beat {
            return;
        }
        let is_new_beat = beat > self.last_sound_beat;
        let is_on_time = rhythm.last_beat_time.elapsed() <= MAX_BELL_DELAY;
        self.last_sound_beat = beat;
        if is_new_beat && is_on_time && self.profile.beat_sound.rings_on(beat) {
            let _ = ring_bell();
        }
    }
}
