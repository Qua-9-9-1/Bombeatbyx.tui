use crate::local::app::App;
use crate::local::settings::GaugeSkin;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Widget},
};

pub fn draw_feedback(buffer: &mut Buffer, app: &App, area: Rect) {
    // 1. On récupère dynamiquement NOTRE joueur depuis l'état du jeu
    let (feedback_text, _combo) = if let Some(player) = app.game_ctx.state.players
        .iter()
        .find(|p| p.id == app.current_player_id) 
    {
        // Si le joueur a déjà fait une action sur le beat en cours, on affiche sa précision
        if player.last_acted_beat == Some(app.game_ctx.rhythm.beat_count) {
            (player.last_accuracy.as_str(), player.combo)
        } else {
            ("WAITING...", player.combo)
        }
    } else {
        ("WAITING...", 0)
    };

    // 2. Attribution de la couleur selon le texte récupéré du joueur
    let feedback_color = match feedback_text {
        "PERFECT!" => Color::Green,
        "GREAT!" => Color::Blue,
        "OKAY!" => Color::Yellow,
        "MISS!" => Color::Red,
        _ => Color::DarkGray, // Pour "WAITING..."
    };

    // 3. Rendu du paragraphe
    Paragraph::new(feedback_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(feedback_color)
                .add_modifier(Modifier::BOLD),
        )
        .render(area, buffer);
}

pub fn draw_rhythm_gauge(buffer: &mut Buffer, app: &App, area: Rect) {
    let progress = app.game_ctx.rhythm.progress();
    let width = 28_usize;

    let gauge_text = match app.settings.gauge_skin {
        GaugeSkin::NecroDancer => format_necrodancer_skin(progress, width),
        GaugeSkin::Undertale => format_undertale_skin(app, width),
        GaugeSkin::Simple => format_simple_skin(progress, width / 2),
    };

    let color = if progress > 0.85 || progress < 0.15 {
        Color::Green
    } else {
        Color::DarkGray
    };

    Paragraph::new(gauge_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .render(area, buffer);
}

fn format_necrodancer_skin(progress: f64, width: usize) -> String {
    let center = width / 2;
    let distance = ((1.0 - progress) * (center as f64)).round() as usize;

    let mut bar = vec![' '; width];
    bar[center] = 'O';

    let left_pos = center.saturating_sub(distance);
    let right_pos = (center + distance).min(width - 1);

    if distance > 0 {
        bar[left_pos] = '>';
        bar[right_pos] = '<';
    } else {
        bar[center] = 'X';
    }
    format!(" [{}] ", bar.iter().collect::<String>())
}

fn format_undertale_skin(app: &App, width: usize) -> String {
    let mut bar = vec!['-'; width];
    let target_pos = width / 2;
    bar[target_pos] = '|';

    let progress = app.game_ctx.rhythm.progress();
    let cycle_segment = (app.game_ctx.rhythm.beat_count % 4) as f64;

    let visual_position = match cycle_segment as i64 {
        0 => progress * 0.5,
        1 => 0.5 + (progress * 0.5),
        2 => 1.0 - (progress * 0.5),
        _ => 0.5 - (progress * 0.5),
    };

    let cursor_pos = (visual_position * (width - 1) as f64).round() as usize;
    if cursor_pos == target_pos {
        bar[cursor_pos] = 'X';
    } else {
        bar[cursor_pos] = '█';
    }
    format!(" [{}] ", bar.iter().collect::<String>())
}

fn format_simple_skin(progress: f64, width: usize) -> String {
    let mut bar = vec!['-'; width];

    let cursor_pos = ((progress * (width as f64)).round() as usize).clamp(0, width - 1);
    if cursor_pos == width - 1 {
        bar[cursor_pos] = 'X';
    } else {
        bar[cursor_pos] = '█';
    }
    format!(" [{}] ", bar.iter().collect::<String>())
}
