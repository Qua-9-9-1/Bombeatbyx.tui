use crate::game::app::{App, CELL_H, CELL_W};
use crate::game::rhythm::GaugeSkin;
use common::game::Cell;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let tui_area = frame.area();
    let buffer = frame.buffer_mut();

    let map_w = (app.game_state.width as u16 * CELL_W) + 2;
    let map_height = (app.game_state.height as u16 * CELL_H) + 2;
    let total_needed_height = map_height + 3;

    if tui_area.width < map_w || tui_area.height < total_needed_height {
        let msg = Paragraph::new("Agrandissez le terminal pour jouer !")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        msg.render(tui_area, buffer);
        return;
    }

    let start_x = (tui_area.width - map_w) / 2;
    let start_y = (tui_area.height - total_needed_height) / 2;

    let map_box = Block::default()
        .title(" BOMBOMBYX ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
    let map_rect = Rect::new(start_x, start_y, map_w, map_height);
    map_box.render(map_rect, buffer);

    let play_zone_x = start_x + 1;
    let play_zone_y = start_y + 1;

    for y in 0..app.game_state.height {
        for x in 0..app.game_state.width {
            let cell = app.game_state.grid[y * app.game_state.width + x];

            let player = &app.game_state.players[0];
            let p_grid_x = (player.sub_x / CELL_W as i32) as usize;
            let p_grid_y = (player.sub_y / CELL_H as i32) as usize;

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

            if is_bomb {
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

        let p_screen_x = play_zone_x + player.sub_x as u16;
        let p_screen_y = play_zone_y + player.sub_y as u16;
        let bg = Style::default().bg(Color::Indexed(234));

        if player.is_alive {
            buffer.set_string(p_screen_x, p_screen_y, "🤖", bg.fg(Color::Cyan));
        } else {
            buffer.set_string(p_screen_x, p_screen_y, "💀", bg.fg(Color::Red));
        }
    }

    let gauge_area = Rect::new(start_x, start_y + map_height + 1, map_w, 1);
    draw_rhythm_gauge(buffer, app, gauge_area);

    if app.controls.mode == crate::game::controls::ControlMode::Menu {
        let menu_w = 30;
        let menu_h = 9;
        let menu_x = (tui_area.width.saturating_sub(menu_w)) / 2;
        let menu_y = (tui_area.height.saturating_sub(menu_h)) / 2;
        let menu_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);

        let menu_block = Block::default()
            .title(" 🛠️ PAUSE / MENU 🛠️ ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::Yellow));

        let mut menu_content = String::new();
        let items = [
            "  Continuer la partie  ",
            "  Configuration Audio  ",
            "  Quitter vers le Bureau  ",
        ];

        for (idx, item) in items.iter().enumerate() {
            if idx == app.controls.menu_cursor {
                menu_content.push_str(&format!("► 📦 {} ◄\n", item.trim()));
            } else {
                menu_content.push_str(&format!("   {}   \n", item.trim()));
            }
        }

        let p_menu = Paragraph::new(menu_content)
            .block(menu_block)
            .alignment(Alignment::Center);

        p_menu.render(menu_rect, buffer);
    }
}

pub fn draw_rhythm_gauge(buffer: &mut Buffer, app: &App, area: Rect) {
    let progress = app.rhythm.progress();
    let width = 28_usize;

    let gauge_text = match app.rhythm.skin {
        GaugeSkin::NecroDancer => {
            let center = width / 2;
            let distance = ((1.0 - progress) * (center as f64)).round() as usize;

            let mut bar = vec![' '; width];
            bar[center] = 'O';

            let left_pos = center.saturating_sub(distance);
            let right_pos = (center + distance).min(width - 1);

            if distance > 0 {
                bar[left_pos] = '>';
                bar[right_pos] = '<';
            } else {
                bar[center] = 'X';
            }
            format!(" [{}] ", bar.iter().collect::<String>())
        }
        GaugeSkin::Undertale => {
            let mut bar = vec!['-'; width];
            let target_pos = width / 2;
            bar[target_pos] = '|';

            let cursor_pos = ((progress * (width as f64)).round() as usize).clamp(0, width - 1);
            if cursor_pos == target_pos {
                bar[cursor_pos] = 'X';
            } else {
                bar[cursor_pos] = '█';
            }
            format!(" [{}] ", bar.iter().collect::<String>())
        }
    };

    let color = if progress > 0.85 || progress < 0.15 {
        Color::Green
    } else {
        Color::DarkGray
    };

    let paragraph = Paragraph::new(gauge_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD));

    paragraph.render(area, buffer);
}
