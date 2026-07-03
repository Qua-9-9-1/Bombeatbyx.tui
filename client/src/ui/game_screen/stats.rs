use super::helpers::{get_collected_bonuses_str, get_second_item_str};
use crate::local::app::App;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_local_player_stats(
    buffer: &mut Buffer,
    app: &App,
    ctx: &common::game::GameContext,
    rect: Rect,
) {
    let ascii = app.profile.ascii_mode;
    let title = if ascii {
        " [ MY STATUS ] "
    } else {
        " ⚡ MY STATUS "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    block.render(rect, buffer);

    if let Some(player) = ctx
        .state
        .players
        .iter()
        .find(|p| p.id == app.current_player_id)
    {
        let mut line1_spans = vec![];
        match ctx.state.mode {
            common::game::models::GameMode::Deathmatch => {
                let heart = if ascii { "L:" } else { "❤️" };
                line1_spans.push(Span::styled(
                    format!(" Lives: {} {}", heart, player.lives),
                    Style::default().fg(Color::Red),
                ));
            }
            common::game::models::GameMode::Score => {
                let star = if ascii { "S:" } else { "⭐" };
                line1_spans.push(Span::styled(
                    format!(" Score: {} {}", star, player.score),
                    Style::default().fg(Color::Yellow),
                ));
            }
        }

        line1_spans.extend(vec![
            Span::styled(" | Bombs: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", player.active_bombs, player.max_bombs),
                Style::default().fg(Color::White),
            ),
            Span::styled(" | Range: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                player.bomb_range.to_string(),
                Style::default().fg(Color::White),
            ),
        ]);

        let items_str = get_collected_bonuses_str(&player.collected_bonuses, ascii);
        let second_item_raw = get_second_item_str(player.second_item, ascii);

        let line2_spans = vec![
            Span::styled(" Items: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                items_str,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" | Item 2: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                second_item_raw,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        let inner_rect = rect.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        let buffer_lines = vec![Line::from(line1_spans), Line::from(line2_spans)];

        Paragraph::new(buffer_lines).render(inner_rect, buffer);
    }
}
