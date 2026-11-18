//! Beat cues drawn on the arena itself, so players never have to look away
//! from the map to keep the rhythm: pulsing border, dancing floor, glowing
//! player cell, timing feedback in the border titles.

use crate::local::app::{App, HitKind};
use crate::local::settings::BEATS_PER_BAR;
use common::game::models::{COMBO_STEP, FEVER_COMBO};
use common::game::rhythm::{BeatAccuracy, BeatSide};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::BorderType,
};
use std::time::Duration;

const FLOOR_DARK: Color = Color::Indexed(234);
const FLOOR_LIGHT: Color = Color::Indexed(236);
/// Background of the local player's cell while an input would be on the beat.
pub const HIT_WINDOW_PLAYER_BG: Color = Color::Indexed(28);
/// Same, brighter on the first beat of each bar (the strong beat).
const DOWNBEAT_PLAYER_BG: Color = Color::Indexed(40);
const FEEDBACK_DURATION: Duration = Duration::from_millis(800);
pub const ARENA_TITLE: &str = " BOMBOMBYX ";

/// Checkerboard floor whose tiles swap on every beat.
pub fn floor_color(x: usize, y: usize, beat: u64) -> Color {
    if (x + y + beat as usize) % 2 == 0 {
        FLOOR_DARK
    } else {
        FLOOR_LIGHT
    }
}

/// Glow of the local player's cell during the hit window.
pub fn player_glow(beat: u64) -> Color {
    if beat % BEATS_PER_BAR == 0 {
        DOWNBEAT_PLAYER_BG
    } else {
        HIT_WINDOW_PLAYER_BG
    }
}

/// Resting border color, heating up with the local player's combo.
pub fn combo_color(combo: u32) -> Color {
    if combo >= FEVER_COMBO {
        Color::Magenta
    } else if combo >= COMBO_STEP {
        Color::Rgb(255, 140, 0)
    } else if combo >= COMBO_STEP / 2 {
        Color::Yellow
    } else {
        Color::DarkGray
    }
}

/// Border lights up (thick, green) while the hit window is open; between
/// beats it shows the combo level.
pub fn arena_border(window_open: bool, combo: u32) -> (Style, BorderType) {
    if window_open {
        (
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            BorderType::Thick,
        )
    } else {
        (Style::default().fg(combo_color(combo)), BorderType::Plain)
    }
}

/// Top title: feedback on the last input for a moment, the game name otherwise.
pub fn feedback_title(app: &App) -> Line<'static> {
    let Some(hit) = app.last_hit.as_ref().filter(|h| h.at.elapsed() < FEEDBACK_DURATION) else {
        return Line::from(ARENA_TITLE);
    };
    let (text, color) = match &hit.kind {
        HitKind::AlreadyActed => (" ONE ACTION PER BEAT ".to_string(), Color::Magenta),
        HitKind::TooFast => (" TOO FAST - SLOW DOWN ".to_string(), Color::Magenta),
        HitKind::Rated(BeatAccuracy::Miss, side) => {
            let side = match side {
                Some(BeatSide::Early) => "EARLY",
                _ => "LATE",
            };
            (format!(" {} - QUEUED FOR NEXT BEAT ", side), Color::LightRed)
        }
        HitKind::Rated(accuracy, side) => {
            let color = match accuracy {
                BeatAccuracy::Perfect => Color::LightGreen,
                BeatAccuracy::Great => Color::LightCyan,
                BeatAccuracy::Ok => Color::Yellow,
                _ => Color::LightRed,
            };
            let side = match side {
                Some(BeatSide::Early) => " - EARLY",
                Some(BeatSide::Late) => " - LATE",
                None => "",
            };
            (format!(" {}{} ", accuracy.as_str(), side), color)
        }
    };
    Line::from(Span::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

/// Bottom title: position in the 4-beat bar, to anticipate the next beats.
pub fn bar_title(beat: u64, window_open: bool, ascii: bool) -> Line<'static> {
    let current = beat % BEATS_PER_BAR;
    let (on, off) = if ascii { ("X", ".") } else { ("◆", "◇") };
    let mut spans = vec![Span::raw(" ")];
    for index in 0..BEATS_PER_BAR {
        let is_current = index == current;
        let style = match (is_current, window_open) {
            (true, true) => Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            (true, false) => Style::default().fg(Color::White),
            _ => Style::default().fg(Color::DarkGray),
        };
        spans.push(Span::styled(if is_current { on } else { off }, style));
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_tiles_swap_on_every_beat() {
        let before = floor_color(3, 4, 10);

        let after = floor_color(3, 4, 11);

        assert_ne!(before, after);
        assert_eq!(before, floor_color(3, 4, 12));
    }

    #[test]
    fn border_heats_up_with_the_combo() {
        let colors: Vec<Color> = [0, COMBO_STEP / 2, COMBO_STEP, FEVER_COMBO]
            .into_iter()
            .map(combo_color)
            .collect();

        assert_eq!(colors[0], Color::DarkGray);
        assert_eq!(colors[3], Color::Magenta);
        assert!(colors.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn player_glow_is_brighter_on_the_downbeat() {
        assert_ne!(player_glow(0), player_glow(1));
        assert_eq!(player_glow(BEATS_PER_BAR), player_glow(0));
    }

    #[test]
    fn bar_title_highlights_current_beat() {
        let line = bar_title(6, false, true);

        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

        assert_eq!(text, " . . X . ");
    }
}
