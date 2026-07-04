use ratatui::layout::Rect;

pub mod main;
pub mod host;
pub mod join;
pub mod settings;
pub mod pause;

pub use main::draw_main_menu;
pub use host::draw_host_modal;
pub use join::draw_join_room_menu;
pub use settings::draw_settings_menu;
pub use pause::draw_pause_menu;

pub fn center_rect(tui_area: Rect, desired_w: u16, desired_h: u16) -> Rect {
    let x = tui_area.x + tui_area.width.saturating_sub(desired_w) / 2;
    let y = tui_area.y + tui_area.height.saturating_sub(desired_h) / 2;
    let w = desired_w.min(tui_area.width);
    let h = desired_h.min(tui_area.height);
    Rect::new(x, y, w, h)
}
