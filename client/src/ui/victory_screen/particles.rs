use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

pub fn draw_confetti(buffer: &mut Buffer, area: Rect, elapsed_sec: f64) {
    let width = area.width as usize;
    let height = area.height as usize;
    if width == 0 || height == 0 {
        return;
    }

    for i in 0..40 {
        let x_factor = (i * 19 + 7) % width;
        let speed = ((i * 11 + 5) % 4) as f64 + 2.5;
        let y_pos = area.height as f64 - (elapsed_sec * speed);

        let y_idx = (y_pos as i32) % (height as i32);
        let y_idx = if y_idx < 0 {
            height as i32 + y_idx
        } else {
            y_idx
        } as u16;

        let x = area.x + x_factor as u16;
        let y = area.y + y_idx;

        if x < area.x + area.width && y < area.y + area.height {
            let symbol = match i % 4 {
                0 => "*",
                1 => "+",
                2 => "o",
                _ => ".",
            };
            let color = match (i + (elapsed_sec * 5.0) as usize) % 5 {
                0 => Color::Red,
                1 => Color::Yellow,
                2 => Color::Green,
                3 => Color::Cyan,
                _ => Color::Magenta,
            };

            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.set_symbol(symbol);
                cell.set_style(Style::default().fg(color).bg(Color::Reset));
            }
        }
    }
}
