pub mod helpers;
pub mod particles;
pub mod podium;

use crate::local::app::App;
use helpers::draw_str_centered;
use particles::draw_confetti;
use podium::{draw_podium_column, draw_podium_player};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};
use std::time::Duration;

pub fn draw_victory_screen(buffer: &mut Buffer, area: Rect, app: &App) {
    let ascii = app.profile.ascii_mode;
    let elapsed_sec = app
        .victory_start_time
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);

    draw_confetti(buffer, area, elapsed_sec);

    let card_w = 70_u16.min(area.width);
    let card_h = 22_u16.min(area.height);
    let card_x = area.x + (area.width - card_w) / 2;
    let card_y = area.y + (area.height - card_h) / 2;

    let bg_color = Color::Indexed(234);
    let frame_style = Style::default().fg(Color::Yellow);

    for y in card_y..card_y + card_h {
        for x in card_x..card_x + card_w {
            let is_border =
                x == card_x || x == card_x + card_w - 1 || y == card_y || y == card_y + card_h - 1;
            if let Some(cell) = buffer.cell_mut((x, y)) {
                if is_border {
                    let sym = if x == card_x && y == card_y {
                        if ascii { "+" } else { "┌" }
                    } else if x == card_x + card_w - 1 && y == card_y {
                        if ascii { "+" } else { "┐" }
                    } else if x == card_x && y == card_y + card_h - 1 {
                        if ascii { "+" } else { "└" }
                    } else if x == card_x + card_w - 1 && y == card_y + card_h - 1 {
                        if ascii { "+" } else { "┘" }
                    } else if y == card_y || y == card_y + card_h - 1 {
                        if ascii { "-" } else { "─" }
                    } else {
                        if ascii { "|" } else { "│" }
                    };
                    cell.set_symbol(sym);
                    cell.set_style(frame_style);
                } else {
                    cell.set_symbol(" ");
                    cell.set_style(Style::default().bg(bg_color));
                }
            }
        }
    }

    let title_style = Style::default()
        .fg(if (elapsed_sec * 4.0) as usize % 2 == 0 {
            Color::Yellow
        } else {
            Color::Cyan
        })
        .add_modifier(Modifier::BOLD);
    let title_text = if ascii {
        "*** CHAMPIONSHIP PODIUM ***"
    } else {
        "🏆 CHAMPIONSHIP PODIUM 🏆"
    };
    draw_str_centered(
        buffer,
        card_x + card_w / 2,
        card_y + 1,
        title_text,
        title_style,
    );

    let final_state = match app.victory_final_state {
        Some(ref s) => s,
        None => return,
    };
    let mode_text = match final_state.mode {
        common::game::models::GameMode::Deathmatch => "Mode: Deathmatch Match",
        common::game::models::GameMode::Score => "Mode: Score Race",
    };
    draw_str_centered(
        buffer,
        card_x + card_w / 2,
        card_y + 2,
        mode_text,
        Style::default().fg(Color::DarkGray),
    );

    let mut ranked_players = final_state.players.clone();
    ranked_players.sort_by(|a, b| match final_state.mode {
        common::game::models::GameMode::Deathmatch => {
            let a_alive = a.lives > 0;
            let b_alive = b.lives > 0;
            if a_alive != b_alive {
                b_alive.cmp(&a_alive)
            } else if a_alive {
                b.lives
                    .cmp(&a.lives)
                    .then_with(|| b.score.cmp(&a.score))
                    .then_with(|| a.id.cmp(&b.id))
            } else {
                b.death_beat
                    .unwrap_or(0)
                    .cmp(&a.death_beat.unwrap_or(0))
                    .then_with(|| b.score.cmp(&a.score))
                    .then_with(|| a.id.cmp(&b.id))
            }
        }
        common::game::models::GameMode::Score => b
            .score
            .cmp(&a.score)
            .then_with(|| b.lives.cmp(&a.lives))
            .then_with(|| a.id.cmp(&b.id)),
    });

    let center_x = card_x + card_w / 2;
    let base_y = card_y + card_h - 4;

    let col_w = 14_u16;
    let gap = 2_u16;

    let first_x = center_x;
    let first_color = Color::Rgb(255, 215, 0);
    let first_height = 6_u16;

    let second_x = center_x - col_w - gap;
    let second_color = Color::Rgb(192, 192, 192);
    let second_height = 4_u16;

    let third_x = center_x + col_w + gap;
    let third_color = Color::Rgb(205, 127, 50);
    let third_height = 2_u16;

    draw_podium_column(
        buffer,
        second_x,
        base_y,
        second_height,
        col_w,
        "2nd Place",
        second_color,
        ascii,
    );

    draw_podium_column(
        buffer,
        first_x,
        base_y,
        first_height,
        col_w,
        "1st Place",
        first_color,
        ascii,
    );

    draw_podium_column(
        buffer,
        third_x,
        base_y,
        third_height,
        col_w,
        "3rd Place",
        third_color,
        ascii,
    );

    let bounce_offset_1st = ((elapsed_sec * 5.0).sin().abs() * 1.6) as u16;
    let bounce_offset_2nd = ((elapsed_sec * 4.5).sin().abs() * 1.1) as u16;
    let bounce_offset_3rd = ((elapsed_sec * 4.0).sin().abs() * 0.6) as u16;

    if let Some(p) = ranked_players.get(0) {
        draw_podium_player(
            buffer,
            first_x,
            base_y - first_height,
            bounce_offset_1st,
            p,
            true,
            ascii,
            final_state.mode,
        );
    }

    if let Some(p) = ranked_players.get(1) {
        draw_podium_player(
            buffer,
            second_x,
            base_y - second_height,
            bounce_offset_2nd,
            p,
            false,
            ascii,
            final_state.mode,
        );
    }

    if let Some(p) = ranked_players.get(2) {
        draw_podium_player(
            buffer,
            third_x,
            base_y - third_height,
            bounce_offset_3rd,
            p,
            false,
            ascii,
            final_state.mode,
        );
    }

    let footer_y = card_y + card_h - 2;
    let elapsed = app
        .victory_start_time
        .map(|t| t.elapsed())
        .unwrap_or(Duration::ZERO);

    if elapsed < Duration::from_millis(3000) {
        draw_str_centered(
            buffer,
            center_x,
            footer_y,
            "[ Wait... ]",
            Style::default().fg(Color::DarkGray),
        );
    } else {
        let flash = (elapsed_sec * 2.0) as usize % 2 == 0;
        let style = if flash {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        draw_str_centered(
            buffer,
            center_x,
            footer_y,
            "[ PRESS ANY KEY TO RETURN TO LOBBY ]",
            style,
        );
    }
}
