use crate::local::app::App;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_players_panel(buffer: &mut Buffer, area: Rect, app: &App) {
    let ascii = app.profile.ascii_mode;
    let title = if ascii {
        " [ PLAYERS ] "
    } else {
        " 👥 PLAYERS "
    };
    let right_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mut right_lines = vec![Line::from("")];

    let players = if let Some(ref ctx) = app.game_ctx {
        &ctx.state.players
    } else {
        return;
    };

    for player in players.iter().take(8) {
        let mut display_name = player.name.clone();
        let mut display_skin = player.skin.clone();

        if player.id == app.current_player_id {
            display_name = app.profile.name.clone();
            display_skin = app.profile.skin.clone();
        }

        let skin_cell = get_player_skin_cell(&display_skin, ascii);
        let fg_color = get_color_from_str(&player.color);

        let mut spans = vec![
            Span::styled(skin_cell, Style::default()),
            Span::styled(
                display_name,
                Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
            ),
        ];

        if player.is_host {
            let host_tag = if ascii { " (Host)" } else { " 👑" };
            spans.push(Span::styled(host_tag, Style::default().fg(Color::Yellow)));
        }

        right_lines.push(Line::from(spans));
        right_lines.push(Line::from(""));
    }

    Paragraph::new(right_lines)
        .block(right_block)
        .render(area, buffer);
}

fn get_color_from_str(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        "yellow" => Color::Yellow,
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "white" => Color::White,
        _ => Color::White,
    }
}

fn get_player_skin_cell(skin: &str, ascii: bool) -> String {
    if ascii {
        match skin {
            "🤖" => "[RO] ".to_string(),
            "🐱" => "[CA] ".to_string(),
            "🐸" => "[FR] ".to_string(),
            "🦊" => "[FO] ".to_string(),
            "🐧" => "[PE] ".to_string(),
            _ => format!("[{}] ", skin),
        }
    } else {
        format!(" {} ", skin)
    }
}
