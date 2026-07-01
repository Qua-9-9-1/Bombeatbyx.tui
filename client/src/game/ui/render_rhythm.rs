use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style, Modifier},
    widgets::{Paragraph, Widget},
};
use crate::game::app::App;
use crate::game::rhythm::GaugeSkin;

pub fn draw_feedback(buffer: &mut Buffer, app: &App, area: Rect) {
    let feedback_color = match app.last_feedback {
        "PERFECT!" => Color::Green,
        "GREAT!" => Color::Blue,
        "OKAY!" => Color::Yellow,
        "MISS!" => Color::Red,
        _ => Color::DarkGray,
    };

    Paragraph::new(app.last_feedback)
        .alignment(Alignment::Center)
        .style(Style::default().fg(feedback_color).add_modifier(Modifier::BOLD))
        .render(area, buffer);
}

pub fn draw_rhythm_gauge(buffer: &mut Buffer, app: &App, area: Rect) {
    let progress = app.rhythm.progress();
    let width = 28_usize; 
    
    let gauge_text = match app.rhythm.skin {
        GaugeSkin::NecroDancer => format_necrodancer_skin(progress, width),
        GaugeSkin::Undertale => format_undertale_skin(progress, width),
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

fn format_undertale_skin(progress: f64, width: usize) -> String {
    let mut bar = vec!['-'; width];
    let target_pos = width / 2; 
    bar[target_pos] = '|'; 
    
    let cursor_pos = ((progress * (width as f64)).round() as usize).clamp(0, width - 1);
    if cursor_pos == target_pos {
        bar[cursor_pos] = 'X';
    } else {
        bar[cursor_pos] = '█';
    }
    format!(" [{}] ", bar.iter().collect::<String>())
}