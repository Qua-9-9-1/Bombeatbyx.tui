use ratatui::{buffer::Buffer, style::Style};

pub fn draw_str_centered(buffer: &mut Buffer, x_center: u16, y: u16, text: &str, style: Style) {
    let len = text.chars().count();
    let start_x = x_center.saturating_sub((len / 2) as u16);
    buffer.set_string(start_x, y, text, style);
}

pub use crate::ui::get_color_for_id;

pub fn get_player_skin_cell(skin: &str, ascii: bool) -> String {
    if ascii {
        let code = common::game::models::get_skin_short_code(skin);
        format!("[{}] ", code)
    } else {
        format!(" {} ", skin)
    }
}
