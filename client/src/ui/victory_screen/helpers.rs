use ratatui::{
    buffer::Buffer,
    style::{Color, Style},
};

pub fn draw_str_centered(buffer: &mut Buffer, x_center: u16, y: u16, text: &str, style: Style) {
    let len = text.chars().count();
    let start_x = x_center.saturating_sub((len / 2) as u16);
    buffer.set_string(start_x, y, text, style);
}

pub fn get_color_for_id(id: u32) -> Color {
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

pub fn get_player_skin_cell(skin: &str, ascii: bool) -> String {
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
