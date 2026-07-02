use ratatui::style::Color;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AnimationFrame {
    pub symbol: String,
    pub fg_color: Color,
    pub bg_color: Color,
    pub duration: Duration,
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub loop_animation: bool,
}

impl Animation {
    pub fn get_frame(&self, elapsed: Duration) -> &AnimationFrame {
        if self.frames.is_empty() {
            panic!("Animation has no frames!");
        }

        let total_duration: Duration = self.frames.iter().map(|f| f.duration).sum();
        if total_duration.is_zero() {
            return &self.frames[0];
        }

        let mut elapsed_ms = elapsed.as_millis();
        let total_ms = total_duration.as_millis();

        if self.loop_animation {
            elapsed_ms %= total_ms;
        } else if elapsed_ms >= total_ms {
            return &self.frames[self.frames.len() - 1];
        }

        let mut current_ms = 0;
        for frame in &self.frames {
            current_ms += frame.duration.as_millis();
            if elapsed_ms < current_ms {
                return frame;
            }
        }

        &self.frames[0]
    }

    pub fn heart_beating(bpm: f64) -> Self {
        let beat_duration_ms = (60.0 / bpm * 1000.0) as u64;
        let f0 = (beat_duration_ms as f64 * 0.25) as u64;
        let f1 = (beat_duration_ms as f64 * 0.15) as u64;
        let f2 = beat_duration_ms.saturating_sub(f0 + f1);

        Self {
            frames: vec![
                AnimationFrame {
                    symbol: "❤️  [BOOM]".to_string(),
                    fg_color: Color::Red,
                    bg_color: Color::Reset,
                    duration: Duration::from_millis(f0.max(10)),
                },
                AnimationFrame {
                    symbol: "💖  [BOOM]".to_string(),
                    fg_color: Color::LightRed,
                    bg_color: Color::Reset,
                    duration: Duration::from_millis(f1.max(10)),
                },
                AnimationFrame {
                    symbol: "🖤  [TICK]".to_string(),
                    fg_color: Color::DarkGray,
                    bg_color: Color::Reset,
                    duration: Duration::from_millis(f2.max(10)),
                },
            ],
            loop_animation: true,
        }
    }

    pub fn bomb_pulsing() -> Self {
        Self {
            frames: vec![
                AnimationFrame {
                    symbol: "💣".to_string(),
                    fg_color: Color::Red,
                    bg_color: Color::Indexed(234),
                    duration: Duration::from_millis(250),
                },
                AnimationFrame {
                    symbol: "💣".to_string(),
                    fg_color: Color::Yellow,
                    bg_color: Color::Indexed(234),
                    duration: Duration::from_millis(250),
                },
                AnimationFrame {
                    symbol: "💣".to_string(),
                    fg_color: Color::Black,
                    bg_color: Color::Red,
                    duration: Duration::from_millis(250),
                },
            ],
            loop_animation: true,
        }
    }

    pub fn explosion_expanding() -> Self {
        Self {
            frames: vec![
                AnimationFrame {
                    symbol: "💥".to_string(),
                    fg_color: Color::Red,
                    bg_color: Color::Rgb(255, 69, 0),
                    duration: Duration::from_millis(100),
                },
                AnimationFrame {
                    symbol: "🔥".to_string(),
                    fg_color: Color::Yellow,
                    bg_color: Color::Rgb(255, 140, 0),
                    duration: Duration::from_millis(100),
                },
                AnimationFrame {
                    symbol: "✨".to_string(),
                    fg_color: Color::White,
                    bg_color: Color::Rgb(255, 215, 0),
                    duration: Duration::from_millis(100),
                },
            ],
            loop_animation: false,
        }
    }
}
