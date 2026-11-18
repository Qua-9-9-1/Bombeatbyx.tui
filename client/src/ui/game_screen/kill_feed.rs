//! Recent kills, shown at the top of the players sidebar.

use crate::ui::get_color_for_id;
use common::game::models::KillEvent;
use common::game::state::GameState;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Kills older than this are no longer shown.
const SHOWN_FOR_BEATS: u64 = 16;
/// Kills older than this are greyed out.
const FRESH_FOR_BEATS: u64 = 4;
const MAX_SHOWN: usize = 3;
const MAX_NAME_CHARS: usize = 8;

pub fn kill_feed_lines(state: &GameState, ascii: bool) -> Vec<Line<'static>> {
    state
        .kill_feed
        .iter()
        .rev()
        .filter(|kill| state.current_beat.saturating_sub(kill.beat) < SHOWN_FOR_BEATS)
        .take(MAX_SHOWN)
        .map(|kill| kill_line(state, kill, ascii))
        .collect()
}

fn kill_line(state: &GameState, kill: &KillEvent, ascii: bool) -> Line<'static> {
    let fresh = state.current_beat.saturating_sub(kill.beat) < FRESH_FOR_BEATS;
    let style_for = |id: u32| {
        let style = Style::default().fg(if fresh { get_color_for_id(id) } else { Color::DarkGray });
        if fresh { style.add_modifier(Modifier::BOLD) } else { style }
    };
    let victim = Span::styled(name_of(state, kill.victim), style_for(kill.victim));
    let blast = if ascii { " x " } else { " 💥 " };
    let mut spans = vec![Span::raw("  ")];
    match kill.killer {
        Some(killer) if killer == kill.victim => {
            spans.push(victim);
            spans.push(Span::styled(
                if ascii { " self-kill" } else { " 💥 self" },
                Style::default().fg(Color::DarkGray),
            ));
        }
        Some(killer) => {
            spans.push(Span::styled(name_of(state, killer), style_for(killer)));
            spans.push(Span::raw(blast));
            spans.push(victim);
        }
        None => {
            spans.push(Span::raw(if ascii { "## " } else { "🧱 " }));
            spans.push(victim);
        }
    }
    Line::from(spans)
}

fn name_of(state: &GameState, id: u32) -> String {
    let name = state
        .players
        .iter()
        .find(|p| p.id == id)
        .map_or_else(|| "?".to_string(), |p| p.name.clone());
    if name.chars().count() > MAX_NAME_CHARS {
        name.chars().take(MAX_NAME_CHARS - 1).collect::<String>() + "."
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::game::Player;

    fn text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn shows_recent_kills_newest_first_and_hides_old_ones() {
        let mut state = GameState::new(9, 9);
        state.players = vec![
            Player::new(1, "Alpha", "🤖", 3),
            Player::new(2, "Beta", "🤖", 3),
        ];
        state.current_beat = 40;
        state.kill_feed = vec![
            KillEvent { killer: Some(2), victim: 1, beat: 10 },
            KillEvent { killer: Some(1), victim: 2, beat: 38 },
            KillEvent { killer: None, victim: 1, beat: 39 },
        ];

        let lines = kill_feed_lines(&state, true);

        assert_eq!(lines.len(), 2);
        assert_eq!(text(&lines[0]), "  ## Alpha");
        assert_eq!(text(&lines[1]), "  Alpha x Beta");
    }
}
