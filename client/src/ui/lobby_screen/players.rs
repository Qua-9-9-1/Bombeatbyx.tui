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

        if display_name.chars().count() > 10 {
            display_name = display_name.chars().take(8).collect::<String>() + "..";
        }

        let skin_cell = get_player_skin_cell(&display_skin, ascii);
        let fg_color = get_color_for_id(player.id);

        let is_selected = Some(player.id) == app.lobby_screen.selected_player_id;
        let prefix = if is_selected {
            if ascii { "> " } else { "▶ " }
        } else {
            "  "
        };

        let mut name_style = Style::default().fg(fg_color).add_modifier(Modifier::BOLD);
        if is_selected {
            name_style = name_style.add_modifier(Modifier::REVERSED);
        }

        let mut spans = vec![
            Span::styled(
                prefix,
                if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(skin_cell, Style::default().fg(fg_color)),
            Span::styled(display_name, name_style),
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

    if app.lobby_screen.selected_player_id.is_some() {
        right_lines.push(Line::from(""));
        right_lines.push(Line::from(vec![
            Span::styled(
                "T",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":Promote ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "K",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(":Kick ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "B",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(":Ban", Style::default().fg(Color::DarkGray)),
        ]));
    }

    Paragraph::new(right_lines)
        .block(right_block)
        .render(area, buffer);
}

use crate::ui::get_color_for_id;

fn get_player_skin_cell(skin: &str, ascii: bool) -> String {
    if ascii {
        let code = common::game::models::get_skin_short_code(skin);
        format!("[{}] ", code)
    } else {
        format!(" {} ", skin)
    }
}
