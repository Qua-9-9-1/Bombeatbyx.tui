use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect, Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    text::{Line, Span},
};
use crate::local::app::App;
use common::game::models::RoomSettings;

pub fn draw_lobby(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let lobby_rect = crate::ui::render_menu::center_rect(tui_area, 102, 18);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32),
            Constraint::Length(44),
            Constraint::Length(26),
        ])
        .split(lobby_rect);

    let cursor = app.lobby_screen.cursor;

    draw_info_panel(buffer, columns[0], cursor, &app.room_settings);
    draw_rules_panel(buffer, columns[1], app);
    draw_players_panel(buffer, columns[2], app);
}

fn draw_info_panel(buffer: &mut Buffer, area: Rect, cursor: usize, rs: &RoomSettings) {
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
            
            let pulse_lines = format_bpm_info(rs.bpm);
            info_lines.extend(pulse_lines);
            
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
        .render(area, buffer);
}

fn format_bpm_info(bpm: f64) -> Vec<Line<'static>> {
    let elapsed_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_millis();
    let beat_duration_ms = (60.0 / bpm * 1000.0) as u128;
    let progress = (elapsed_ms % beat_duration_ms) as f64 / beat_duration_ms as f64;
    
    let heart_symbol = if progress < 0.25 { "❤️  [BOOM]" } else { "🖤  [tick]" };
    let gauge_width = 18;
    let cursor_pos = (progress * gauge_width as f64) as usize;
    let mut bar = vec!['-'; gauge_width];
    if cursor_pos < gauge_width {
        bar[cursor_pos] = '●';
    }
    let pulse_bar = bar.iter().collect::<String>();
    
    vec![
        Line::from(vec![
            Span::styled("Pulse: ", Style::default()),
            Span::styled(heart_symbol, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(format!(" [{}]", pulse_bar), Style::default().fg(Color::LightRed))),
    ]
}

fn draw_rules_panel(buffer: &mut Buffer, area: Rect, app: &App) {
    let center_block = Block::default()
        .title(" 🎮 ROOM CONFIG ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let rs = &app.room_settings;
    let is_host = app.is_local_player_host();
    let cursor = app.lobby_screen.cursor;

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
        .render(area, buffer);
}

fn draw_players_panel(buffer: &mut Buffer, area: Rect, app: &App) {
    let right_block = Block::default()
        .title(" 👥 PLAYERS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mut right_lines = vec![Line::from("")];

    right_lines.push(Line::from(vec![
        Span::styled(format!(" {} ", app.profile.skin), Style::default()),
        Span::styled(app.profile.name.as_str(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" 👑", Style::default().fg(Color::Yellow)),
    ]));
    right_lines.push(Line::from(""));

    // Mocked player
    right_lines.push(Line::from(vec![
        Span::styled(" 🐱 ", Style::default()),
        Span::styled("GigaPlayer", Style::default().fg(Color::Magenta)),
    ]));
    right_lines.push(Line::from(""));

    // Mocked player
    right_lines.push(Line::from(vec![
        Span::styled(" 🐸 ", Style::default()),
        Span::styled("Ribbit", Style::default().fg(Color::Yellow)),
    ]));

    Paragraph::new(right_lines)
        .block(right_block)
        .render(area, buffer);
}
