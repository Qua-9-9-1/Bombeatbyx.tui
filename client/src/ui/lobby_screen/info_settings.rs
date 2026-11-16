//! Info panel texts for the host-only room settings rows.

use super::info::{blank, dim, header, text, title};
use common::game::models::{
    BONUS_PICKUP_POINTS, BRICK_POINTS, KILL_POINTS, RoomSettings, SELF_KILL_PENALTY,
};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn setting_lines(cursor: usize, rs: &RoomSettings, ascii: bool) -> Vec<Line<'static>> {
    match cursor {
        0 => vec![
            header("[ MAP WIDTH ]", "🗺️  MAP WIDTH", ascii),
            blank(),
            text("Defines the width of the"),
            text("game arena grid."),
            blank(),
            title("Rule:"),
            text("Must be odd (7 to 29)."),
            text("Larger maps provide more"),
            text("space to evade explosions."),
            blank(),
            dim("Example:"),
            dim("15 (Standard grid width)"),
        ],
        1 => vec![
            header("[ MAP HEIGHT ]", "🗺️  MAP HEIGHT", ascii),
            blank(),
            text("Defines the height of the"),
            text("game arena grid."),
            blank(),
            title("Rule:"),
            text("Must be odd (7 to 29)."),
            text("Standard is 15x15."),
            blank(),
            dim("Example:"),
            dim("15 (Standard grid height)"),
        ],
        2 => {
            let mut lines = vec![
                header("[ BPM / TEMPO ]", "⚡ BPM / TEMPO", ascii),
                blank(),
                text("Controls the speed of the"),
                text("rhythm beat count."),
                blank(),
                title("Current Speed Meter:"),
            ];
            lines.extend(format_bpm_info(rs.bpm, ascii));
            lines.extend([
                blank(),
                dim("Tip:"),
                dim("Higher BPM requires faster"),
                dim("inputs to get PERFECT accuracy."),
            ]);
            lines
        }
        3 => vec![
            header("[ SUDDEN DEATH ]", "💀 SUDDEN DEATH", ascii),
            blank(),
            text("If active, after 2 minutes"),
            text("(or half the time limit),"),
            text("walls close in one cell per"),
            text("beat, spiralling inward."),
            text("Crushed players lose a life."),
            blank(),
            dim("Example:"),
            dim("True: High-stress endgame"),
        ],
        4 => bonus_lines(rs, ascii),
        5 => vec![
            header("[ PLAYER LIVES ]", "❤️ PLAYER LIVES", ascii),
            blank(),
            text("Specifies the number of lives"),
            text("each player has in the game."),
            blank(),
            title("Lives:"),
            text(format!("Each player starts with {} lives.", rs.lives)),
            blank(),
            dim("Example:"),
            dim(format!("You can die {} time(s).", rs.lives)),
        ],
        6 => vec![
            header("[ GAME MODE ]", "🏆 GAME MODE", ascii),
            blank(),
            text("Toggles the rules for"),
            text("victory in the match."),
            blank(),
            text("Deathmatch: Last player"),
            text("standing with lives wins."),
            text("Score: first to the target."),
            text(format!("Kill +{} | Brick +{}", KILL_POINTS, BRICK_POINTS)),
            text(format!(
                "Bonus +{} | Suicide -{}",
                BONUS_PICKUP_POINTS, SELF_KILL_PENALTY
            )),
            text("On-beat moves x combo mult."),
        ],
        7 => vec![
            header("[ SCORE TO WIN ]", "🎯 SCORE TO WIN", ascii),
            blank(),
            text("Sets the score to reach"),
            text("to win the match."),
        ],
        8 => vec![
            header("[ TIME LIMIT ]", "⏱️ TIME LIMIT", ascii),
            blank(),
            text("Sets the time limit for"),
            text("the match."),
        ],
        _ => Vec::new(),
    }
}

fn bonus_lines(rs: &RoomSettings, ascii: bool) -> Vec<Line<'static>> {
    let (bomb, fire, pierce, life) = if ascii {
        ("B+", "R+", "P+", "L+")
    } else {
        ("💣", "🔥", "🔪", "💖")
    };
    let (shield, detonator, dash) = if ascii {
        ("S+", "D+", "F+")
    } else {
        ("🛡️", "🧨", "👟")
    };
    vec![
        header("[ BONUS SPAWN ]", "🎁 BONUS SPAWN", ascii),
        blank(),
        text(format!("A random bonus every {} beats,", rs.bonus_every)),
        text("plus hidden ones in bricks."),
        blank(),
        text(format!("{} +1 bomb  {} +1 range", bomb, fire)),
        text(format!("{} pierce   {} +1 life", pierce, life)),
        title("Item (E key):"),
        text(format!("{} shield {} detonate", shield, detonator)),
        text(format!("{} dash 3 cells", dash)),
    ]
}

fn format_bpm_info(bpm: f64, ascii: bool) -> Vec<Line<'static>> {
    let elapsed_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_millis();
    let beat_duration_ms = (60.0 / bpm * 1000.0) as u128;
    let progress = (elapsed_ms % beat_duration_ms) as f64 / beat_duration_ms as f64;

    let anim = crate::ui::animation::Animation::heart_beating(bpm);
    let frame = anim.get_frame(std::time::Duration::from_millis(elapsed_ms as u64));

    let pulse_symbol = if ascii {
        if frame.symbol.contains("❤️") {
            "<3 [BOOM]"
        } else if frame.symbol.contains("💖") {
            "<3 [boom]"
        } else {
            ".. [TICK]"
        }
    } else {
        &frame.symbol
    };

    let gauge_width = 18;
    let cursor_pos = (progress * gauge_width as f64) as usize;
    let mut bar = vec!['-'; gauge_width];
    if cursor_pos < gauge_width {
        bar[cursor_pos] = if ascii { 'X' } else { '●' };
    }
    let pulse_bar = bar.iter().collect::<String>();

    vec![
        Line::from(vec![
            Span::styled("Pulse: ", Style::default()),
            Span::styled(
                pulse_symbol.to_string(),
                Style::default()
                    .fg(frame.fg_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            format!(" [{}]", pulse_bar),
            Style::default().fg(Color::LightRed),
        )),
    ]
}
