use common::game::models::RoomSettings;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_info_panel(
    buffer: &mut Buffer,
    area: Rect,
    cursor: usize,
    rs: &RoomSettings,
    ascii: bool,
) {
    let title = if ascii {
        " [ INFO PANEL ] "
    } else {
        " ℹ️ INFO PANEL "
    };
    let info_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let mut info_lines = vec![Line::from("")];

    let header_width = if ascii {
        " [ MAP WIDTH ] "
    } else {
        "🗺️  MAP WIDTH"
    };
    let header_height = if ascii {
        " [ MAP HEIGHT ] "
    } else {
        "🗺️  MAP HEIGHT"
    };
    let header_bpm = if ascii {
        " [ BPM / TEMPO ] "
    } else {
        "⚡ BPM / TEMPO"
    };
    let header_sudden = if ascii {
        " [ SUDDEN DEATH ] "
    } else {
        "💀 SUDDEN DEATH"
    };
    let header_bonus = if ascii {
        " [ BONUS SPAWN ] "
    } else {
        "🎁 BONUS SPAWN"
    };
    let header_lives = if ascii {
        " [ PLAYER LIVES ] "
    } else {
        "❤️ PLAYER LIVES"
    };
    let header_skin = if ascii {
        " [ CHARACTER SKIN ] "
    } else {
        "🎭 CHARACTER SKIN"
    };
    let header_start = if ascii {
        " [ START MATCH ] "
    } else {
        "🚀 START MATCH"
    };

    match cursor {
        0 => {
            info_lines.push(Line::from(Span::styled(
                header_width,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Defines the width of the"));
            info_lines.push(Line::from("game arena grid."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Rule:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            info_lines.push(Line::from("Must be odd (7 to 29)."));
            info_lines.push(Line::from("Larger maps provide more"));
            info_lines.push(Line::from("space to evade explosions."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Example:",
                Style::default().fg(Color::DarkGray),
            )));
            info_lines.push(Line::from(Span::styled(
                "15 (Standard grid width)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        1 => {
            info_lines.push(Line::from(Span::styled(
                header_height,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Defines the height of the"));
            info_lines.push(Line::from("game arena grid."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Rule:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            info_lines.push(Line::from("Must be odd (7 to 29)."));
            info_lines.push(Line::from("Standard is 15x15."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Example:",
                Style::default().fg(Color::DarkGray),
            )));
            info_lines.push(Line::from(Span::styled(
                "15 (Standard grid height)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        2 => {
            info_lines.push(Line::from(Span::styled(
                header_bpm,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Controls the speed of the"));
            info_lines.push(Line::from("rhythm beat count."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Current Speed Meter:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));

            let pulse_lines = format_bpm_info(rs.bpm, ascii);
            info_lines.extend(pulse_lines);

            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Tip:",
                Style::default().fg(Color::DarkGray),
            )));
            info_lines.push(Line::from(Span::styled(
                "Higher BPM requires faster",
                Style::default().fg(Color::DarkGray),
            )));
            info_lines.push(Line::from(Span::styled(
                "inputs to get PERFECT accuracy.",
                Style::default().fg(Color::DarkGray),
            )));
        }
        3 => {
            info_lines.push(Line::from(Span::styled(
                header_sudden,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("If active, the walls start"));
            info_lines.push(Line::from("closing in automatically"));
            info_lines.push(Line::from("after 2 minutes, and the"));
            info_lines.push(Line::from("tempo rises to 160 BPM."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Example:",
                Style::default().fg(Color::DarkGray),
            )));
            info_lines.push(Line::from(Span::styled(
                "True: High-stress endgame",
                Style::default().fg(Color::DarkGray),
            )));
        }
        4 => {
            info_lines.push(Line::from(Span::styled(
                header_bonus,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Specifies how often power-ups"));
            info_lines.push(Line::from("spawn onto the map grid."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Frequency:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            info_lines.push(Line::from(format!("Every {} beats.", rs.bonus_every)));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Example:",
                Style::default().fg(Color::DarkGray),
            )));
            info_lines.push(Line::from(Span::styled(
                "10: Spawns every 10 beats.",
                Style::default().fg(Color::DarkGray),
            )));
        }
        5 => {
            info_lines.push(Line::from(Span::styled(
                header_lives,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Specifies the number of lives"));
            info_lines.push(Line::from("each player has in the game."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Lives:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            info_lines.push(Line::from(format!(
                "Each player starts with {} lives.",
                rs.lives
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Example:",
                Style::default().fg(Color::DarkGray),
            )));
            info_lines.push(Line::from(Span::styled(
                format!("You can die {} time(s).", rs.lives),
                Style::default().fg(Color::DarkGray),
            )));
        }
        6 => {
            let header_mode = if ascii {
                " [ GAME MODE ] "
            } else {
                "🏆 GAME MODE"
            };
            info_lines.push(Line::from(Span::styled(
                header_mode,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Toggles the rules for"));
            info_lines.push(Line::from("victory in the match."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Deathmatch: Last player"));
            info_lines.push(Line::from("standing with lives wins."));
            info_lines.push(Line::from("Score: High score from"));
            info_lines.push(Line::from("rhythm steps wins."));
        }
        7 => {
            info_lines.push(Line::from(Span::styled(
                header_skin,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(if ascii {
                "Select your character skin."
            } else {
                "Select your emoji skin."
            }));
            info_lines.push(Line::from("This is how other players"));
            info_lines.push(Line::from("will see you on the map."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled(
                "Options:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            if ascii {
                info_lines.push(Line::from("RO (Robot), CA (Cat), FR (Frog),"));
                info_lines.push(Line::from("FO (Fox), PE (Penguin)."));
            } else {
                info_lines.push(Line::from("🤖 Robot, 🐱 Cat, 🐸 Frog,"));
                info_lines.push(Line::from("🦊 Fox, 🐧 Penguin."));
            }
        }
        8 => {
            info_lines.push(Line::from(Span::styled(
                header_start,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Launches the local match."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Press Enter to join the"));
            info_lines.push(Line::from("arena now!"));
        }
        _ => {}
    }

    Paragraph::new(info_lines)
        .block(info_block)
        .render(area, buffer);
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

pub fn get_skin_label(skin: &str, ascii: bool) -> String {
    if ascii {
        match skin {
            "🤖" => "Robot [RO]".to_string(),
            "🐱" => "Cat [CA]".to_string(),
            "🐸" => "Frog [FR]".to_string(),
            "🦊" => "Fox [FO]".to_string(),
            "🐧" => "Penguin [PE]".to_string(),
            _ => format!("[{}]", skin),
        }
    } else {
        skin.to_string()
    }
}
