pub mod animation;
pub mod render_lobby;
pub mod render_map;
pub mod render_menu;
pub mod render_rhythm;

use crate::local::app::{App, AppState, CELL_H, CELL_W};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let tui_area = frame.area();
    let buffer = frame.buffer_mut();

    let min_w = 102;
    let min_h = 20;
    if tui_area.width < min_w || tui_area.height < min_h {
        enlarge_terminal_message(buffer, tui_area);
        return;
    }

    match app.state {
        AppState::MainMenu => {
            render_menu::draw_main_menu(buffer, tui_area, app);
        }
        AppState::Lobby => {
            render_lobby::draw_lobby(buffer, tui_area, app);
        }
        AppState::SettingsMenu => {
            render_menu::draw_settings_menu(buffer, tui_area, app);
        }
        AppState::InGame | AppState::PauseMenu => {
            if let Some(ref ctx) = app.game_ctx {
                let map_w = (ctx.state.width as u16 * CELL_W) + 2;
                let map_height = (ctx.state.height as u16 * CELL_H) + 2;
                let sidebar_w = 26_u16;
                let spacing = 2_u16;
                let total_needed_width = map_w + spacing + sidebar_w;
                let total_needed_height = map_height + 8;

                if tui_area.width < total_needed_width || tui_area.height < total_needed_height {
                    enlarge_terminal_message(buffer, tui_area);
                    return;
                }

                let start_x = (tui_area.width - total_needed_width) / 2;
                let start_y = (tui_area.height - total_needed_height) / 2;

                let map_rect = Rect::new(start_x, start_y, map_w, map_height);
                render_map::draw_map(buffer, app, ctx, map_rect);

                let feedback_area = Rect::new(start_x, start_y + map_height, map_w, 1);
                render_rhythm::draw_feedback(buffer, app, ctx, feedback_area);

                let gauge_area = Rect::new(start_x, start_y + map_height + 1, map_w, 1);
                render_rhythm::draw_rhythm_gauge(buffer, app, ctx, gauge_area);

                let combo_area = Rect::new(start_x, start_y + map_height + 2, map_w, 1);
                render_rhythm::draw_local_combo(buffer, app, ctx, combo_area);

                let stats_area = Rect::new(start_x, start_y + map_height + 3, map_w, 5);
                render_map::draw_local_player_stats(buffer, app, ctx, stats_area);

                let sidebar_area = Rect::new(start_x + map_w + spacing, start_y, sidebar_w, total_needed_height);
                render_map::draw_game_sidebar(buffer, app, ctx, sidebar_area);
            }

            if app.state == AppState::PauseMenu {
                render_menu::draw_pause_menu(buffer, tui_area, app);
            }
        }
    }
}

fn enlarge_terminal_message(buffer: &mut ratatui::buffer::Buffer, tui_area: Rect) {
    let msg = Paragraph::new("Enlarge terminal to play!")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));
    msg.render(tui_area, buffer);
}
