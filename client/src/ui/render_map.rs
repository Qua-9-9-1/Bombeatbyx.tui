use crate::local::app::{App, CELL_H, CELL_W};
use common::game::Cell;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_map(buffer: &mut Buffer, _app: &App, ctx: &common::game::GameContext, rect: Rect) {
    let ascii = _app.profile.ascii_mode;
    let title = if ascii { " [ BOMBOMBYX ] " } else { " BOMBOMBYX " };
    let map_box = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
    map_box.render(rect, buffer);

    let play_zone_x = rect.x + 1;
    let play_zone_y = rect.y + 1;

    for y in 0..ctx.state.height {
        for x in 0..ctx.state.width {
            let cell = ctx.state.grid[y * ctx.state.width + x];

            let player_is_here = ctx.state.players.iter()
                .find(|p| p.id == _app.current_player_id)
                .map(|p| p.is_alive && x == (p.sub_x / CELL_W as i32) as usize && y == (p.sub_y / CELL_H as i32) as usize)
                .unwrap_or(false);
            let is_bomb = matches!(cell, Cell::Bomb { .. });

            draw_cell(
                buffer,
                cell,
                x,
                y,
                play_zone_x,
                play_zone_y,
                player_is_here && is_bomb,
                ascii,
            );
        }
    }

    for player in &ctx.state.players {
        if player.is_spectator {
            continue;
        }
        let p_grid_x = (player.sub_x / CELL_W as i32) as usize;
        let p_grid_y = (player.sub_y / CELL_H as i32) as usize;
        if p_grid_x >= ctx.state.width || p_grid_y >= ctx.state.height {
            continue;
        }
        let idx = p_grid_y * ctx.state.width + p_grid_x;

        if let Cell::Bomb { .. } = ctx.state.grid[idx] {
            if player.is_alive {
                continue;
            }
        }

        draw_player(buffer, player, play_zone_x, play_zone_y, ascii, Some(ctx.rhythm.beat_count));
    }
}

fn draw_cell(
    buffer: &mut Buffer,
    cell: Cell,
    x: usize,
    y: usize,
    play_zone_x: u16,
    play_zone_y: u16,
    player_under_bomb: bool,
    ascii: bool,
) {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO);

    let (symbol, fg_color, bg_color) = match cell {
        Cell::Empty => ("  ".to_string(), Color::Reset, Color::Indexed(234)),
        Cell::Wall => {
            let sym = if ascii { "##" } else { "██" };
            (sym.to_string(), Color::Indexed(248), Color::Indexed(243))
        }
        Cell::Brick => {
            let sym = if ascii { "[]" } else { "░░" };
            (sym.to_string(), Color::Rgb(205, 133, 63), Color::Rgb(139, 69, 19))
        }
        Cell::Bomb { .. } => {
            let anim = crate::ui::animation::Animation::bomb_pulsing();
            let frame = anim.get_frame(elapsed);
            let sym = if ascii { "()".to_string() } else { frame.symbol.clone() };
            (sym, frame.fg_color, frame.bg_color)
        }
        Cell::Explosion { .. } => {
            let anim = crate::ui::animation::Animation::explosion_expanding();
            let frame = anim.get_frame(elapsed);
            let sym = if ascii {
                if frame.symbol == "💥" {
                    "##".to_string()
                } else if frame.symbol == "🔥" {
                    "**".to_string()
                } else {
                    "::".to_string()
                }
            } else {
                frame.symbol.clone()
            };
            (sym, frame.fg_color, frame.bg_color)
        }
        Cell::Bonus(b_type) => {
            let sym = match b_type {
                common::game::models::BonusType::BombQty => if ascii { "B+" } else { "💣" },
                common::game::models::BonusType::BombRange => if ascii { "R+" } else { "🔥" },
                common::game::models::BonusType::Shield => if ascii { "S+" } else { "🛡️" },
            };
            (sym.to_string(), Color::Cyan, Color::Indexed(234))
        }
    };

    let bg_color = if player_under_bomb {
        Color::Yellow
    } else {
        bg_color
    };

    let out_x = play_zone_x + (x as u16 * CELL_W);
    let out_y = play_zone_y + (y as u16 * CELL_H);

    buffer[(out_x, out_y)]
        .set_symbol(" ")
        .set_style(Style::default().bg(bg_color));
    buffer[(out_x + 1, out_y)]
        .set_symbol(" ")
        .set_style(Style::default().bg(bg_color));

    buffer.set_string(out_x, out_y, &symbol, Style::default().bg(bg_color).fg(fg_color));
}

fn draw_player(
    buffer: &mut Buffer,
    player: &common::game::Player,
    play_zone_x: u16,
    play_zone_y: u16,
    ascii: bool,
    current_beat: Option<u64>,
) {
    let bg = Style::default().bg(Color::Indexed(234));
    let fg = get_color_from_str(&player.color);

    if player.is_alive {
        let p_screen_x = play_zone_x + player.sub_x as u16;
        let p_screen_y = play_zone_y + player.sub_y as u16;
        let mut sym = get_player_symbol(&player.skin, player.is_alive, ascii);
        if let Some(beat) = current_beat {
            if player.shield_until_beat == Some(beat) {
                sym = if ascii { "SH" } else { "🛡️" };
            }
        }
        if player.shield_until_beat.is_none() || player.shield_until_beat != current_beat {
            if let Some(ref emote) = player.active_emote {
                if let Some(until) = player.emote_until {
                    if std::time::Instant::now() < until {
                        sym = get_emote_symbol(emote, ascii);
                    }
                }
            }
        }
        buffer.set_string(p_screen_x, p_screen_y, sym, bg.fg(fg));
    } else {
        if let (Some((dx, dy)), Some(timer)) = (player.death_pos, player.respawn_timer) {
            let now = std::time::Instant::now();
            if now < timer {
                let time_remaining = timer.duration_since(now);
                let time_elapsed = std::time::Duration::from_secs(3).saturating_sub(time_remaining);

                let anim = crate::ui::animation::Animation::player_death(ascii);
                let frame = anim.get_frame(time_elapsed);

                if frame.symbol != "  " {
                    let p_screen_x = play_zone_x + dx as u16;
                    let p_screen_y = play_zone_y + dy as u16;
                    buffer.set_string(p_screen_x, p_screen_y, &frame.symbol, bg.fg(frame.fg_color));
                }
            }
        }
    }
}

fn get_emote_symbol(emote: &str, ascii: bool) -> &str {
    if ascii {
        match emote {
            "👋" => "HI",
            "✌" | "✌️" => "VI",
            "🖕" => "FU",
            "👍" => "OK",
            _ => emote,
        }
    } else {
        emote
    }
}

fn get_player_symbol(skin: &str, is_alive: bool, ascii: bool) -> &str {
    if !is_alive {
        return if ascii { "XX" } else { "💀" };
    }
    if ascii {
        match skin {
            "🤖" => "RO",
            "🐱" => "CA",
            "🐸" => "FR",
            "🦊" => "FO",
            "🐧" => "PE",
            _ => "PL",
        }
    } else {
        skin
    }
}

fn get_color_from_str(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        "yellow" => Color::Yellow,
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "white" => Color::White,
        _ => Color::White,
    }
}

pub fn draw_game_sidebar(buffer: &mut Buffer, app: &App, ctx: &common::game::GameContext, rect: Rect) {
    let ascii = app.profile.ascii_mode;
    let title = if ascii { " [ PLAYERS ] " } else { " 👥 PLAYERS " };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    block.render(rect, buffer);

    let mut lines = vec![Line::from("")];
    let players = &ctx.state.players;
    let mode = ctx.state.mode;

    let mut sorted_players = players.clone();
    sorted_players.sort_by(|a, b| {
        if a.is_spectator != b.is_spectator {
            return a.is_spectator.cmp(&b.is_spectator);
        }
        if a.is_spectator {
            return a.id.cmp(&b.id);
        }
        match mode {
            common::game::models::GameMode::Deathmatch => {
                b.lives.cmp(&a.lives)
                    .then_with(|| b.score.cmp(&a.score))
                    .then_with(|| a.id.cmp(&b.id))
            }
            common::game::models::GameMode::Score => {
                b.score.cmp(&a.score)
                    .then_with(|| b.lives.cmp(&a.lives))
                    .then_with(|| a.id.cmp(&b.id))
            }
        }
    });

    for player in sorted_players {
        let icon = if player.is_spectator {
            if ascii { "EY ".to_string() } else { "👀 ".to_string() }
        } else if !player.is_alive {
            if player.lives == 0 {
                if ascii { "XX ".to_string() } else { "🪦  ".to_string() }
            } else {
                if ascii { "XX ".to_string() } else { "💀 ".to_string() }
            }
        } else {
            if ascii {
                match player.skin.as_str() {
                    "🤖" => "RO ".to_string(),
                    "🐱" => "CA ".to_string(),
                    "🐸" => "FR ".to_string(),
                    "🦊" => "FO ".to_string(),
                    "🐧" => "PE ".to_string(),
                    _ => "PL ".to_string(),
                }
            } else {
                format!("{} ", player.skin)
            }
        };

        let fg_color = get_color_from_str(&player.color);
        let name = if player.name.len() > 14 {
            format!("{}..", &player.name[..12])
        } else {
            player.name.clone()
        };

        let second_item_case = match player.second_item {
            Some(common::game::models::SecondItem::Shield) => if ascii { " [SH]" } else { " [🛡️]" },
            None => " [  ]",
        };

        let spans = vec![
            Span::styled(icon, Style::default()),
            Span::styled(name, Style::default().fg(fg_color).add_modifier(Modifier::BOLD)),
            Span::styled(second_item_case, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ];

        lines.push(Line::from(spans));

        if player.is_alive && !player.is_spectator {
            let mut line1_spans = vec![Span::raw("  ")];
            match mode {
                common::game::models::GameMode::Deathmatch => {
                    let heart = if ascii { "L:" } else { "❤️" };
                    line1_spans.push(Span::styled(format!("Lives: {} {}", heart, player.lives), Style::default().fg(Color::Red)));
                }
                common::game::models::GameMode::Score => {
                    let star = if ascii { "S:" } else { "⭐" };
                    line1_spans.push(Span::styled(format!("Score: {} {}", star, player.score), Style::default().fg(Color::Yellow)));
                }
            }
            lines.push(Line::from(line1_spans));

            let mut line2_spans = vec![Span::raw("  ")];
            let combo_emoji = if player.combo == 0 {
                "".to_string()
            } else if player.combo < 5 {
                if ascii { " +".to_string() } else { " ⚡".to_string() }
            } else if player.combo < 10 {
                if ascii { " *".to_string() } else { " 🔥".to_string() }
            } else if player.combo < 20 {
                if ascii { " !".to_string() } else { " 💥".to_string() }
            } else {
                if ascii { " K".to_string() } else { " 👑".to_string() }
            };
            line2_spans.push(Span::styled(format!("Combo: {}{}", player.combo, combo_emoji), Style::default().fg(Color::LightRed)));
            lines.push(Line::from(line2_spans));

            let mut line3_spans = vec![Span::raw("  ")];
            let items_str = if player.collected_bonuses.is_empty() {
                if ascii { "None".to_string() } else { "🚫 None".to_string() }
            } else {
                if ascii {
                    player.collected_bonuses.iter().map(|b| {
                        match b.as_str() {
                            "💣" => "B",
                            "🔥" => "R",
                            _ => "?",
                        }
                    }).collect::<Vec<_>>().join(" ")
                } else {
                    player.collected_bonuses.join(" ")
                }
            };
            line3_spans.push(Span::styled(format!("Items: {}", items_str), Style::default().fg(Color::Cyan)));
            lines.push(Line::from(line3_spans));
        } else {
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
    }

    let inner_rect = rect.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
    Paragraph::new(lines).render(inner_rect, buffer);
}

pub fn draw_local_player_stats(buffer: &mut Buffer, app: &App, ctx: &common::game::GameContext, rect: Rect) {
    let ascii = app.profile.ascii_mode;
    let title = if ascii { " [ MY STATUS ] " } else { " ⚡ MY STATUS " };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    block.render(rect, buffer);

    if let Some(player) = ctx.state.players.iter().find(|p| p.id == app.current_player_id) {
        let mut line1_spans = vec![];
        match ctx.state.mode {
            common::game::models::GameMode::Deathmatch => {
                let heart = if ascii { "L:" } else { "❤️" };
                line1_spans.push(Span::styled(format!(" Lives: {} {}", heart, player.lives), Style::default().fg(Color::Red)));
            }
            common::game::models::GameMode::Score => {
                let star = if ascii { "S:" } else { "⭐" };
                line1_spans.push(Span::styled(format!(" Score: {} {}", star, player.score), Style::default().fg(Color::Yellow)));
            }
        }

        line1_spans.extend(vec![
            Span::styled(" | Bombs: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}/{}", player.active_bombs, player.max_bombs), Style::default().fg(Color::White)),
            Span::styled(" | Range: ", Style::default().fg(Color::DarkGray)),
            Span::styled(player.bomb_range.to_string(), Style::default().fg(Color::White)),
        ]);

        let items_str = if player.collected_bonuses.is_empty() {
            if ascii { "None".to_string() } else { "🚫 None".to_string() }
        } else {
            if ascii {
                player.collected_bonuses.iter().map(|b| {
                    match b.as_str() {
                        "💣" => "B",
                        "🔥" => "R",
                        _ => "?",
                    }
                }).collect::<Vec<_>>().join(" ")
            } else {
                player.collected_bonuses.join(" ")
            }
        };

        let second_item_case = match player.second_item {
            Some(common::game::models::SecondItem::Shield) => if ascii { "SH" } else { "🛡️" },
            None => "",
        };

        let line2_spans = vec![
            Span::styled(" Items: ", Style::default().fg(Color::DarkGray)),
            Span::styled(items_str, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" | Item 2: ", Style::default().fg(Color::DarkGray)),
            Span::styled(second_item_case, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ];

        let inner_rect = rect.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
        
        let buffer_lines = vec![
            Line::from(line1_spans),
            Line::from(line2_spans),
        ];

        Paragraph::new(buffer_lines).render(inner_rect, buffer);
    }
}
