use crate::local::app::{App, ConfirmationPopup, NotificationPopup};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap},
};

pub fn draw_confirmation_popup(
    buffer: &mut Buffer,
    tui_area: Rect,
    app: &App,
    conf: &ConfirmationPopup,
) {
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
        Line::from(Span::styled(
            &conf.message,
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if ascii {
                    "[Y] Confirm"
                } else {
                    "✅ [Y] Confirm"
                },
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   ", Style::default()),
            Span::styled(
                if ascii {
                    "[N] Cancel"
                } else {
                    "❌ [N] Cancel"
                },
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    Clear.render(popup_rect, buffer);
    Paragraph::new(lines)
        .block(block)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow))
        .alignment(Alignment::Center)
        .render(popup_rect, buffer);
}

pub fn draw_notification_popup(
    buffer: &mut Buffer,
    tui_area: Rect,
    app: &App,
    notif: &NotificationPopup,
) {
    let popup_w = NOTIFICATION_WIDTH.min(tui_area.width);
    let text_w = popup_w.saturating_sub(4).max(1) as usize;
    let message_lines = wrapped_line_count(&notif.message, text_w) as u16;
    // Borders (2) + blank line + message + blank line + dismiss hint.
    let popup_h = (message_lines + 5).min(tui_area.height);
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
        Line::from(Span::styled(
            &notif.message,
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to dismiss",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    Clear.render(popup_rect, buffer);
    Paragraph::new(lines)
        .block(block.padding(Padding::horizontal(1)))
        .style(Style::default().bg(Color::Black).fg(Color::Cyan))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .render(popup_rect, buffer);
}

const NOTIFICATION_WIDTH: u16 = 64;

/// Lines needed to word-wrap `text` at `width` columns (same rule as `Wrap`).
pub fn wrapped_line_count(text: &str, width: usize) -> usize {
    let mut lines = 1;
    let mut current = 0;
    for word in text.split_whitespace() {
        let len = word.chars().count();
        let needed = if current == 0 { len } else { current + 1 + len };
        if needed <= width {
            current = needed;
        } else {
            lines += 1 + (len.saturating_sub(1)) / width;
            current = len % width;
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_line_count_breaks_on_words() {
        // "No server is" | "running" | "here, really"
        let text = "No server is running here, really";

        let count = wrapped_line_count(text, 12);

        assert_eq!(count, 3);
    }

    #[test]
    fn wrapped_line_count_single_line_when_it_fits() {
        assert_eq!(wrapped_line_count("short", 20), 1);
    }
}
