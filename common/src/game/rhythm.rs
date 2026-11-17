use crate::game::rules::{GREAT_WINDOW, OK_WINDOW, PERFECT_WINDOW};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BeatAccuracy {
    Perfect,
    Great,
    Ok,
    Miss,
    Waiting,
}

impl BeatAccuracy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Perfect => "PERFECT!",
            Self::Great => "GREAT!",
            Self::Ok => "OKAY!",
            Self::Miss => "MISS!",
            Self::Waiting => "WAITING...",
        }
    }

    pub fn bonus_points(&self) -> u32 {
        match self {
            Self::Perfect => 50,
            Self::Great => 20,
            Self::Ok => 5,
            Self::Miss => 0,
            Self::Waiting => 0,
        }
    }
}

pub struct RhythmEngine {
    pub bpm: f64,
    pub beat_interval: Duration,
    pub last_beat_time: Instant,
    pub next_beat_time: Instant,
    pub beat_count: u64,
}

impl RhythmEngine {
    pub fn new(bpm: f64) -> Self {
        let interval = Duration::from_secs_f64(60.0 / bpm);
        let now = Instant::now();
        Self {
            bpm,
            beat_interval: interval,
            last_beat_time: now,
            next_beat_time: now + interval,
            beat_count: 0,
        }
    }

    pub fn progress(&self) -> f64 {
        let now = Instant::now();
        if now >= self.next_beat_time {
            return 1.0;
        }
        let elapsed = now.duration_since(self.last_beat_time).as_secs_f64();
        let total = self.beat_interval.as_secs_f64();
        (elapsed / total).clamp(0.0, 1.0)
    }

    pub fn tick_logic(&mut self) -> bool {
        let now = Instant::now();
        if now >= self.next_beat_time {
            self.last_beat_time = self.next_beat_time;
            self.next_beat_time += self.beat_interval;
            self.beat_count += 1;
            true
        } else {
            false
        }
    }

    pub fn evaluate_accuracy(&self) -> BeatAccuracy {
        self.timing_at(Instant::now()).1
    }

    /// Beat targeted by an input made at `at` and how accurate it was.
    /// `at` may be slightly in the past (latency compensation), up to one beat.
    pub fn timing_at(&self, at: Instant) -> (u64, BeatAccuracy) {
        let (base_beat, base_time) = if at >= self.last_beat_time {
            (self.beat_count, self.last_beat_time)
        } else {
            (
                self.beat_count.saturating_sub(1),
                self.last_beat_time
                    .checked_sub(self.beat_interval)
                    .unwrap_or(self.last_beat_time),
            )
        };
        let elapsed = at.saturating_duration_since(base_time).as_secs_f64();
        let progress = (elapsed / self.beat_interval.as_secs_f64()).clamp(0.0, 1.0);
        let target_beat = if progress > 0.5 { base_beat + 1 } else { base_beat };
        (target_beat, accuracy_from_progress(progress))
    }

    /// Realigns the clock on an authoritative beat that started `since_beat` ago.
    /// Drift below `tolerance` is ignored to avoid jitter. Returns `true` if it moved.
    pub fn align(&mut self, beat: u64, since_beat: Duration, tolerance: Duration) -> bool {
        let now = Instant::now();
        if self.beat_count == beat {
            let local_since = now.saturating_duration_since(self.last_beat_time);
            let drift = local_since.abs_diff(since_beat);
            if drift <= tolerance {
                return false;
            }
        }
        self.beat_count = beat;
        self.last_beat_time = now.checked_sub(since_beat).unwrap_or(now);
        self.next_beat_time = self.last_beat_time + self.beat_interval;
        true
    }
}

/// Side of the nearest beat an input fell on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatSide {
    Early,
    Late,
}

/// Inputs in the second half of a beat target the next beat: they are early.
pub fn beat_side(progress: f64) -> BeatSide {
    if progress > 0.5 { BeatSide::Early } else { BeatSide::Late }
}

/// Distance to the nearest beat, as a fraction of the beat interval (0..=0.5).
pub fn distance_to_beat(progress: f64) -> f64 {
    if progress > 0.5 { 1.0 - progress } else { progress }
}

/// `true` while an input would not be a miss.
pub fn is_in_hit_window(progress: f64) -> bool {
    distance_to_beat(progress) <= OK_WINDOW
}

fn accuracy_from_progress(progress: f64) -> BeatAccuracy {
    let distance = distance_to_beat(progress);

    if distance <= PERFECT_WINDOW {
        BeatAccuracy::Perfect
    } else if distance <= GREAT_WINDOW {
        BeatAccuracy::Great
    } else if distance <= OK_WINDOW {
        BeatAccuracy::Ok
    } else {
        BeatAccuracy::Miss
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_accuracy_window_correct_ratings() {
        let mut engine = RhythmEngine::new(60.0);
        let interval = engine.beat_interval;

        engine.last_beat_time = Instant::now() - interval.mul_f64(0.02);
        engine.next_beat_time = engine.last_beat_time + interval;
        assert_eq!(engine.evaluate_accuracy(), BeatAccuracy::Perfect);

        engine.last_beat_time = Instant::now() - interval.mul_f64(0.07);
        engine.next_beat_time = engine.last_beat_time + interval;
        assert_eq!(engine.evaluate_accuracy(), BeatAccuracy::Great);

        engine.last_beat_time = Instant::now() - interval.mul_f64(0.12);
        engine.next_beat_time = engine.last_beat_time + interval;
        assert_eq!(engine.evaluate_accuracy(), BeatAccuracy::Ok);

        engine.last_beat_time = Instant::now() - interval.mul_f64(0.25);
        engine.next_beat_time = engine.last_beat_time + interval;
        assert_eq!(engine.evaluate_accuracy(), BeatAccuracy::Miss);
    }

    #[test]
    fn timing_at_past_instant_targets_previous_beat_late_window() {
        let mut engine = RhythmEngine::new(60.0);
        let interval = engine.beat_interval;
        engine.beat_count = 10;
        engine.last_beat_time = Instant::now() - interval.mul_f64(0.05);
        engine.next_beat_time = engine.last_beat_time + interval;
        let input_time = engine.last_beat_time - interval.mul_f64(0.02);

        let (target, accuracy) = engine.timing_at(input_time);

        assert_eq!(target, 10);
        assert_eq!(accuracy, BeatAccuracy::Perfect);
    }

    #[test]
    fn timing_at_late_in_beat_targets_next_beat() {
        let mut engine = RhythmEngine::new(60.0);
        let interval = engine.beat_interval;
        engine.beat_count = 4;
        engine.last_beat_time = Instant::now() - interval.mul_f64(0.97);
        engine.next_beat_time = engine.last_beat_time + interval;

        let (target, accuracy) = engine.timing_at(Instant::now());

        assert_eq!(target, 5);
        assert_eq!(accuracy, BeatAccuracy::Perfect);
    }

    #[test]
    fn align_ignores_drift_within_tolerance() {
        let mut engine = RhythmEngine::new(60.0);
        engine.beat_count = 3;
        let before = engine.last_beat_time;

        let has_moved = engine.align(3, Duration::from_millis(5), Duration::from_millis(25));

        assert!(!has_moved);
        assert_eq!(engine.last_beat_time, before);
    }

    #[test]
    fn align_jumps_to_authoritative_beat() {
        let mut engine = RhythmEngine::new(60.0);
        engine.beat_count = 3;

        let has_moved = engine.align(7, Duration::from_millis(40), Duration::from_millis(25));

        assert!(has_moved);
        assert_eq!(engine.beat_count, 7);
        let since = Instant::now().duration_since(engine.last_beat_time);
        assert!(since >= Duration::from_millis(40) && since < Duration::from_millis(200));
    }

    #[test]
    fn bonus_points_returns_correct_values() {
        assert_eq!(BeatAccuracy::Perfect.bonus_points(), 50);
        assert_eq!(BeatAccuracy::Great.bonus_points(), 20);
        assert_eq!(BeatAccuracy::Ok.bonus_points(), 5);
        assert_eq!(BeatAccuracy::Miss.bonus_points(), 0);
        assert_eq!(BeatAccuracy::Waiting.bonus_points(), 0);
    }

    #[test]
    fn hit_window_is_symmetric_around_the_beat() {
        let edges = [OK_WINDOW, 1.0 - OK_WINDOW, OK_WINDOW + 0.01, 1.0 - OK_WINDOW - 0.01];

        let open: Vec<bool> = edges.iter().map(|p| is_in_hit_window(*p)).collect();

        assert_eq!(open, vec![true, true, false, false]);
    }

    #[test]
    fn beat_side_reports_early_before_and_late_after_the_beat() {
        let before_beat = 0.9;
        let after_beat = 0.1;

        let sides = (beat_side(before_beat), beat_side(after_beat));

        assert_eq!(sides, (BeatSide::Early, BeatSide::Late));
    }
}
