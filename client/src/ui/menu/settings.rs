use crate::local::app::App;
use crate::local::settings::GaugeSkin;
use crate::ui::menu::center_rect;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_settings_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 50, 17);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        " [ SETTINGS ] "
    } else {
        " ⚙️ SETTINGS ⚙️ "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let gauge_str = match app.profile.gauge_skin {
        GaugeSkin::NecroDancer => "Crypt of the NecroDancer",
        GaugeSkin::Undertale => "Undertale",
        GaugeSkin::Simple => "Simple",
    };

    let name_display = if app.editing_name {
        format!("{}█", app.profile.name)
    } else {
        app.profile.name.clone()
    };

    let mode_str = if ascii { "ASCII" } else { "Emojis" };

    let items = [
        format!("Gauge Skin  : < {} >", gauge_str),
        format!("Player Name : < {} >", name_display),
        format!("Display Mode: < {} >", mode_str),
        "Back to Main Menu".to_string(),
    ];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "SETTINGS CONFIGURATION",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let arrow_l = if ascii { "  => " } else { "  ► " };
    let arrow_r = if ascii { " <=  " } else { " ◄  " };

    for (idx, item) in items.iter().enumerate() {
        if idx == app.settings_cursor {
            let is_editing = idx == 1 && app.editing_name;
            let item_color = if is_editing {
                Color::LightGreen
            } else {
                Color::Yellow
            };
            lines.push(Line::from(vec![
                Span::styled(
                    arrow_l,
                    Style::default().fg(item_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    item.as_str(),
                    Style::default().fg(item_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    arrow_r,
                    Style::default().fg(item_color).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(format!("    {}    ", item)));
        }
    }

    lines.push(Line::from(""));
    if app.editing_name {
        let help_text = "Type new name, Backspace to delete, Enter to save";
        lines.push(Line::from(Span::styled(
            help_text,
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Use Z/S (Up/Down) to navigate, Q/D (Left/Right) to adjust, Enter to edit/exit",
            Style::default().fg(Color::DarkGray),
        )));
    }

    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}
