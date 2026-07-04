pub mod info;
pub mod players;
pub mod rules;

use crate::local::app::App;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
};

pub fn draw_lobby(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let lobby_rect = crate::ui::menu::center_rect(tui_area, 102, 18);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32),
            Constraint::Length(44),
            Constraint::Length(26),
        ])
        .split(lobby_rect);

    let cursor = app.lobby_screen.cursor;

    info::draw_info_panel(
        buffer,
        columns[0],
        cursor,
        &app.room_settings,
        app.profile.ascii_mode,
    );
    rules::draw_rules_panel(buffer, columns[1], app);
    players::draw_players_panel(buffer, columns[2], app);
}
