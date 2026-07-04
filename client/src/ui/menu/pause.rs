use crate::local::app::App;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_pause_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_w = 34;
    let menu_h = 10;
    let menu_x = (tui_area.width.saturating_sub(menu_w)) / 2;
    let menu_y = (tui_area.height.saturating_sub(menu_h)) / 2;
    let menu_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        " [ PAUSE MENU ] "
    } else {
        " 🛠️ PAUSE MENU 🛠️ "
    };
    let menu_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    let items = ["Continue", "Settings", "Quit to Main Menu"];

    let mut lines = vec![Line::from("")];

    let arrow_l = if ascii { "=> " } else { "► " };
    let arrow_r = if ascii { " <=" } else { " ◄" };

    for (idx, item) in items.iter().enumerate() {
        if idx == app.pause_cursor {
            lines.push(Line::from(vec![
                Span::styled(
                    arrow_l,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    *item,
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
            lines.push(Line::from(format!("   {}   ", item)));
        }
    }

    Paragraph::new(lines)
        .block(menu_block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}
