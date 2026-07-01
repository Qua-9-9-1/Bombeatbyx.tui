use crate::local::app::{App, CELL_H, CELL_W};
use common::game::Cell;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

pub fn draw_map(buffer: &mut Buffer, app: &App, rect: Rect) {
    let map_box = Block::default()
        .title(" BOMBOMBYX ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
    map_box.render(rect, buffer);

    let play_zone_x = rect.x + 1;
    let play_zone_y = rect.y + 1;

    for y in 0..app.game_state.height {
        for x in 0..app.game_state.width {
            let cell = app.game_state.grid[y * app.game_state.width + x];

            let player = &app.game_state.players[0];
            let p_grid_x = (player.sub_x / CELL_W as i32) as usize;
            let p_grid_y = (player.sub_y / CELL_H as i32) as usize;

            let player_is_here = player.is_alive && x == p_grid_x && y == p_grid_y;
            let is_bomb = matches!(cell, Cell::Bomb { .. });

            draw_cell(
                buffer,
                cell,
                x,
                y,
                play_zone_x,
                play_zone_y,
                player_is_here && is_bomb,
            );
        }
    }

    for player in &app.game_state.players {
        let p_grid_x = (player.sub_x / CELL_W as i32) as usize;
        let p_grid_y = (player.sub_y / CELL_H as i32) as usize;
        let idx = p_grid_y * app.game_state.width + p_grid_x;

        if let Cell::Bomb { .. } = app.game_state.grid[idx] {
            if player.is_alive {
                continue;
            }
        }

        draw_player(buffer, player, play_zone_x, play_zone_y);
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
) {
    let bg_color = if player_under_bomb {
        Color::Yellow
    } else {
        match cell {
            Cell::Empty => Color::Indexed(234),
            Cell::Wall => Color::Indexed(243),
            Cell::Brick => Color::Rgb(139, 69, 19),
            Cell::Explosion { .. } => Color::Rgb(255, 69, 0),
            Cell::Bomb { ticks_left, .. } => {
                if ticks_left <= 1 {
                    Color::Red
                } else {
                    Color::Indexed(234)
                }
            }
        }
    };

    let out_x = play_zone_x + (x as u16 * CELL_W);
    let out_y = play_zone_y + (y as u16 * CELL_H);

    buffer[(out_x, out_y)]
        .set_symbol(" ")
        .set_style(Style::default().bg(bg_color));
    buffer[(out_x + 1, out_y)]
        .set_symbol(" ")
        .set_style(Style::default().bg(bg_color));

    if let Cell::Bomb { .. } = cell {
        buffer.set_string(out_x, out_y, "💣", Style::default().bg(bg_color));
    } else {
        match cell {
            Cell::Wall => {
                buffer.set_string(
                    out_x,
                    out_y,
                    "██",
                    Style::default().bg(bg_color).fg(Color::Indexed(248)),
                );
            }
            Cell::Brick => {
                buffer.set_string(
                    out_x,
                    out_y,
                    "░░",
                    Style::default().bg(bg_color).fg(Color::Rgb(205, 133, 63)),
                );
            }
            Cell::Explosion { .. } => {
                buffer.set_string(out_x, out_y, "💥", Style::default().bg(bg_color));
            }
            _ => {}
        }
    }
}

fn draw_player(
    buffer: &mut Buffer,
    player: &common::game::Player,
    play_zone_x: u16,
    play_zone_y: u16,
) {
    let p_screen_x = play_zone_x + player.sub_x as u16;
    let p_screen_y = play_zone_y + player.sub_y as u16;
    let bg = Style::default().bg(Color::Indexed(234));

    if player.is_alive {
        buffer.set_string(p_screen_x, p_screen_y, "🤖", bg.fg(Color::Cyan));
    } else {
        buffer.set_string(p_screen_x, p_screen_y, "💀", bg.fg(Color::Red));
    }
}
