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
        let fg_color = get_color_for_id(player.id);

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

        let ready_tag = if player.is_ready {
            if ascii { " [READY]" } else { " ✅" }
        } else {
            if ascii { " [WAIT]" } else { " ⏳" }
        };
        let ready_color = if player.is_ready {
            Color::Green
        } else {
            Color::DarkGray
        };
        spans.push(Span::styled(ready_tag, Style::default().fg(ready_color)));

        right_lines.push(Line::from(spans));
        right_lines.push(Line::from(""));
    }

    Paragraph::new(right_lines)
        .block(right_block)
        .render(area, buffer);
}

fn get_color_for_id(id: u32) -> Color {
    let colors = [
        Color::Green,
        Color::Magenta,
        Color::Yellow,
        Color::Blue,
        Color::Red,
        Color::Cyan,
        Color::White,
    ];
    colors[(id as usize) % colors.len()]
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
