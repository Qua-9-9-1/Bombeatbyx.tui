//! Rhythm accuracy shown on the victory screen.

use super::helpers::draw_str_centered;
use common::game::Player;
use common::game::models::HitStats;
use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
};

/// Short label drawn on a podium column: share of inputs on the beat.
pub fn podium_label(stats: &HitStats) -> String {
    if stats.total() == 0 {
        "no input".to_string()
    } else {
        format!("{}% on beat", stats.on_beat_percent())
    }
}

/// Full breakdown of one player's inputs.
pub fn breakdown(stats: &HitStats) -> String {
    format!(
        "Perfect {}% | Great {}% | Ok {}% | Miss {}%",
        stats.percent(stats.perfect),
        stats.percent(stats.great),
        stats.percent(stats.ok),
        stats.percent(stats.miss),
    )
}

pub fn draw_local_breakdown(buffer: &mut Buffer, x_center: u16, y: u16, player: Option<&Player>) {
    let Some(player) = player.filter(|p| !p.is_spectator) else {
        return;
    };
    let text = if player.hits.total() == 0 {
        "Your rhythm: no input this match".to_string()
    } else {
        format!("Your rhythm: {}", breakdown(&player.hits))
    };
    draw_str_centered(
        buffer,
        x_center,
        y,
        &text,
        Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_show_percentages() {
        let stats = HitStats {
            perfect: 2,
            great: 1,
            ok: 0,
            miss: 1,
        };

        assert_eq!(podium_label(&stats), "75% on beat");
        assert_eq!(breakdown(&stats), "Perfect 50% | Great 25% | Ok 0% | Miss 25%");
        assert_eq!(podium_label(&HitStats::default()), "no input");
    }
}
