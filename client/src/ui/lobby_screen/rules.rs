use super::info::get_skin_label;
use crate::local::app::App;
use crate::screens::lobby::{ROW_NAME, ROW_READY, ROW_SKIN, is_bot_row};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub fn draw_rules_panel(buffer: &mut Buffer, area: Rect, app: &App) {
    let ascii = app.profile.ascii_mode;
    let title = if let Some(ref code) = app.network.room_code {
        if ascii {
            format!(" [ ROOM CONFIG | CODE: {} ] ", code)
        } else {
            format!(" 🎮 ROOM CONFIG | 🔑 CODE: {} ", code)
        }
    } else {
        if ascii {
            " [ ROOM CONFIG ] ".to_string()
        } else {
            " 🎮 ROOM CONFIG ".to_string()
        }
    };
    let center_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let rs = &app.room_settings;
    let is_host = app.is_local_player_host();
    let cursor = app.lobby_screen.cursor;

    let skin_name = get_skin_label(&app.profile.skin, ascii);

    let name_display = if app.editing_name {
        format!("{}█", app.profile.name)
    } else {
        app.profile.name.clone()
    };

    let my_player_ready = if let Some(ref ctx) = app.game_ctx {
        ctx.state
            .players
            .iter()
            .find(|p| p.id == app.current_player_id)
            .map(|p| p.is_ready)
            .unwrap_or(false)
    } else {
        false
    };

    let skin_taken = if let Some(ref ctx) = app.game_ctx {
        ctx.state
            .players
            .iter()
            .any(|p| p.id != app.current_player_id && p.is_ready && p.skin == app.profile.skin)
    } else {
        false
    };

    let ready_btn_text = if skin_taken {
        " [ SKIN ALREADY TAKEN! ] ".to_string()
    } else if my_player_ready {
        " [ READY ] ".to_string()
    } else {
        " [ READY ] ".to_string()
    };

    // Host-only settings show `< value >` arrows to the host only.
    let setting = |label: &str, value: String| {
        if is_host {
            format!("{:<14}: < {} >", label, value)
        } else {
            format!("{:<14}: {}", label, value)
        }
    };
    let on_off = |on: bool| if on { "ON" } else { "OFF" }.to_string();
    let items = [
        setting("Map Width", rs.width.to_string()),
        setting("Map Height", rs.height.to_string()),
        setting("BPM (Tempo)", format!("{:.0}", rs.bpm)),
        setting("Sudden Death", on_off(rs.sudden_death)),
        setting("Bonus Spawn", format!("Every {} beats", rs.bonus_every)),
        setting("Player Lives", rs.lives.to_string()),
        setting("Game Mode", format!("{:?}", rs.mode)),
        setting("Target Score", rs.target_score.to_string()),
        setting(
            "Time Limit",
            rs.time_limit_mins
                .map_or_else(|| "OFF".to_string(), |m| format!("{} mins", m)),
        ),
        setting("Map Pattern", rs.map_layout.label().to_string()),
        setting("Brick Hints", on_off(rs.brick_hints)),
        format!("Bots          : < {} >", rs.bot_count),
        format!("Bot Level     : < {} >", rs.bot_difficulty.label()),
        format!("Your Name     : < {} >", name_display),
        format!("Your Skin     : < {} >", skin_name),
        ready_btn_text,
    ];

    let mut center_lines = vec![];
    if is_host {
        let label = if ascii {
            "  == HOST SETTINGS (Q/D to adjust) =="
        } else {
            "  ★ HOST SETTINGS (Q/D to adjust) ★"
        };
        center_lines.push(Line::from(Span::styled(
            label,
            Style::default().fg(Color::Yellow),
        )));
    } else {
        let label = if ascii {
            "  ... WAITING FOR HOST ..."
        } else {
            "  ⏱️ WAITING FOR HOST..."
        };
        center_lines.push(Line::from(Span::styled(
            label,
            Style::default().fg(Color::DarkGray),
        )));
    }

    let arrow_l = if ascii { " => " } else { " ► " };
    let arrow_r = if ascii { " <= " } else { " ◄" };

    for (idx, item) in items.iter().enumerate() {
        if is_bot_row(idx) && !app.is_solo_mode {
            continue;
        }
        if idx == 0 {
            let label = if ascii {
                "  -- GAME CONFIGURATION --"
            } else {
                "  ⚙️  GAME CONFIGURATION ⚙️"
            };
            center_lines.push(Line::from(Span::styled(
                label,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        if idx == ROW_NAME {
            let label = if ascii {
                "  -- PLAYER PERSONALIZATION --"
            } else {
                "  👤 PLAYER PERSONALIZATION 👤"
            };
            center_lines.push(Line::from(Span::styled(
                label,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        if idx == cursor {
            if idx == ROW_READY {
                let color = if skin_taken {
                    Color::Red
                } else if my_player_ready {
                    Color::Green
                } else {
                    Color::Yellow
                };
                let text = if ascii {
                    format!(" =>> {} <<=", item)
                } else {
                    format!(" ► 🔥 {} 🔥 ◄", item)
                };
                center_lines.push(Line::from(Span::styled(
                    text,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )));
            } else {
                let is_editing = idx == ROW_NAME && app.editing_name;
                let color = if is_editing {
                    Color::LightGreen
                } else {
                    Color::Cyan
                };
                center_lines.push(Line::from(vec![
                    Span::styled(
                        arrow_l,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        item.as_str(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        arrow_r,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        } else {
            if idx == ROW_READY {
                let color = if skin_taken {
                    Color::Red
                } else if my_player_ready {
                    Color::Green
                } else {
                    Color::DarkGray
                };
                center_lines.push(Line::from(Span::styled(
                    format!("     {}", item),
                    Style::default().fg(color),
                )));
            } else {
                center_lines.push(Line::from(format!("   {}   ", item)));
            }
        }
    }

    center_lines.push(Line::from(""));

    let help_desc = if cursor == ROW_NAME {
        if app.editing_name {
            "Type name, Backspace to delete, Enter to save"
        } else {
            "Press Enter to edit name"
        }
    } else if cursor == ROW_SKIN {
        "Press Q/D or Left/Right to change skin"
    } else if cursor == ROW_READY {
        if skin_taken {
            "Change your skin! Someone else is ready with it"
        } else if my_player_ready {
            "Press Enter to toggle NOT READY"
        } else {
            "Press Enter to toggle READY"
        }
    } else {
        "Press Q/D or Left/Right to adjust values"
    };

    center_lines.push(Line::from(Span::styled(
        format!("  {}", help_desc),
        Style::default().fg(Color::DarkGray),
    )));

    Paragraph::new(center_lines)
        .block(center_block)
        .render(area, buffer);
}
