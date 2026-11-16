use super::info_settings::setting_lines;
use crate::screens::lobby::{ROW_BOT_COUNT, ROW_BOT_LEVEL, ROW_NAME, ROW_READY, ROW_SKIN};
use common::game::models::{MAX_BOTS, RoomSettings};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_info_panel(
    buffer: &mut Buffer,
    area: Rect,
    cursor: usize,
    rs: &RoomSettings,
    ascii: bool,
) {
    let title = if ascii {
        " [ INFO PANEL ] "
    } else {
        " ℹ️ INFO PANEL "
    };
    let info_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let mut info_lines = vec![blank()];
    info_lines.extend(if cursor < ROW_BOT_COUNT {
        setting_lines(cursor, rs, ascii)
    } else {
        player_lines(cursor, ascii)
    });

    Paragraph::new(info_lines)
        .block(info_block)
        .render(area, buffer);
}

fn player_lines(cursor: usize, ascii: bool) -> Vec<Line<'static>> {
    match cursor {
        ROW_BOT_COUNT => vec![
            header("[ BOTS ]", "🤖 BOTS", ascii),
            blank(),
            text("Number of AI opponents"),
            text(format!("in solo mode (1 to {}).", MAX_BOTS)),
            blank(),
            text("Bots play on the beat too:"),
            text("they break bricks, grab"),
            text("bonuses and hunt you down."),
        ],
        ROW_BOT_LEVEL => vec![
            header("[ BOT LEVEL ]", "🧠 BOT LEVEL", ascii),
            blank(),
            text("Easy: slow, clumsy, only"),
            text("breaks bricks."),
            text("Normal: hunts players and"),
            text("uses items."),
            text("Hard: near-perfect rhythm,"),
            text("aggressive and careful."),
        ],
        ROW_NAME => vec![
            header("[ PLAYER NAME ]", "📝 PLAYER NAME", ascii),
            blank(),
            text("Set your nickname."),
            blank(),
            title("Options:"),
            text("A-Z, 0-9, A-Z, underscores"),
            text("Max 12 characters."),
        ],
        ROW_SKIN => vec![
            header("[ CHARACTER SKIN ]", "🎭 CHARACTER SKIN", ascii),
            blank(),
            text(if ascii {
                "Select your character skin."
            } else {
                "Select your emoji skin."
            }),
            text("This is how other players"),
            text("will see you on the map."),
            blank(),
            title("Options:"),
            text(format!(
                "{} skins, Q/D to browse.",
                common::game::models::ALL_SKINS.len()
            )),
            text("A skin can't be shared by"),
            text("two ready players."),
        ],
        ROW_READY => vec![
            Line::from(Span::styled(
                if ascii { " [ START MATCH ] " } else { "🚀 START MATCH" },
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )),
            blank(),
            text("Launches the local match."),
            blank(),
            text("Press Enter to join the"),
            text("arena now!"),
        ],
        _ => Vec::new(),
    }
}

pub fn header(ascii_label: &str, emoji_label: &str, ascii: bool) -> Line<'static> {
    let label = if ascii {
        format!(" {} ", ascii_label)
    } else {
        emoji_label.to_string()
    };
    Line::from(Span::styled(
        label,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan),
    ))
}

pub fn text(content: impl Into<String>) -> Line<'static> {
    Line::from(content.into())
}

pub fn title(content: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(
        content.into(),
        Style::default().add_modifier(Modifier::UNDERLINED),
    ))
}

pub fn dim(content: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(
        content.into(),
        Style::default().fg(Color::DarkGray),
    ))
}

pub fn blank() -> Line<'static> {
    Line::from("")
}

pub fn get_skin_label(skin: &str, ascii: bool) -> String {
    if ascii {
        let name = common::game::models::get_skin_name(skin);
        let code = common::game::models::get_skin_short_code(skin);
        if name == "Player" {
            format!("[{}]", skin)
        } else {
            format!("{} [{}]", name, code)
        }
    } else {
        skin.to_string()
    }
}
