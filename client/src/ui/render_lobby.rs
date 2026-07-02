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

    draw_info_panel(buffer, columns[0], cursor, &app.room_settings, app.profile.ascii_mode);
    draw_rules_panel(buffer, columns[1], app);
    draw_players_panel(buffer, columns[2], app);
}

fn draw_info_panel(buffer: &mut Buffer, area: Rect, cursor: usize, rs: &RoomSettings, ascii: bool) {
    let title = if ascii { " [ INFO PANEL ] " } else { " ℹ️ INFO PANEL " };
    let info_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let mut info_lines = vec![Line::from("")];

    let header_width = if ascii { " [ MAP WIDTH ] " } else { "🗺️  MAP WIDTH" };
    let header_height = if ascii { " [ MAP HEIGHT ] " } else { "🗺️  MAP HEIGHT" };
    let header_bpm = if ascii { " [ BPM / TEMPO ] " } else { "⚡ BPM / TEMPO" };
    let header_sudden = if ascii { " [ SUDDEN DEATH ] " } else { "💀 SUDDEN DEATH" };
    let header_bonus = if ascii { " [ BONUS SPAWN ] " } else { "🎁 BONUS SPAWN" };
    let header_skin = if ascii { " [ CHARACTER SKIN ] " } else { "🎭 CHARACTER SKIN" };
    let header_start = if ascii { " [ START MATCH ] " } else { "🚀 START MATCH" };

    match cursor {
        0 => {
            info_lines.push(Line::from(Span::styled(header_width, Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
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
            info_lines.push(Line::from(Span::styled(header_height, Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
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
            info_lines.push(Line::from(Span::styled(header_bpm, Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from("Controls the speed of the"));
            info_lines.push(Line::from("rhythm beat count."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Current Speed Meter:", Style::default().add_modifier(Modifier::UNDERLINED))));
            
            let pulse_lines = format_bpm_info(rs.bpm, ascii);
            info_lines.extend(pulse_lines);
            
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Tip:", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("Higher BPM requires faster", Style::default().fg(Color::DarkGray))));
            info_lines.push(Line::from(Span::styled("inputs to get PERFECT accuracy.", Style::default().fg(Color::DarkGray))));
        }
        3 => {
            info_lines.push(Line::from(Span::styled(header_sudden, Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
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
            info_lines.push(Line::from(Span::styled(header_bonus, Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
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
            info_lines.push(Line::from(Span::styled(header_skin, Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(if ascii { "Select your character skin." } else { "Select your emoji skin." }));
            info_lines.push(Line::from("This is how other players"));
            info_lines.push(Line::from("will see you on the map."));
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(Span::styled("Options:", Style::default().add_modifier(Modifier::UNDERLINED))));
            if ascii {
                info_lines.push(Line::from("RO (Robot), CA (Cat), FR (Frog),"));
                info_lines.push(Line::from("FO (Fox), PE (Penguin)."));
            } else {
                info_lines.push(Line::from("🤖 Robot, 🐱 Cat, 🐸 Frog,"));
                info_lines.push(Line::from("🦊 Fox, 🐧 Penguin."));
            }
        }
        6 => {
            info_lines.push(Line::from(Span::styled(header_start, Style::default().add_modifier(Modifier::BOLD).fg(Color::Green))));
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

fn format_bpm_info(bpm: f64, ascii: bool) -> Vec<Line<'static>> {
    let elapsed_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_millis();
    let beat_duration_ms = (60.0 / bpm * 1000.0) as u128;
    let progress = (elapsed_ms % beat_duration_ms) as f64 / beat_duration_ms as f64;
    
    let anim = crate::ui::animation::Animation::heart_beating(bpm);
    let frame = anim.get_frame(std::time::Duration::from_millis(elapsed_ms as u64));
    
    let pulse_symbol = if ascii {
        if frame.symbol.contains("❤️") {
            "<3 [BOOM]"
        } else if frame.symbol.contains("💖") {
            "<3 [boom]"
        } else {
            ".. [TICK]"
        }
    } else {
        &frame.symbol
    };

    let gauge_width = 18;
    let cursor_pos = (progress * gauge_width as f64) as usize;
    let mut bar = vec!['-'; gauge_width];
    if cursor_pos < gauge_width {
        bar[cursor_pos] = if ascii { 'X' } else { '●' };
    }
    let pulse_bar = bar.iter().collect::<String>();
    
    vec![
        Line::from(vec![
            Span::styled("Pulse: ", Style::default()),
            Span::styled(pulse_symbol.to_string(), Style::default().fg(frame.fg_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(format!(" [{}]", pulse_bar), Style::default().fg(Color::LightRed))),
    ]
}

fn draw_rules_panel(buffer: &mut Buffer, area: Rect, app: &App) {
    let ascii = app.profile.ascii_mode;
    let title = if ascii { " [ ROOM CONFIG ] " } else { " 🎮 ROOM CONFIG " };
    let center_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let rs = &app.room_settings;
    let is_host = app.is_local_player_host();
    let cursor = app.lobby_screen.cursor;

    let skin_name = get_skin_label(&app.profile.skin, ascii);

    let items = [
        if is_host { format!("Map Width     : < {} >", rs.width) } else { format!("Map Width     : {}", rs.width) },
        if is_host { format!("Map Height    : < {} >", rs.height) } else { format!("Map Height    : {}", rs.height) },
        if is_host { format!("BPM (Tempo)   : < {:.0} >", rs.bpm) } else { format!("BPM (Tempo)   : {:.0}", rs.bpm) },
        if is_host { format!("Sudden Death  : < {} >", if rs.sudden_death { "ON" } else { "OFF" }) } else { format!("Sudden Death  : {}", if rs.sudden_death { "ON" } else { "OFF" }) },
        if is_host { format!("Bonus Spawn   : < Every {} beats >", rs.bonus_every) } else { format!("Bonus Spawn   : Every {} beats", rs.bonus_every) },
        format!("Your Skin     : < {} >", skin_name),
        " [ START GAME ] ".to_string(),
    ];

    let mut center_lines = vec![Line::from("")];
    if is_host {
        let label = if ascii { "  == HOST SETTINGS (Q/D to adjust) ==" } else { "  ★ HOST SETTINGS (Q/D to adjust) ★" };
        center_lines.push(Line::from(Span::styled(label, Style::default().fg(Color::Yellow))));
    } else {
        let label = if ascii { "  ... WAITING FOR HOST ..." } else { "  ⏱️ WAITING FOR HOST..." };
        center_lines.push(Line::from(Span::styled(label, Style::default().fg(Color::DarkGray))));
    }
    center_lines.push(Line::from(""));

    let arrow_l = if ascii { " => " } else { " ► " };
    let arrow_r = if ascii { " <= " } else { " ◄" };

    for (idx, item) in items.iter().enumerate() {
        if idx == 6 && !is_host { continue; }

        if idx == cursor {
            if idx == 6 {
                let text = if ascii { format!(" =>> {} <<=", item) } else { format!(" ► 🔥 {} 🔥 ◄", item) };
                center_lines.push(Line::from(Span::styled(text, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
            } else {
                center_lines.push(Line::from(vec![
                    Span::styled(arrow_l, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(item.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(arrow_r, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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
    let ascii = app.profile.ascii_mode;
    let title = if ascii { " [ PLAYERS ] " } else { " 👥 PLAYERS " };
    let right_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mut right_lines = vec![Line::from("")];

    let player_skin = get_player_skin_cell(&app.profile.skin, ascii);
    let host_tag = if ascii { " (Host)" } else { " 👑" };

    right_lines.push(Line::from(vec![
        Span::styled(player_skin, Style::default()),
        Span::styled(app.profile.name.as_str(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(host_tag, Style::default().fg(Color::Yellow)),
    ]));
    right_lines.push(Line::from(""));

    let p2_skin = get_player_skin_cell("🐱", ascii);
    right_lines.push(Line::from(vec![
        Span::styled(p2_skin, Style::default()),
        Span::styled("GigaPlayer", Style::default().fg(Color::Magenta)),
    ]));
    right_lines.push(Line::from(""));

    let p3_skin = get_player_skin_cell("🐸", ascii);
    right_lines.push(Line::from(vec![
        Span::styled(p3_skin, Style::default()),
        Span::styled("Ribbit", Style::default().fg(Color::Yellow)),
    ]));

    Paragraph::new(right_lines)
        .block(right_block)
        .render(area, buffer);
}

fn get_skin_label(skin: &str, ascii: bool) -> String {
    if ascii {
        match skin {
            "🤖" => "Robot [RO]".to_string(),
            "🐱" => "Cat [CA]".to_string(),
            "🐸" => "Frog [FR]".to_string(),
            "🦊" => "Fox [FO]".to_string(),
            "🐧" => "Penguin [PE]".to_string(),
            _ => format!("[{}]", skin),
        }
    } else {
        skin.to_string()
    }
}

fn get_player_skin_cell(skin: &str, ascii: bool) -> String {
    if ascii {
        match skin {
            "🤖" => "[RO] ".to_string(),
            "🐱" => "[CA] ".to_string(),
            "🐸" => "[FR] ".to_string(),
            "🦊" => "[FO] ".to_string(),
            "🐧" => "[PE] ".to_string(),
            _ => format!("[{}] ", skin),
        }
    } else {
        format!(" {} ", skin)
    }
}
