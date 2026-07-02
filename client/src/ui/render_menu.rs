use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect, Layout, Constraint, Direction},
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

    let name_display = if app.editing_name {
        format!("{}█", app.profile.name)
    } else {
        app.profile.name.clone()
    };

    let items = [
        format!("Gauge Skin : < {} >", gauge_str),
        format!("Player Name: < {} >", name_display),
        "Back to Main Menu".to_string(),
    ];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("SETTINGS CONFIGURATION", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    for (idx, item) in items.iter().enumerate() {
        if idx == app.settings_cursor {
            let item_color = if idx == 1 && app.editing_name { Color::LightGreen } else { Color::Yellow };
            lines.push(Line::from(vec![
                Span::styled("  ► ", Style::default().fg(item_color).add_modifier(Modifier::BOLD)),
                Span::styled(item.as_str(), Style::default().fg(item_color).add_modifier(Modifier::BOLD)),
                Span::styled(" ◄  ", Style::default().fg(item_color).add_modifier(Modifier::BOLD)),
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

pub fn draw_lobby(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let lobby_w = 102;
    let lobby_h = 18;
    let lobby_rect = center_rect(tui_area, lobby_w, lobby_h);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32), // Info column
            Constraint::Length(44), // Main settings column
            Constraint::Length(26), // Player list column
        ])
        .split(lobby_rect);

    let rs = &app.room_settings;
    let is_host = app.is_local_player_host();
    let cursor = app.lobby_screen.cursor;

    // --- LEFT COLUMN: INFO PANEL ---
    let info_block = Block::default()
        .title(" ℹ️ INFO PANEL ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let mut info_lines = vec![Line::from("")];

    match cursor {
        0 => {
            info_lines.push(Line::from(Span::styled("🗺️  MAP WIDTH", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Defines the width of the"));
            info_lines.push(Line::from("game arena grid."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Rule:", Style::default().add_modifier(Modifier::UNDERLINED))));
            info_lines.push(Line::from("Must be odd (7 to 29)."));
            info_lines.push(Line::from("Larger maps provide more"));
            info_lines.push(Line::from("space to evade explosions."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Example:", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("15 (Standard grid width)", Style::default().fg(Color::DarkGray))));
        }
        1 => {
            info_lines.push(Line::from(Span::styled("🗺️  MAP HEIGHT", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Defines the height of the"));
            info_lines.push(Line::from("game arena grid."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Rule:", Style::default().add_modifier(Modifier::UNDERLINED))));
            info_lines.push(Line::from("Must be odd (7 to 29)."));
            info_lines.push(Line::from("Standard is 15x15."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Example:", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("15 (Standard grid height)", Style::default().fg(Color::DarkGray))));
        }
        2 => {
            info_lines.push(Line::from(Span::styled("⚡ BPM / TEMPO", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Controls the speed of the"));
            info_lines.push(Line::from("rhythm beat count."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Current Speed Meter:", Style::default().add_modifier(Modifier::UNDERLINED))));
            
            // Visual heartbeat animation
            let elapsed_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::ZERO)
                .as_millis();
            let beat_duration_ms = (60.0 / rs.bpm * 1000.0) as u128;
            let progress = (elapsed_ms % beat_duration_ms) as f64 / beat_duration_ms as f64;
            
            let heart_symbol = if progress < 0.25 { "❤️  [BOOM]" } else { "🖤  [tick]" };
            let gauge_width = 18;
            let cursor_pos = (progress * gauge_width as f64) as usize;
            let mut bar = vec!['-'; gauge_width];
            if cursor_pos < gauge_width {
                bar[cursor_pos] = '●';
            }
            let pulse_bar = bar.iter().collect::<String>();
            
            info_lines.push(Line::from(vec![
                Span::styled("Pulse: ", Style::default()),
                Span::styled(heart_symbol, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]));
            info_lines.push(Line::from(Span::styled(format!(" [{}]", pulse_bar), Style::default().fg(Color::LightRed))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Tip:", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("Higher BPM requires faster", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("inputs to get PERFECT accuracy.", Style::default().fg(Color::DarkGray))));
        }
        3 => {
            info_lines.push(Line::from(Span::styled("💀 SUDDEN DEATH", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("If active, the walls start"));
            info_lines.push(Line::from("closing in automatically"));
            info_lines.push(Line::from("after 2 minutes, and the"));
            info_lines.push(Line::from("tempo rises to 160 BPM."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Example:", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("True: High-stress endgame", Style::default().fg(Color::DarkGray))));
        }
        4 => {
            info_lines.push(Line::from(Span::styled("🎁 BONUS SPAWN", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Specifies how often power-ups"));
            info_lines.push(Line::from("spawn onto the map grid."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Frequency:", Style::default().add_modifier(Modifier::UNDERLINED))));
            info_lines.push(Line::from(format!("Every {} beats.", rs.bonus_every)));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Example:", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("10: Spawns every 10 beats.", Style::default().fg(Color::DarkGray))));
        }
        5 => {
            info_lines.push(Line::from(Span::styled("🎭 CHARACTER SKIN", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Select your emoji skin."));
            info_lines.push(Line::from("This is how other players"));
            info_lines.push(Line::from("will see you on the map."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Options:", Style::default().add_modifier(Modifier::UNDERLINED))));
            info_lines.push(Line::from("🤖 Robot, 🐱 Cat, 🐸 Frog,"));
            info_lines.push(Line::from("🦊 Fox, 🐧 Penguin."));
        }
        6 => {
            info_lines.push(Line::from(Span::styled("🚀 START MATCH", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Launches the local match."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Press Enter to join the"));
            info_lines.push(Line::from("arena now!"));
        }
        _ => {}
    }

    Paragraph::new(info_lines)
        .block(info_block)
        .render(columns[0], buffer);

    // --- CENTER COLUMN: MAIN RULES ---
    let center_block = Block::default()
        .title(" 🎮 ROOM CONFIG ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let items = [
        if is_host { format!("Map Width     : < {} >", rs.width) } else { format!("Map Width     : {}", rs.width) },
        if is_host { format!("Map Height    : < {} >", rs.height) } else { format!("Map Height    : {}", rs.height) },
        if is_host { format!("BPM (Tempo)   : < {:.0} >", rs.bpm) } else { format!("BPM (Tempo)   : {:.0}", rs.bpm) },
        if is_host { format!("Sudden Death  : < {} >", if rs.sudden_death { "ON" } else { "OFF" }) } else { format!("Sudden Death  : {}", if rs.sudden_death { "ON" } else { "OFF" }) },
        if is_host { format!("Bonus Spawn   : < Every {} beats >", rs.bonus_every) } else { format!("Bonus Spawn   : Every {} beats", rs.bonus_every) },
        format!("Your Skin     : < {} >", app.profile.skin),
        " [ START GAME ] ".to_string(),
    ];

    let mut center_lines = vec![Line::from("")];
    if is_host {
        center_lines.push(Line::from(Span::styled("  ★ HOST SETTINGS (Q/D to adjust) ★", Style::default().fg(Color::Yellow))));
    } else {
        center_lines.push(Line::from(Span::styled("  ⏱️ WAITING FOR HOST...", Style::default().fg(Color::DarkGray))));
    }
    center_lines.push(Line::from(""));

    for (idx, item) in items.iter().enumerate() {
        if idx == 6 && !is_host { continue; }

        if idx == cursor {
            if idx == 6 {
                center_lines.push(Line::from(vec![
                    Span::styled(" ► 🔥 ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(item.as_str(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(" 🔥 ◄", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
            } else {
                center_lines.push(Line::from(vec![
                    Span::styled(" ► ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(item.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" ◄", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
            }
        } else {
            if idx == 6 {
                center_lines.push(Line::from(Span::styled(format!("     {}", item), Style::default().fg(Color::LightGreen))));
            } else {
                center_lines.push(Line::from(format!("   {}   ", item)));
            }
        }
    }

    Paragraph::new(center_lines)
        .block(center_block)
        .render(columns[1], buffer);

    // --- RIGHT COLUMN: PLAYERS LIST ---
    let right_block = Block::default()
        .title(" 👥 PLAYERS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mut right_lines = vec![Line::from("")];

    // Local player (You)
    right_lines.push(Line::from(vec![
        Span::styled(format!(" {} ", app.profile.skin), Style::default()),
        Span::styled(app.profile.name.as_str(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" 👑", Style::default().fg(Color::Yellow)),
    ]));
    right_lines.push(Line::from(""));

    // Mock Player 2
    right_lines.push(Line::from(vec![
        Span::styled(" 🐱 ", Style::default()),
        Span::styled("GigaPlayer", Style::default().fg(Color::Magenta)),
    ]));
    right_lines.push(Line::from(""));

    // Mock Player 3
    right_lines.push(Line::from(vec![
        Span::styled(" 🐸 ", Style::default()),
        Span::styled("Ribbit", Style::default().fg(Color::Yellow)),
    ]));

    Paragraph::new(right_lines)
        .block(right_block)
        .render(columns[2], buffer);
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
