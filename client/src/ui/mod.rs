pub mod animation;
pub mod game_screen;
pub mod lobby_screen;
pub mod menu;
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
            menu::draw_main_menu(buffer, tui_area, app);
        }
        AppState::Lobby => {
            lobby_screen::draw_lobby(buffer, tui_area, app);
        }
        AppState::SettingsMenu => {
            menu::draw_settings_menu(buffer, tui_area, app);
        }
        AppState::HostModal => {
            menu::draw_host_modal(buffer, tui_area, app);
        }
        AppState::JoinRoomMenu => {
            menu::draw_join_room_menu(buffer, tui_area, app);
        }
        AppState::InGame => {
            draw_game_board(buffer, tui_area, app);
        }
        AppState::PauseMenu => {
            if let Some(AppState::Lobby) = app.paused_from {
                lobby_screen::draw_lobby(buffer, tui_area, app);
            } else {
                draw_game_board(buffer, tui_area, app);
            }
            menu::draw_pause_menu(buffer, tui_area, app);
        }
    }
}

fn draw_game_board(buffer: &mut ratatui::buffer::Buffer, tui_area: Rect, app: &App) {
    if let Some(ref ctx) = app.game_ctx {
        let min_display_w = 36_u16;
        let min_layout_h = 42_u16;
        let map_content_w = ctx.state.width as u16 * CELL_W;
        let map_content_h = ctx.state.height as u16 * CELL_H;

        let display_w = map_content_w.max(min_display_w) + 2;
        let map_height = map_content_h + 2;
        let sidebar_w = 26_u16;
        let spacing = 2_u16;
        let total_needed_width = display_w + spacing + sidebar_w;
        let total_needed_height = (map_height + 8).max(min_layout_h);
        let map_box_h = total_needed_height - 8;

        if tui_area.width < total_needed_width || tui_area.height < total_needed_height {
            enlarge_terminal_message(buffer, tui_area);
            return;
        }

        let start_x = (tui_area.width - total_needed_width) / 2;
        let start_y = (tui_area.height - total_needed_height) / 2;

        let map_rect = Rect::new(start_x, start_y, display_w, map_box_h);
        game_screen::draw_map(buffer, app, ctx, map_rect);

        let feedback_area = Rect::new(start_x, start_y + map_box_h, display_w, 1);
        render_rhythm::draw_feedback(buffer, app, ctx, feedback_area);

        let gauge_area = Rect::new(start_x, start_y + map_box_h + 1, display_w, 1);
        render_rhythm::draw_rhythm_gauge(buffer, app, ctx, gauge_area);

        let combo_area = Rect::new(start_x, start_y + map_box_h + 2, display_w, 1);
        render_rhythm::draw_local_combo(buffer, app, ctx, combo_area);

        let stats_area = Rect::new(start_x, start_y + map_box_h + 3, display_w, 5);
        game_screen::draw_local_player_stats(buffer, app, ctx, stats_area);

        let sidebar_area = Rect::new(start_x + display_w + spacing, start_y, sidebar_w, total_needed_height);
        game_screen::draw_game_sidebar(buffer, app, ctx, sidebar_area);
    }
}

fn enlarge_terminal_message(buffer: &mut ratatui::buffer::Buffer, tui_area: Rect) {
    let msg = Paragraph::new("Enlarge terminal to play!")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));
    msg.render(tui_area, buffer);
}
