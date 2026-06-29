use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use common::game::{Cell, GameState};
use crate::game::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(frame.area());

    draw_map(frame, &app.game_state, chunks[0]);
    draw_sidebar(frame, &app.game_state, chunks[1]);
}

fn draw_map(frame: &mut Frame, state: &GameState, area: Rect) {
    let mut lines = Vec::new();

    for (y, row) in state.grid.chunks(state.width).enumerate() {
        let mut spans = Vec::new();
        for (x, cell) in row.iter().enumerate() {
            let player_here = state.players.iter().find(|p| p.x == x && p.y == y && p.is_alive);
            let span = if player_here.is_some() {
                Span::styled("P ", Style::default().fg(Color::Green))
            } else {
                match cell {
                    Cell::Empty => Span::styled("  ", Style::default().fg(Color::DarkGray)),
                    Cell::Wall => Span::styled("█ ", Style::default().fg(Color::White)),
                    Cell::Brick => Span::styled("▒ ", Style::default().fg(Color::Red)),
                    Cell::Bomb { .. } => Span::styled("Ó ", Style::default().fg(Color::Yellow)),
                    Cell::Explosion { .. } => Span::styled("* ", Style::default().fg(Color::LightRed)),
                }
            };
            spans.push(span);
        }
        lines.push(Line::from(spans));
    }

    let board = Paragraph::new(lines).block(Block::default().title(" Map ").borders(Borders::ALL));
    frame.render_widget(board, area);
}

fn draw_sidebar(frame: &mut Frame, state: &GameState, area: Rect) {
    let info_text = vec![
        Line::from(vec![Span::raw("Controls :")]),
        Line::from(vec![Span::raw("ZQSD / Arrows")]),
        Line::from(vec![Span::raw("escape to quit")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(format!("Players : {}", state.players.len()), Style::default().fg(Color::Blue))]),
    ];

    let sidebar = Paragraph::new(info_text).block(Block::default().title(" Infos ").borders(Borders::ALL));
    frame.render_widget(sidebar, area);
}