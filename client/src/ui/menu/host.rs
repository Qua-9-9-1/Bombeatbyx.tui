use crate::local::app::App;
use crate::ui::menu::center_rect;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};

pub fn draw_host_modal(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let modal_rect = center_rect(tui_area, 72, 17);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        " [ HOST CONFIGURATION ] "
    } else {
        " 🚀 HOST CONFIGURATION "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mode_str = if app.host_mode == 0 { "Online" } else { "LAN" };
    let vis_str = if app.host_visibility == 0 {
        "Public"
    } else {
        "Private"
    };

    let items = [
        format!("Connection Mode: < {} >", mode_str),
        format!("Room Visibility: < {} >", vis_str),
        " [ Create Room ] ".to_string(),
        " [ Cancel ] ".to_string(),
    ];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "SELECT HOST SETTINGS",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let arrow_l = if ascii { "  => " } else { "  ► " };
    let arrow_r = if ascii { " <=  " } else { " ◄  " };

    for (idx, item) in items.iter().enumerate() {
        if idx == app.host_cursor {
            lines.push(Line::from(vec![
                Span::styled(
                    arrow_l,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    item.as_str(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    arrow_r,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(format!("     {}     ", item)));
        }
    }

    lines.push(Line::from(""));

    let server_desc;
    let desc = match app.host_cursor {
        0 => {
            if app.host_mode == 0 {
                server_desc = format!(
                    "Online: the room lives on the server set in Settings > Server ({}). A server must be running there.",
                    app.profile.server_addr
                );
                server_desc.as_str()
            } else {
                "LAN: your machine hosts the room for players on the same local network."
            }
        }
        1 => {
            if app.host_visibility == 0 {
                "Public: the room appears in the room list."
            } else {
                "Private: the room is hidden. Others join with its code."
            }
        }
        2 => "Create Room: starts the room with these settings.",
        _ => "Cancel: back to the main menu.",
    };

    lines.push(Line::from(Span::styled(
        desc,
        Style::default().fg(Color::LightGreen),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Z/S navigate   Q/D adjust   Enter select   Esc back",
        Style::default().fg(Color::DarkGray),
    )));

    Paragraph::new(lines)
        .block(block.padding(Padding::horizontal(1)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .render(modal_rect, buffer);
}
