use crate::local::app::App;
use crate::ui::menu::center_rect;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_join_room_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 80, 18);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        " [ JOIN GAME ] "
    } else {
        " 🌐 JOIN GAME "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let filter_str = match app.join_filter_mode {
        0 => "All",
        1 => "Online Only",
        2 => "LAN Only",
        _ => "All",
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("FILTER MODE (Q/D to toggle): < {} >", filter_str),
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let header = format!(
        "  {:<8} | {:<16} | {:<8} | {:<8} | {:<8}",
        "CODE", "HOST", "PLAYERS", "MODE", "TYPE"
    );
    lines.push(Line::from(Span::styled(
        header,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::UNDERLINED),
    )));

    let mut filtered_rooms = Vec::new();
    if app.join_filter_mode == 0 || app.join_filter_mode == 1 {
        for r in &app.network.online_rooms {
            filtered_rooms.push((
                r.code.clone(),
                r.host_name.clone(),
                r.player_count,
                "Online".to_string(),
                if r.is_public { "Public" } else { "Private" },
            ));
        }
    }
    if app.join_filter_mode == 0 || app.join_filter_mode == 2 {
        for r in &app.network.lan_rooms {
            filtered_rooms.push((r.0.clone(), r.1.clone(), r.2, "LAN".to_string(), "Public"));
        }
    }

    if filtered_rooms.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No public rooms found.",
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(
            "  Press [R] to refresh or [P] to join a room by code.",
        ));
        lines.push(Line::from(""));
    } else {
        for (idx, room) in filtered_rooms.iter().enumerate() {
            let row = format!(
                "  {:<8} | {:<16} | {:<8} | {:<8} | {:<8}",
                room.0,
                room.1,
                format!("{}/8", room.2),
                room.3,
                room.4
            );
            if idx == app.join_cursor {
                lines.push(Line::from(Span::styled(
                    format!("► {}", row),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
            } else {
                lines.push(Line::from(format!("  {}", row)));
            }
        }
    }

    lines.push(Line::from(""));

    let instruct =
        " [Enter] Join selected room   [P] Join room by code   [R] Refresh   [Esc] Back ";
    lines.push(Line::from(Span::styled(
        instruct,
        Style::default().fg(Color::DarkGray),
    )));

    if app.network.show_private_join_prompt {
        let overlay_rect = center_rect(menu_rect, 40, 6);
        let overlay_block = Block::default()
            .title(" ENTER ROOM CODE ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::LightGreen));

        let overlay_lines = vec![
            Line::from(""),
            Line::from(format!("  Code: {}█", app.network.private_room_code_input)),
            Line::from(""),
        ];
        Paragraph::new(overlay_lines)
            .block(overlay_block)
            .alignment(Alignment::Center)
            .render(overlay_rect, buffer);
    } else {
        Paragraph::new(lines).block(block).render(menu_rect, buffer);
    }
}
