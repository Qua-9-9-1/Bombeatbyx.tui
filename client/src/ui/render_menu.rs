use crate::local::app::App;
use crate::local::controls::ControlMode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_pause_menu(buffer: &mut Buffer, app: &App, tui_area: Rect) {
    if app.controls.mode != ControlMode::Menu {
        return;
    }

    let menu_w = 30;
    let menu_h = 9;
    let menu_x = (tui_area.width.saturating_sub(menu_w)) / 2;
    let menu_y = (tui_area.height.saturating_sub(menu_h)) / 2;
    let menu_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);

    let menu_block = Block::default()
        .title(" 🛠️ PAUSE / MENU 🛠️ ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    let mut menu_content = String::new();
    let items = ["  Continue  ", "  Config  ", "  Quit  "];

    for (idx, item) in items.iter().enumerate() {
        if idx == app.controls.menu_cursor {
            menu_content.push_str(&format!("► 📦 {} ◄\n", item.trim()));
        } else {
            menu_content.push_str(&format!("   {}   \n", item.trim()));
        }
    }

    Paragraph::new(menu_content)
        .block(menu_block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}
