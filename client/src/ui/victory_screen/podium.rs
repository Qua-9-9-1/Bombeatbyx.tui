use crate::ui::victory_screen::helpers::{
    draw_str_centered, get_color_for_id, get_player_skin_cell,
};
use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
};

pub fn draw_podium_column(
    buffer: &mut Buffer,
    x_center: u16,
    base_y: u16,
    height: u16,
    width: u16,
    label: &str,
    col_color: Color,
    ascii: bool,
) {
    let symbol = if ascii { "#" } else { "█" };

    for h in 0..height {
        let y = base_y - h;
        let left_x = x_center - width / 2;
        let right_x = x_center + width / 2 - 1;

        for x in left_x..=right_x {
            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.set_symbol(symbol);
                cell.set_style(Style::default().fg(col_color).bg(Color::Indexed(234)));
            }
        }
    }

    let label_y = base_y - height + 1;
    draw_str_centered(
        buffer,
        x_center,
        label_y,
        label,
        Style::default()
            .fg(Color::Indexed(232))
            .bg(col_color)
            .add_modifier(Modifier::BOLD),
    );
}

pub fn draw_podium_player(
    buffer: &mut Buffer,
    x_center: u16,
    top_column_y: u16,
    bounce: u16,
    player: &common::game::Player,
    is_winner: bool,
    ascii: bool,
    mode: common::game::models::GameMode,
) {
    let fg_color = get_color_for_id(player.id);
    let skin_cell = get_player_skin_cell(&player.skin, ascii);

    let y_emoji = top_column_y - 1 - bounce;
    let y_name = top_column_y - 2 - bounce;
    let y_detail = top_column_y - 3 - bounce;

    let detail_text = match mode {
        common::game::models::GameMode::Score => {
            format!("{} pts", player.score)
        }
        common::game::models::GameMode::Deathmatch => {
            if player.lives > 0 {
                format!("{} lives", player.lives)
            } else {
                "Defeated".to_string()
            }
        }
    };
    draw_str_centered(
        buffer,
        x_center,
        y_detail,
        &detail_text,
        Style::default().fg(Color::DarkGray),
    );

    let truncated_name = if player.name.chars().count() > 10 {
        player.name.chars().take(8).collect::<String>() + ".."
    } else {
        player.name.clone()
    };
    draw_str_centered(
        buffer,
        x_center,
        y_name,
        &truncated_name,
        Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
    );

    if is_winner {
        let sym = if ascii {
            format!("(C) {}", skin_cell.trim())
        } else {
            format!("👑 {}", skin_cell.trim())
        };
        draw_str_centered(
            buffer,
            x_center,
            y_emoji,
            &sym,
            Style::default().fg(Color::Yellow),
        );
    } else {
        draw_str_centered(
            buffer,
            x_center,
            y_emoji,
            skin_cell.trim(),
            Style::default().fg(fg_color),
        );
    }
}
