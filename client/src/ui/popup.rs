use crate::local::app::{App, ConfirmationPopup, NotificationPopup};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub fn draw_confirmation_popup(buffer: &mut Buffer, tui_area: Rect, app: &App, conf: &ConfirmationPopup) {
    let popup_w = 46;
    let popup_h = 8;
    let popup_x = tui_area.x + (tui_area.width.saturating_sub(popup_w)) / 2;
    let popup_y = tui_area.y + (tui_area.height.saturating_sub(popup_h)) / 2;
    let popup_rect = Rect::new(popup_x, popup_y, popup_w, popup_h);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        format!(" [ {} ] ", conf.title)
    } else {
        format!(" ⚠️  {} ⚠️ ", conf.title)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(&conf.message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(vec![
            Span::styled(if ascii { "[Y] Confirm" } else { "✅ [Y] Confirm" }, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("   ", Style::default()),
            Span::styled(if ascii { "[N] Cancel" } else { "❌ [N] Cancel" }, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
    ];

    Clear.render(popup_rect, buffer);
    Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow))
        .alignment(Alignment::Center)
        .render(popup_rect, buffer);
}

pub fn draw_notification_popup(buffer: &mut Buffer, tui_area: Rect, app: &App, notif: &NotificationPopup) {
    let popup_w = 46;
    let popup_h = 7;
    let popup_x = tui_area.x + (tui_area.width.saturating_sub(popup_w)) / 2;
    let popup_y = tui_area.y + (tui_area.height.saturating_sub(popup_h)) / 2;
    let popup_rect = Rect::new(popup_x, popup_y, popup_w, popup_h);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        format!(" [ {} ] ", notif.title)
    } else {
        format!(" 🔔 {} 🔔 ", notif.title)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Cyan));

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(&notif.message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("Press any key to dismiss", Style::default().fg(Color::DarkGray))),
    ];

    Clear.render(popup_rect, buffer);
    Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(Color::Black).fg(Color::Cyan))
        .alignment(Alignment::Center)
        .render(popup_rect, buffer);
}
