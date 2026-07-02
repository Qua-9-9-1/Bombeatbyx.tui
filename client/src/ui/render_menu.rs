use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    text::{Line, Span},
};
use crate::local::app::App;
use crate::local::settings::GaugeSkin;

fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

pub fn draw_main_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 50, 14);
    let block = Block::default()
        .title(" 👾 MAIN MENU 👾 ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let items = [
        "🎮 Host Game (Local)",
        "🌐 Join Network Game",
        "⚙️  Settings",
        "❌ Quit",
    ];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("💣 B O M B E A S T 💣", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    for (idx, item) in items.iter().enumerate() {
        if idx == app.main_menu_screen.cursor {
            lines.push(Line::from(vec![
                Span::styled("  ► ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(*item, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" ◄  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(format!("    {}    ", item)));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Use Z/S or Up/Down to navigate, Enter to select", Style::default().fg(Color::DarkGray))));

    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}

pub fn draw_settings_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 50, 14);
    let block = Block::default()
        .title(" ⚙️ SETTINGS ⚙️ ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let gauge_str = match app.profile.gauge_skin {
        GaugeSkin::NecroDancer => "Crypt of the NecroDancer",
        GaugeSkin::Undertale => "Undertale",
        GaugeSkin::Simple => "Simple",
    };

    let items = [
        format!("Gauge Skin : < {} >", gauge_str),
        format!("Player Name: < {} >", app.profile.name),
        "Back to Main Menu".to_string(),
    ];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("SETTINGS CONFIGURATION", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    for (idx, item) in items.iter().enumerate() {
        if idx == app.settings_cursor {
            lines.push(Line::from(vec![
                Span::styled("  ► ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(item.as_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" ◄  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(format!("    {}    ", item)));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Use Z/S (Up/Down) to navigate, Q/D (Left/Right) to adjust, Enter to exit", Style::default().fg(Color::DarkGray))));

    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}

pub fn draw_lobby(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 55, 18);
    let block = Block::default()
        .title(" 🎮 GAME LOBBY 🎮 ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let rs = &app.room_settings;
    let is_host = app.is_local_player_host();
    let cursor = app.lobby_screen.cursor;

    let items = [
        if is_host { format!("Map Width      : < {} >", rs.width) } else { format!("Map Width      : {}", rs.width) },
        if is_host { format!("Map Height     : < {} >", rs.height) } else { format!("Map Height     : {}", rs.height) },
        if is_host { format!("BPM (Tempo)    : < {:.0} >", rs.bpm) } else { format!("BPM (Tempo)    : {:.0}", rs.bpm) },
        if is_host { format!("Sudden Death   : < {} >", if rs.sudden_death { "ON" } else { "OFF" }) } else { format!("Sudden Death   : {}", if rs.sudden_death { "ON" } else { "OFF" }) },
        if is_host { format!("Bonus Spawn    : < Every {} beats >", rs.bonus_every) } else { format!("Bonus Spawn    : Every {} beats", rs.bonus_every) },
        format!("Your Skin      : < {} >", app.profile.skin),
        " [ START GAME ] ".to_string(),
    ];

    let mut lines = vec![];
    if is_host {
        lines.push(Line::from(Span::styled("★ YOU ARE HOST ★ Change fields with Q/D or Left/Right:", Style::default().fg(Color::Yellow))));
    } else {
        lines.push(Line::from(Span::styled("⏱️ WAITING FOR HOST TO START...", Style::default().fg(Color::DarkGray))));
    }
    lines.push(Line::from(""));

    for (idx, item) in items.iter().enumerate() {
        if idx == 6 && !is_host { continue; }

        if idx == cursor {
            if idx == 6 {
                lines.push(Line::from(vec![
                    Span::styled("   ► 🔥 ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(item.as_str(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(" 🔥 ◄", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("  ► ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(item.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" ◄", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
            }
        } else {
            if idx == 6 {
                lines.push(Line::from(Span::styled(format!("       {}", item), Style::default().fg(Color::LightGreen))));
            } else {
                lines.push(Line::from(format!("     {}   ", item)));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(" Connected Players:", Style::default().add_modifier(Modifier::UNDERLINED))));

    let role = if is_host { " [Host]" } else { "" };
    lines.push(Line::from(format!("   {} {} (You){}", app.profile.skin, app.profile.name, role)));

    Paragraph::new(lines)
        .block(block)
        .render(menu_rect, buffer);
}

pub fn draw_pause_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_w = 34;
    let menu_h = 10;
    let menu_x = (tui_area.width.saturating_sub(menu_w)) / 2;
    let menu_y = (tui_area.height.saturating_sub(menu_h)) / 2;
    let menu_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);

    let menu_block = Block::default()
        .title(" 🛠️ PAUSE MENU 🛠️ ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    let items = ["Continue", "Settings", "Quit to Main Menu"];

    let mut lines = vec![Line::from("")];

    for (idx, item) in items.iter().enumerate() {
        if idx == app.pause_cursor {
            lines.push(Line::from(vec![
                Span::styled("► ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(*item, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" ◄", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
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
