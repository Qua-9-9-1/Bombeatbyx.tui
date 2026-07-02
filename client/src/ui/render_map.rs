use crate::local::app::{App, CELL_H, CELL_W};
use common::game::Cell;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
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
        let p_grid_x = (player.sub_x / CELL_W as i32) as usize;
        let p_grid_y = (player.sub_y / CELL_H as i32) as usize;
        let idx = p_grid_y * ctx.state.width + p_grid_x;

        if let Cell::Bomb { .. } = ctx.state.grid[idx] {
            if player.is_alive {
                continue;
            }
        }

        draw_player(buffer, player, play_zone_x, play_zone_y, ascii);
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
) {
    let p_screen_x = play_zone_x + player.sub_x as u16;
    let p_screen_y = play_zone_y + player.sub_y as u16;
    let bg = Style::default().bg(Color::Indexed(234));

    let mut sym = get_player_symbol(&player.skin, player.is_alive, ascii);
    if player.is_alive {
        if let Some(ref emote) = player.active_emote {
            if let Some(until) = player.emote_until {
                if std::time::Instant::now() < until {
                    sym = get_emote_symbol(emote, ascii);
                }
            }
        }
    }

    let fg = get_color_from_str(&player.color);

    if player.is_alive {
        buffer.set_string(p_screen_x, p_screen_y, sym, bg.fg(fg));
    } else {
        buffer.set_string(p_screen_x, p_screen_y, sym, bg.fg(Color::Red));
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
