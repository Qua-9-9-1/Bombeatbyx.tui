use super::beat_fx;
use super::cell_style::{self, MapView};
use super::helpers::{get_emote_symbol, get_player_symbol};
use crate::local::app::{App, CELL_H, CELL_W};
use crate::ui::get_color_for_id;
use common::game::Cell;
use common::game::rhythm::is_in_hit_window;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

/// What every cell of this frame needs to be drawn.
struct Frame<'a> {
    view: MapView,
    origin: (u16, u16),
    ascii: bool,
    beat: u64,
    progress: f64,
    current_beat: u64,
    grid: &'a [Cell],
    width: usize,
}

pub fn draw_map(buffer: &mut Buffer, app: &App, ctx: &common::game::GameContext, rect: Rect) {
    let ascii = app.profile.ascii_mode;
    let beat = ctx.rhythm.beat_count;
    let progress = ctx.rhythm.progress();
    let window_open = is_in_hit_window(progress);
    let combo = ctx
        .state
        .players
        .iter()
        .find(|p| p.id == app.current_player_id)
        .map_or(0, |p| p.combo);
    let (border_style, border_type) = beat_fx::arena_border(window_open, combo);
    let map_box = Block::default()
        .title_top(beat_fx::feedback_title(app).alignment(Alignment::Center))
        .title_bottom(beat_fx::bar_title(beat, window_open, ascii).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style);
    map_box.render(rect, buffer);

    let arena_w = ctx.state.width as u16 * CELL_W;
    let arena_h = ctx.state.height as u16 * CELL_H;
    let offset_x = rect.width.saturating_sub(2).saturating_sub(arena_w) / 2;
    let offset_y = rect.height.saturating_sub(2).saturating_sub(arena_h) / 2;

    let frame = Frame {
        view: MapView::new(&ctx.state),
        origin: (rect.x + 1 + offset_x, rect.y + 1 + offset_y),
        ascii,
        beat,
        progress,
        current_beat: ctx.state.current_beat,
        grid: &ctx.state.grid,
        width: ctx.state.width,
    };

    for y in 0..ctx.state.height {
        for x in 0..ctx.state.width {
            draw_cell(buffer, &frame, ctx.state.grid[y * ctx.state.width + x], x, y);
        }
    }

    let mut sorted_players = ctx.state.players.clone();
    sorted_players.sort_by_key(|p| p.is_alive);

    for player in sorted_players.iter().filter(|p| !p.is_spectator) {
        let (gx, gy) = (
            (player.sub_x / CELL_W as i32) as usize,
            (player.sub_y / CELL_H as i32) as usize,
        );
        if gx >= ctx.state.width || gy >= ctx.state.height {
            continue;
        }
        let is_local = player.id == app.current_player_id;
        let bg = if is_local && window_open {
            beat_fx::player_glow(beat)
        } else {
            cell_style::floor(&frame.view, gx, gy, beat)
        };
        draw_player(buffer, &frame, player, bg);
    }
}

fn draw_cell(buffer: &mut Buffer, frame: &Frame, cell: Cell, x: usize, y: usize) {
    let ascii = frame.ascii;
    let (symbol, fg_color, bg_color): (String, Color, Color) = match cell {
        Cell::Empty => (
            "  ".to_string(),
            Color::Reset,
            cell_style::floor(&frame.view, x, y, frame.beat),
        ),
        Cell::Wall => {
            let sym = if ascii { "##" } else { "██" };
            (sym.to_string(), Color::Indexed(248), Color::Indexed(243))
        }
        Cell::Brick => {
            let (sym, fg, bg) = cell_style::brick(frame.view.is_cracked(x, y), ascii);
            (sym.to_string(), fg, bg)
        }
        Cell::Bomb { ticks_left, .. } => (
            cell_style::bomb_symbol(ascii).to_string(),
            Color::Yellow,
            cell_style::bomb_heat(ticks_left),
        ),
        Cell::Explosion { .. } => {
            // A blast lives one beat: play 💥 → 🔥 → ✨ over that beat
            // (wall-clock time would always land on the last frame).
            let anim = crate::ui::animation::Animation::explosion_expanding();
            let frame = anim.get_frame(std::time::Duration::from_millis(
                (frame.progress.clamp(0.0, 1.0) * 300.0) as u64,
            ));
            let sym = if ascii {
                match frame.symbol.as_str() {
                    "💥" => "##",
                    "🔥" => "**",
                    _ => "::",
                }
                .to_string()
            } else {
                frame.symbol.clone()
            };
            (sym, frame.fg_color, frame.bg_color)
        }
        Cell::Bonus {
            bonus_type,
            ticks_left,
        } => {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::ZERO);
            let anim =
                crate::ui::animation::Animation::bonus_blinking(bonus_type, ticks_left, ascii);
            let frame = anim.get_frame(elapsed);
            (frame.symbol.clone(), frame.fg_color, frame.bg_color)
        }
    };
    put(buffer, frame, x, y, &symbol, fg_color, bg_color);
}

/// Writes a two-column cell, clearing both columns first (wide emojis).
fn put(buffer: &mut Buffer, frame: &Frame, x: usize, y: usize, symbol: &str, fg: Color, bg: Color) {
    let out_x = frame.origin.0 + (x as u16 * CELL_W);
    let out_y = frame.origin.1 + (y as u16 * CELL_H);
    for dx in 0..CELL_W {
        buffer[(out_x + dx, out_y)]
            .set_symbol(" ")
            .set_style(Style::default().bg(bg));
    }
    buffer.set_string(out_x, out_y, symbol, Style::default().bg(bg).fg(fg));
}

fn draw_player(buffer: &mut Buffer, frame: &Frame, player: &common::game::Player, bg: Color) {
    let ascii = frame.ascii;
    let fg = get_color_for_id(player.id);
    let gx = (player.sub_x / CELL_W as i32) as usize;
    let gy = (player.sub_y / CELL_H as i32) as usize;

    if player.is_alive {
        let mut sym = get_player_symbol(&player.skin, player.is_alive, ascii);
        if let Some(ref emote) = player.active_emote {
            let not_expired = player
                .emote_until
                .is_none_or(|until| std::time::Instant::now() < until);
            if not_expired {
                sym = get_emote_symbol(emote, ascii);
            }
        }
        // Standing on a bomb: the cell keeps the bomb's heat color and
        // alternates player / bomb on the beat.
        if let Some(ticks_left) = frame.grid.get(gy * frame.width + gx).copied().and_then(cell_style::is_bomb) {
            let bomb_bg = cell_style::bomb_heat(ticks_left);
            if cell_style::shows_bomb_over_player(frame.progress) {
                put(buffer, frame, gx, gy, cell_style::bomb_symbol(ascii), Color::Yellow, bomb_bg);
            } else {
                put(buffer, frame, gx, gy, sym, fg, bomb_bg);
            }
            return;
        }
        let bg = if player.is_protected(frame.current_beat) {
            Color::Rgb(0, 95, 135)
        } else {
            bg
        };
        put(buffer, frame, gx, gy, sym, fg, bg);
    } else if let (Some((dx, dy)), Some(timer)) = (player.death_pos, player.respawn_timer) {
        let now = std::time::Instant::now();
        if now < timer {
            let time_remaining = timer.duration_since(now);
            let time_elapsed = std::time::Duration::from_secs(3).saturating_sub(time_remaining);
            let anim = crate::ui::animation::Animation::player_death(ascii);
            let death = anim.get_frame(time_elapsed);
            if death.symbol != "  " {
                let (cx, cy) = ((dx / CELL_W as i32) as usize, (dy / CELL_H as i32) as usize);
                put(buffer, frame, cx, cy, &death.symbol, death.fg_color, bg);
            }
        }
    }
}
