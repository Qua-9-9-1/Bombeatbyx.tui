use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GaugeSkin {
    NecroDancer,
    Undertale,
}

pub struct RhythmManager {
    pub bpm: f64,
    pub beat_interval: Duration,
    pub last_beat_time: Instant,
    pub next_beat_time: Instant,
    pub skin: GaugeSkin,
}

impl RhythmManager {
    pub fn new(bpm: f64, skin: GaugeSkin) -> Self {
        let interval = Duration::from_secs_f64(60.0 / bpm);
        let now = Instant::now();
        Self {
            bpm,
            beat_interval: interval,
            last_beat_time: now,
            next_beat_time: now + interval,
            skin,
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
            true
        } else {
            false
        }
    }
}
