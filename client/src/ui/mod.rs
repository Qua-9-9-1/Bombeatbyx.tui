pub mod render_map;
pub mod render_menu;
pub mod render_rhythm;

use crate::local::app::{App, CELL_H, CELL_W};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let tui_area = frame.area();
    let buffer = frame.buffer_mut();

    let map_w = (app.game_ctx.state.width as u16 * CELL_W) + 2;
    let map_height = (app.game_ctx.state.height as u16 * CELL_H) + 2;
    let total_needed_height = map_height + 4;

    if tui_area.width < map_w || tui_area.height < total_needed_height {
        enlarge_terminal_message(buffer, tui_area);
        return;
    }

    let start_x = (tui_area.width - map_w) / 2;
    let start_y = (tui_area.height - total_needed_height) / 2;

    let map_rect = Rect::new(start_x, start_y, map_w, map_height);
    render_map::draw_map(buffer, app, map_rect);

    let feedback_area = Rect::new(start_x, start_y + map_height + 1, map_w, 1);
    render_rhythm::draw_feedback(buffer, app, feedback_area);

    let gauge_area = Rect::new(start_x, start_y + map_height + 2, map_w, 1);
    render_rhythm::draw_rhythm_gauge(buffer, app, gauge_area);
    if app.game_run {
        render_menu::draw_pause_menu(buffer, tui_area);
    }
}

fn enlarge_terminal_message(buffer: &mut ratatui::buffer::Buffer, tui_area: Rect) {
    let msg = Paragraph::new("Enlarge terminal to play!")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));
    msg.render(tui_area, buffer);
}
