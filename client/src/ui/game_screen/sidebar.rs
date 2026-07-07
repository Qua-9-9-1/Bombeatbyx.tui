use super::helpers::{
    get_collected_bonuses_str, get_combo_info, get_player_status_icon, get_second_item_str,
};
use crate::local::app::App;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_game_sidebar(
    buffer: &mut Buffer,
    app: &App,
    ctx: &common::game::GameContext,
    rect: Rect,
) {
    let ascii = app.profile.ascii_mode;
    let title = if ascii {
        " [ PLAYERS ] "
    } else {
        " 👥 PLAYERS "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    block.render(rect, buffer);

    let mut lines = vec![Line::from("")];
    let players = &ctx.state.players;
    let mode = ctx.state.mode;

    let mut has_meta = false;

    if let Some(limit) = ctx.state.time_limit_mins {
        let limit_secs = limit * 60;
        let remaining = limit_secs.saturating_sub(ctx.state.elapsed_time_secs);
        let mins = remaining / 60;
        let secs = remaining % 60;
        let timer_str = format!("Time Left: {:02}:{:02}", mins, secs);
        lines.push(Line::from(vec![Span::styled(
            format!("  {}", timer_str),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        has_meta = true;
    }

    if mode == common::game::models::GameMode::Score {
        lines.push(Line::from(vec![Span::styled(
            format!("  Target: {}", ctx.state.target_score),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        has_meta = true;
    }

    if has_meta {
        lines.push(Line::from(if ascii {
            "  ----------------------"
        } else {
            "  ──────────────────────"
        }));
    }
    lines.push(Line::from(""));

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
        }
    });

    for player in sorted_players {
        draw_sidebar_player(&mut lines, &player, mode, ascii);
    }

    let inner_rect = rect.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    Paragraph::new(lines).render(inner_rect, buffer);
}

fn draw_sidebar_player(
    lines: &mut Vec<Line>,
    player: &common::game::Player,
    mode: common::game::models::GameMode,
    ascii: bool,
) {
    let icon = get_player_status_icon(player, ascii);
    let fg_color = get_color_for_id(player.id);
    let name = if player.name.chars().count() > 14 {
        player.name.chars().take(12).collect::<String>() + ".."
    } else {
        player.name.clone()
    };

    let second_item_raw = get_second_item_str(player.second_item, ascii);
    let second_item_case = if second_item_raw.is_empty() {
        " [  ]".to_string()
    } else {
        format!(" [{}]", second_item_raw)
    };

    let spans = vec![
        Span::styled(icon, Style::default()),
        Span::styled(
            name,
            Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            second_item_case,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    lines.push(Line::from(spans));

    if player.is_alive && !player.is_spectator {
        let mut line1_spans = vec![Span::raw("  ")];
        match mode {
            common::game::models::GameMode::Deathmatch => {
                let heart = if ascii { "L:" } else { "❤️" };
                line1_spans.push(Span::styled(
                    format!("Lives: {} {}", heart, player.lives),
                    Style::default().fg(Color::Red),
                ));
            }
            common::game::models::GameMode::Score => {
                let star = if ascii { "S:" } else { "⭐" };
                line1_spans.push(Span::styled(
                    format!("Score: {} {}", star, player.score),
                    Style::default().fg(Color::Yellow),
                ));
            }
        }
        lines.push(Line::from(line1_spans));

        let combo_info = get_combo_info(player.combo, ascii);
        let line2_spans = vec![
            Span::raw("  "),
            Span::styled(
                format!("Combo: {}", combo_info),
                Style::default().fg(Color::LightRed),
            ),
        ];
        lines.push(Line::from(line2_spans));

        let items_str = get_collected_bonuses_str(&player.collected_bonuses, ascii);
        let line3_spans = vec![
            Span::raw("  "),
            Span::styled(
                format!("Items: {}", items_str),
                Style::default().fg(Color::Cyan),
            ),
        ];
        lines.push(Line::from(line3_spans));
    } else {
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
}

fn get_color_for_id(id: u32) -> Color {
    let colors = [
        Color::Green,
        Color::Magenta,
        Color::Yellow,
        Color::Blue,
        Color::Red,
        Color::Cyan,
        Color::White,
    ];
    colors[(id as usize) % colors.len()]
}
