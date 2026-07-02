use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    text::{Line, Span},
};
use crate::local::app::App;
use crate::local::settings::GaugeSkin;

pub fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

pub fn draw_main_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 50, 14);
    let ascii = app.profile.ascii_mode;
    
    let title = if ascii { " [ MAIN MENU ] " } else { " 👾 MAIN MENU 👾 " };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let items = if ascii {
        vec!["Host Game", "Join Game", "Settings", "Quit"]
    } else {
        vec!["🎮 Host Game", "🌐 Join Game", "⚙️  Settings", "❌ Quit"]
    };

    let subtitle = if ascii { "=== Bombeatbyx ===" } else { "💣 Bombeatbyx 💣" };
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(subtitle, Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    let arrow_l = if ascii { "  => " } else { "  ► " };
    let arrow_r = if ascii { " <=  " } else { " ◄  " };

    for (idx, item) in items.iter().enumerate() {
        if idx == app.main_menu_screen.cursor {
            lines.push(Line::from(vec![
                Span::styled(arrow_l, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(*item, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(arrow_r, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(format!("    {}    ", item)));
        }
    }

    lines.push(Line::from(""));
    let instruct = if ascii {
        "Use Z/S (Up/Down) to navigate, Enter to select"
    } else {
        "Use Z/S or Up/Down to navigate, Enter to select"
    };
    lines.push(Line::from(Span::styled(instruct, Style::default().fg(Color::DarkGray))));

    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}

pub fn draw_settings_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 50, 15);
    let ascii = app.profile.ascii_mode;
    
    let title = if ascii { " [ SETTINGS ] " } else { " ⚙️ SETTINGS ⚙️ " };
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
        Line::from(Span::styled("SETTINGS CONFIGURATION", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    let arrow_l = if ascii { "  => " } else { "  ► " };
    let arrow_r = if ascii { " <=  " } else { " ◄  " };

    for (idx, item) in items.iter().enumerate() {
        if idx == app.settings_cursor {
            let item_color = if idx == 1 && app.editing_name { Color::LightGreen } else { Color::Yellow };
            lines.push(Line::from(vec![
                Span::styled(arrow_l, Style::default().fg(item_color).add_modifier(Modifier::BOLD)),
                Span::styled(item.as_str(), Style::default().fg(item_color).add_modifier(Modifier::BOLD)),
                Span::styled(arrow_r, Style::default().fg(item_color).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(format!("    {}    ", item)));
        }
    }

    lines.push(Line::from(""));
    if app.editing_name {
        lines.push(Line::from(Span::styled("Type new name, Backspace to delete, Enter to save", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD))));
    } else {
        lines.push(Line::from(Span::styled("Use Z/S (Up/Down) to navigate, Q/D (Left/Right) to adjust, Enter to edit/exit", Style::default().fg(Color::DarkGray))));
    }

    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}

pub fn draw_pause_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_w = 34;
    let menu_h = 10;
    let menu_x = (tui_area.width.saturating_sub(menu_w)) / 2;
    let menu_y = (tui_area.height.saturating_sub(menu_h)) / 2;
    let menu_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);
    let ascii = app.profile.ascii_mode;

    let title = if ascii { " [ PAUSE MENU ] " } else { " 🛠️ PAUSE MENU 🛠️ " };
    let menu_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    let items = ["Continue", "Settings", "Quit to Main Menu"];

    let mut lines = vec![Line::from("")];

    let arrow_l = if ascii { "=> " } else { "► " };
    let arrow_r = if ascii { " <=" } else { " ◄" };

    for (idx, item) in items.iter().enumerate() {
        if idx == app.pause_cursor {
            lines.push(Line::from(vec![
                Span::styled(arrow_l, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(*item, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(arrow_r, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(format!("   {}   ", item)));
        }
    }

    Paragraph::new(lines)
        .block(menu_block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}
