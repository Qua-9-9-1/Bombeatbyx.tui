use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use common::game::Cell;
use crate::game::app::{App, CELL_H, CELL_W};

pub fn draw(frame: &mut Frame, app: &App) {
    let tui_area = frame.area();

    let map_w = (app.game_state.width as i32 * CELL_W) + 2;
    let map_height = (app.game_state.height as i32 * CELL_H) + 2;

    if tui_area.width < map_w as u16 || tui_area.height < map_height as u16 {
        let msg = Paragraph::new("Agrandissez le terminal pour jouer !")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        frame.render_widget(msg, tui_area);
        return;
    }

    let start_x = (tui_area.width - map_w as u16) / 2;
    let start_y = (tui_area.height - map_height as u16) / 2;

    let map_box = Block::default()
        .title(" Bombombyx")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
    let map_rect = Rect::new(start_x, start_y, map_w as u16, map_height as u16);
    frame.render_widget(map_box, map_rect);

    let play_zone_x = start_x + 1;
    let play_zone_y = start_y + 1;

    let buffer = frame.buffer_mut();

    for y in 0..app.game_state.height {
        for x in 0..app.game_state.width {
            let cell = app.game_state.grid[y * app.game_state.width + x];
            
            let player = &app.game_state.players[0];
            let p_grid_x = ((player.sub_x + 1) / CELL_W) as usize;
            let p_grid_y = (player.sub_y / CELL_H) as usize;
            
            let player_is_here = player.is_alive && x == p_grid_x && y == p_grid_y;
            let is_bomb = matches!(cell, Cell::Bomb { .. });

            let bg_color = if player_is_here && is_bomb {
                Color::Yellow
            } else {
                match cell {
                    Cell::Empty => Color::Indexed(234),
                    Cell::Wall => Color::Indexed(243),
                    Cell::Brick => Color::Rgb(139, 69, 19),
                    Cell::Explosion { .. } => Color::Rgb(255, 69, 0),
                    Cell::Bomb { ticks_left, .. } => {
                        if ticks_left < 60 && (ticks_left / 10) % 2 == 0 { Color::Red } else { Color::Indexed(234) }
                    }
                }
            };

            for bh in 0..CELL_H {
                for bw in 0..CELL_W {
                    let out_x = play_zone_x + (x as u16 * CELL_W as u16) + bw as u16;
                    let out_y = play_zone_y + (y as u16 * CELL_H as u16) + bh as u16;
                    
                    let cell_pixel = &mut buffer[(out_x, out_y)];
                    cell_pixel.set_symbol(" ");
                    cell_pixel.set_style(Style::default().bg(bg_color));
                }
            }

            let cell_screen_x = play_zone_x + (x as u16 * CELL_W as u16);
            let cell_screen_y = play_zone_y + (y as u16 * CELL_H as u16);
            let cell_rect = Rect::new(cell_screen_x, cell_screen_y, CELL_W as u16, CELL_H as u16);

            if is_bomb {
                let p = Paragraph::new("💣").alignment(Alignment::Center).style(Style::default().bg(bg_color));
                p.render(cell_rect, buffer);
            } else {
                match cell {
                    Cell::Explosion { .. } => {
                        let p = Paragraph::new("💥").alignment(Alignment::Center).style(Style::default().bg(bg_color));
                        p.render(cell_rect, buffer);
                    }
                    _ => {}
                }
            }
        }
    }

    for player in &app.game_state.players {
        let p_grid_x = ((player.sub_x + 1) / CELL_W) as usize;
        let p_grid_y = (player.sub_y / CELL_H) as usize;
        let idx = p_grid_y * app.game_state.width + p_grid_x;

        if let Cell::Bomb { .. } = app.game_state.grid[idx] {
            if player.is_alive { continue; } 
        }

        let p_screen_x = play_zone_x + player.sub_x as u16;
        let p_screen_y = play_zone_y + player.sub_y as u16;
        let p_rect = Rect::new(p_screen_x, p_screen_y, CELL_W as u16, CELL_H as u16);

        let avatar = if player.is_alive { "🤖" } else { "💀" };
        
        let p = Paragraph::new(avatar)
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Indexed(234)));
        p.render(p_rect, buffer);
    }
}