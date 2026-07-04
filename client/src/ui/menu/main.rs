use crate::local::app::App;
use crate::ui::menu::center_rect;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_main_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 50, 16);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        " [ MAIN MENU ] "
    } else {
        " 👾 MAIN MENU 👾 "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let items = if ascii {
        vec![
            "Host Game",
            "Join Game",
            "Settings",
            "Quit",
        ]
    } else {
        vec![
            "🚀 Host Game",
            "🌐 Join Game",
            "⚙️  Settings",
            "❌ Quit",
        ]
    };

    let subtitle = if ascii {
        "=== Bombeatbyx ==="
    } else {
        "💣 Bombeatbyx 💣"
    };
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            subtitle,
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let arrow_l = if ascii { "  => " } else { "  ► " };
    let arrow_r = if ascii { " <=  " } else { " ◄  " };

    for (idx, item) in items.iter().enumerate() {
        if idx == app.main_menu_screen.cursor {
            lines.push(Line::from(vec![
                Span::styled(
                    arrow_l,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    *item,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    arrow_r,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(format!("    {}    ", item)));
        }
    }

    lines.push(Line::from(""));
    let instruct = if ascii {
        "Use Z/S (Up/Down) to navigate, Enter to select"
    } else {
        "Use Z/S or Up/Down to navigate, Enter to select"
    };
    lines.push(Line::from(Span::styled(
        instruct,
        Style::default().fg(Color::DarkGray),
    )));

    if let Some(ref err) = app.network.network_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Error: {}", err),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}
