use crate::local::app::App;
use crate::local::settings::GaugeSkin;
use crate::ui::menu::center_rect;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

fn key_display(k: char) -> String {
    if k == ' ' {
        "SPACE".to_string()
    } else {
        k.to_uppercase().to_string()
    }
}

pub fn draw_settings_menu(buffer: &mut Buffer, tui_area: Rect, app: &App) {
    let menu_rect = center_rect(tui_area, 58, 22);
    let ascii = app.profile.ascii_mode;

    let title = if ascii {
        " [ SETTINGS ] "
    } else {
        " ⚙️ SETTINGS ⚙️ "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let preset_str = app.profile.key_preset.label();
    let gauge_str = match app.profile.gauge_skin {
        GaugeSkin::NecroDancer => "Crypt of the NecroDancer",
        GaugeSkin::Undertale => "Undertale",
        GaugeSkin::Simple => "Simple",
    };
    let mode_str = if ascii { "ASCII" } else { "Emojis" };
    let back_label = if app.paused_from.is_some() {
        "Back to Pause"
    } else {
        "Back to Main Menu"
    };

    let capturing = app.capturing_key;
    let cursor = app.settings_cursor;

    let key_label = |slot: usize, key: char, label: &str| -> String {
        if capturing && cursor == slot {
            format!("{:<12}: [ Press any key... ]", label)
        } else {
            format!("{:<12}: [ {} ]", label, key_display(key))
        }
    };

    let items: Vec<String> = vec![
        format!("Key Preset  : < {} >", preset_str),
        key_label(1, app.profile.key_up,    "Move Up"),
        key_label(2, app.profile.key_down,  "Move Down"),
        key_label(3, app.profile.key_left,  "Move Left"),
        key_label(4, app.profile.key_right, "Move Right"),
        key_label(5, app.profile.key_bomb,  "Place Bomb"),
        key_label(6, app.profile.key_spell, "Spell / Item"),
        format!("Gauge Skin  : < {} >", gauge_str),
        format!("Display Mode: < {} >", mode_str),
        back_label.to_string(),
    ];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "SETTINGS CONFIGURATION",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let arrow_l = if ascii { "  => " } else { "  ► " };
    let arrow_r = if ascii { " <=  " } else { " ◄  " };

    for (idx, item) in items.iter().enumerate() {
        let base_color = match idx {
            1..=6 => Color::Cyan,
            9 => Color::White,
            _ => Color::Yellow,
        };
        let is_capturing_slot = capturing && idx == cursor;
        let color = if is_capturing_slot { Color::LightRed } else { base_color };

        if idx == cursor {
            lines.push(Line::from(vec![
                Span::styled(arrow_l, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(item.as_str(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(arrow_r, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                format!("    {}    ", item),
                Style::default().fg(if (1..=6).contains(&idx) { Color::Cyan } else { Color::Reset }),
            )));
        }
    }

    lines.push(Line::from(""));
    if capturing {
        lines.push(Line::from(Span::styled(
            "Press any character key to bind  |  Esc to cancel",
            Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Z/S navigate  Q/D adjust  Enter=bind key  Emotes: F1-F4",
            Style::default().fg(Color::DarkGray),
        )));
    }

    Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .render(menu_rect, buffer);
}
