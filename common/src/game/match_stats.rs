//! Per-match statistics shown in game (kill feed) and on the victory screen.

use crate::game::rhythm::BeatAccuracy;
use serde::{Deserialize, Serialize};

/// Kills kept for the in-game feed.
pub const KILL_FEED_LEN: usize = 5;

/// How many inputs of each timing a player made during the match.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitStats {
    pub perfect: u32,
    pub great: u32,
    pub ok: u32,
    pub miss: u32,
}

impl HitStats {
    pub fn record(&mut self, accuracy: &BeatAccuracy) {
        let slot = match accuracy {
            BeatAccuracy::Perfect => &mut self.perfect,
            BeatAccuracy::Great => &mut self.great,
            BeatAccuracy::Ok => &mut self.ok,
            BeatAccuracy::Miss => &mut self.miss,
            _ => return,
        };
        *slot = slot.saturating_add(1);
    }

    pub fn total(&self) -> u32 {
        self.perfect + self.great + self.ok + self.miss
    }

    /// Share of `count` in all inputs, in percent (0 when nothing was played).
    pub fn percent(&self, count: u32) -> u32 {
        match self.total() {
            0 => 0,
            total => (count as u64 * 100 / total as u64) as u32,
        }
    }

    /// Inputs that landed in a hit window, in percent.
    pub fn on_beat_percent(&self) -> u32 {
        self.percent(self.perfect + self.great + self.ok)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillEvent {
    /// `None` when crushed by sudden death.
    pub killer: Option<u32>,
    pub victim: u32,
    pub beat: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_stats_count_and_compute_percentages() {
        let mut stats = HitStats::default();
        for accuracy in [
            BeatAccuracy::Perfect,
            BeatAccuracy::Perfect,
            BeatAccuracy::Great,
            BeatAccuracy::Miss,
            BeatAccuracy::Waiting,
        ] {
            stats.record(&accuracy);
        }

        assert_eq!(stats.total(), 4);
        assert_eq!(stats.percent(stats.perfect), 50);
        assert_eq!(stats.on_beat_percent(), 75);
        assert_eq!(HitStats::default().on_beat_percent(), 0);
    }
}
