use crate::local::app::App;
use crate::ui::menu::center_rect;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_host_modal(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let modal_rect = center_rect(tui_area, 60, 16);
    let ascii = app.profile.ascii_mode;
    
    let title = if ascii { " [ HOST CONFIGURATION ] " } else { " 🚀 HOST CONFIGURATION " };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
        
    let mode_str = if app.host_mode == 0 { "Online" } else { "LAN" };
    let vis_str = if app.host_visibility == 0 { "Public" } else { "Private" };
    
    let items = [
        format!("Connection Mode: < {} >", mode_str),
        format!("Room Visibility: < {} >", vis_str),
        " [ Create Room ] ".to_string(),
        " [ Cancel ] ".to_string(),
    ];
    
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("SELECT HOST SETTINGS", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];
    
    let arrow_l = if ascii { "  => " } else { "  ► " };
    let arrow_r = if ascii { " <=  " } else { " ◄  " };
    
    for (idx, item) in items.iter().enumerate() {
        if idx == app.host_cursor {
            lines.push(Line::from(vec![
                Span::styled(arrow_l, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(item.as_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(arrow_r, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(format!("     {}     ", item)));
        }
    }
    
    lines.push(Line::from(""));
    
    let desc = match app.host_cursor {
        0 => {
            if app.host_mode == 0 {
                "Online: Depends on the server. Not recommended if there are server issues."
            } else {
                "LAN: Depends on the host's machine. Not recommended if PC is slow or connection is bad."
            }
        }
        1 => {
            if app.host_visibility == 0 {
                "Public: The room will appear in the public rooms list."
            } else {
                "Private: The room will be hidden. Others will have to enter the code."
            }
        }
        2 => "Create Room: Starts the room with the configured settings.",
        _ => "Cancel: Return to the main menu.",
    };
    
    lines.push(Line::from(Span::styled(desc, Style::default().fg(Color::LightGreen))));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Use Z/S (Up/Down) to navigate, Q/D (Left/Right) to adjust, Enter to select", Style::default().fg(Color::DarkGray))));
    
    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(modal_rect, buffer);
}
