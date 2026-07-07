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
        let progress = self.progress();
        let distance = if progress > 0.5 {
            1.0 - progress
        } else {
            progress
        };

        if distance <= 0.04 {
            BeatAccuracy::Perfect
        } else if distance <= 0.09 {
            BeatAccuracy::Great
        } else if distance <= 0.15 {
            BeatAccuracy::Ok
        } else {
            BeatAccuracy::Miss
        }
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
    fn bonus_points_returns_correct_values() {
        assert_eq!(BeatAccuracy::Perfect.bonus_points(), 50);
        assert_eq!(BeatAccuracy::Great.bonus_points(), 20);
        assert_eq!(BeatAccuracy::Ok.bonus_points(), 5);
        assert_eq!(BeatAccuracy::Miss.bonus_points(), 0);
        assert_eq!(BeatAccuracy::Waiting.bonus_points(), 0);
    }
}
