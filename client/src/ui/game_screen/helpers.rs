use ratatui::style::Color;

pub fn get_combo_info(combo: u32, ascii: bool) -> String {
    if combo == 0 {
        "0".to_string()
    } else {
        let badge = if combo < 5 {
            if ascii { " +" } else { " ⚡" }
        } else if combo < 10 {
            if ascii { " *" } else { " 🔥" }
        } else if combo < 20 {
            if ascii { " !" } else { " 💥" }
        } else {
            if ascii { " K" } else { " 👑" }
        };
        format!("{}{}", combo, badge)
    }
}

pub fn get_collected_bonuses_str(bonuses: &[String], ascii: bool) -> String {
    if bonuses.is_empty() {
        if ascii {
            "None".to_string()
        } else {
            "🚫 None".to_string()
        }
    } else {
        if ascii {
            bonuses
                .iter()
                .map(|b| match b.as_str() {
                    "💣" => "B",
                    "🔥" => "R",
                    _ => "?",
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            bonuses.join(" ")
        }
    }
}

pub fn get_second_item_str(
    second_item: Option<common::game::models::SecondItem>,
    ascii: bool,
) -> String {
    match second_item {
        Some(common::game::models::SecondItem::Shield) => {
            if ascii {
                "SH".to_string()
            } else {
                "🛡️".to_string()
            }
        }
        None => "".to_string(),
    }
}

pub fn get_player_status_icon(player: &common::game::Player, ascii: bool) -> String {
    if player.is_spectator {
        if ascii {
            "EY ".to_string()
        } else {
            "👀 ".to_string()
        }
    } else if !player.is_alive {
        if player.lives == 0 {
            if ascii {
                "XX ".to_string()
            } else {
                "🪦  ".to_string()
            }
        } else {
            if ascii {
                "XX ".to_string()
            } else {
                "💀 ".to_string()
            }
        }
    } else {
        if ascii {
            let code = common::game::models::get_skin_short_code(&player.skin);
            format!("{} ", code)
        } else {
            format!("{} ", player.skin)
        }
    }
}

#[allow(dead_code)]
pub fn get_color_from_str(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        "yellow" => Color::Yellow,
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "white" => Color::White,
        _ => Color::White,
    }
}

pub fn get_emote_symbol(emote: &str, ascii: bool) -> &str {
    if ascii {
        match emote {
            "👋" => "HI",
            "✌" | "✌️" => "VI",
            "🖕" => "FU",
            "👍" => "OK",
            _ => emote,
        }
    } else {
        emote
    }
}

pub fn get_player_symbol(skin: &str, is_alive: bool, ascii: bool) -> &str {
    if !is_alive {
        return if ascii { "XX" } else { "💀" };
    }
    if ascii {
        common::game::models::get_skin_short_code(skin)
    } else {
        skin
    }
}
