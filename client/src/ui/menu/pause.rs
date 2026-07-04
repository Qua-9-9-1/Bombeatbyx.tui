use crate::local::app::{App, AppState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub fn draw_pause_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let is_host_ingame = app.network.is_multiplayer
        && app.is_local_player_host()
        && app.paused_from == Some(AppState::InGame);

    let items: Vec<&str> = if is_host_ingame {
        vec!["Continue", "Settings", "Stop Game", "Quit to Main Menu"]
    } else {
        vec!["Continue", "Settings", "Quit to Main Menu"]
    };

    let menu_w = 34;
    let menu_h = (items.len() as u16) + 4;
    let menu_x = (tui_area.width.saturating_sub(menu_w)) / 2;
    let menu_y = (tui_area.height.saturating_sub(menu_h)) / 2;
    let menu_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        " [ PAUSE MENU ] "
    } else {
        " 🛠️ PAUSE MENU 🛠️ "
    };
    let menu_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    let mut lines = vec![Line::from("")];

    let arrow_l = if ascii { "=> " } else { "► " };
    let arrow_r = if ascii { " <=" } else { " ◄" };

    let cursor = app.pause_cursor.min(items.len().saturating_sub(1));

    for (idx, item) in items.iter().enumerate() {
        let is_stop = *item == "Stop Game";
        if idx == cursor {
            let color = if is_stop { Color::Red } else { Color::Yellow };
            lines.push(Line::from(vec![
                Span::styled(
                    arrow_l,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    *item,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    arrow_r,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if is_stop {
            lines.push(Line::from(Span::styled(
                format!("   {}   ", item),
                Style::default().fg(Color::Red),
            )));
        } else {
            lines.push(Line::from(format!("   {}   ", item)));
        }
    }

    Clear.render(menu_rect, buffer);
    Paragraph::new(lines)
        .block(menu_block)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow))
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}
