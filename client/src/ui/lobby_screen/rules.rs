use super::info::get_skin_label;
use crate::local::app::App;
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

    let items = [
        if is_host {
            format!("Map Width     : < {} >", rs.width)
        } else {
            format!("Map Width     : {}", rs.width)
        },
        if is_host {
            format!("Map Height    : < {} >", rs.height)
        } else {
            format!("Map Height    : {}", rs.height)
        },
        if is_host {
            format!("BPM (Tempo)   : < {:.0} >", rs.bpm)
        } else {
            format!("BPM (Tempo)   : {:.0}", rs.bpm)
        },
        if is_host {
            format!(
                "Sudden Death  : < {} >",
                if rs.sudden_death { "ON" } else { "OFF" }
            )
        } else {
            format!(
                "Sudden Death  : {}",
                if rs.sudden_death { "ON" } else { "OFF" }
            )
        },
        if is_host {
            format!("Bonus Spawn   : < Every {} beats >", rs.bonus_every)
        } else {
            format!("Bonus Spawn   : Every {} beats", rs.bonus_every)
        },
        if is_host {
            format!("Player Lives  : < {} >", rs.lives)
        } else {
            format!("Player Lives  : {}", rs.lives)
        },
        if is_host {
            format!("Game Mode     : < {:?} >", rs.mode)
        } else {
            format!("Game Mode     : {:?}", rs.mode)
        },
        format!("Your Name     : < {} >", name_display),
        format!("Your Skin     : < {} >", skin_name),
        ready_btn_text,
    ];

    let mut center_lines = vec![Line::from("")];
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
    center_lines.push(Line::from(""));

    let arrow_l = if ascii { " => " } else { " ► " };
    let arrow_r = if ascii { " <= " } else { " ◄" };

    for (idx, item) in items.iter().enumerate() {
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
            center_lines.push(Line::from(""));
        }

        if idx == 7 {
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
            center_lines.push(Line::from(""));
        }

        if idx == cursor {
            if idx == 9 {
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
                let is_editing = idx == 7 && app.editing_name;
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
            if idx == 9 {
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

    let help_desc = if cursor == 7 {
        if app.editing_name {
            "Type name, Backspace to delete, Enter to save"
        } else {
            "Press Enter to edit name"
        }
    } else if cursor == 8 {
        "Press Q/D or Left/Right to change skin"
    } else if cursor == 9 {
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
