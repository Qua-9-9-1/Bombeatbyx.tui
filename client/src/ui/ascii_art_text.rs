use ratatui::prelude::{Color, Modifier, Rect, Style};

const ART_READY: &[&str] = & [
    "██████╗ ███████╗ █████╗ ██████╗ ██╗   ██╗██╗",
    "██╔══██╗██╔════╝██╔══██╗██╔══██╗╚██╗ ██╔╝██║",
    "██████╔╝█████╗  ███████║██║  ██║ ╚████╔╝ ██║",
    "██╔══██╗██╔══╝  ██╔══██║██║  ██║  ╚██╔╝  ╚═╝",
    "██║  ██║███████╗██║  ██║██████╔╝   ██║   ██╗",
    "╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝    ╚═╝   ╚═╝",
];

const ART_READY_ASCII: &[&str] = & [
    " READY? ",
];

const ART_3: &[&str] = & [
    "██████╗ ",
    "╚════██╗",
    " █████╔╝",
    " ╚═══██╗",
    "██████╔╝",
    "╚═════╝ ",
];

const ART_3_ASCII: &[&str] = & [
    " 3 ",
];

const ART_2: &[&str] = & [
    "██████╗ ",
    "╚════██╗",
    " █████╔╝",
    "██╔═══╝ ",
    "███████╗",
    "╚══════╝",
];

const ART_2_ASCII: &[&str] = & [
    " 2 ",
];

const ART_1: &[&str] = & [
    " ██╗  ",
    "███║  ",
    "╚██║  ",
    " ██║  ",
    " ██║  ",
    " ╚═╝  ",
];

const ART_1_ASCII: &[&str] = & [
    " 1 ",
];

const ART_GO: &[&str] = & [
    " ██████╗  ██████╗ ",
    "██╔════╝ ██╔═══██╗",
    "██║  ███╗██║   ██║",
    "██║   ██║██║   ██║",
    "╚██████╔╝╚██████╔╝",
    " ╚═════╝  ╚═════╝ ",
];

const ART_GO_ASCII: &[&str] = & [
    " GO! ",
];

const ART_FINISH: &[&str] = & [
    "███████╗██╗███╗   ██╗██╗███████╗██╗  ██╗",
    "██╔════╝██║████╗  ██║██║██╔════╝██║  ██║",
    "█████╗  ██║██╔██╗ ██║██║███████╗███████║",
    "██╔══╝  ██║██║╚██╗██║██║╚════██║██╔══██║",
    "██║     ██║██║ ╚████║██║███████║██║  ██║",
    "╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝╚══════╝╚═╝  ╚═╝",
];

const ART_FINISH_ASCII: &[&str] = & [
    " FINISH! ",
];

fn draw_big_text_overlay(buffer: &mut ratatui::buffer::Buffer, area: Rect, content: &[&str], fg_color: Color) {
    let text_h = content.len() as u16;
    let text_w = content.iter().map(|line| line.chars().count()).max().unwrap_or(0) as u16;

    let bg_w = text_w + 6;
    let bg_h = text_h + 4;
    let bg_x = (area.width.saturating_sub(bg_w)) / 2;
    let bg_y = (area.height.saturating_sub(bg_h)) / 2;
    let bg_rect = Rect::new(bg_x, bg_y, bg_w, bg_h);

    draw_rectangle_background(buffer, area, bg_rect);
    draw_rectangle_borders(buffer, area, bg_rect, fg_color);
    draw_rectangle_text(buffer, area, bg_rect, content, fg_color);
}

fn draw_rectangle_background(buffer: &mut ratatui::buffer::Buffer, area: Rect, bg_rect: Rect) {
    for y in bg_rect.y..(bg_rect.y + bg_rect.height) {
        for x in bg_rect.x..(bg_rect.x + bg_rect.width) {
            if x < area.x + area.width && y < area.y + area.height {
                let cell = &mut buffer[(x, y)];
                cell.set_char(' ');
                cell.set_style(Style::default().bg(Color::Black).fg(Color::White));
            }
        }
    }
}

fn draw_rectangle_borders(buffer: &mut ratatui::buffer::Buffer, area: Rect, bg_rect: Rect, fg_color: Color) {
        let border_style = Style::default().fg(fg_color).bg(Color::Black);
        for x in bg_rect.x..(bg_rect.x + bg_rect.width) {
        if x < area.x + area.width {
            if bg_rect.y < area.y + area.height {
                buffer[(x, bg_rect.y)].set_char('═').set_style(border_style);
            }
            if (bg_rect.y + bg_rect.height - 1) < area.y + area.height {
                buffer[(x, bg_rect.y + bg_rect.height - 1)].set_char('═').set_style(border_style);
            }
        }
    }
    for y in bg_rect.y..(bg_rect.y + bg_rect.height) {
        if y < area.y + area.height {
            if bg_rect.x < area.x + area.width {
                buffer[(bg_rect.x, y)].set_char('║').set_style(border_style);
            }
            if (bg_rect.x + bg_rect.width - 1) < area.x + area.width {
                buffer[(bg_rect.x + bg_rect.width - 1, y)].set_char('║').set_style(border_style);
            }
        }
    }
    if bg_rect.x < area.x + area.width && bg_rect.y < area.y + area.height {
        buffer[(bg_rect.x, bg_rect.y)].set_char('╔').set_style(border_style);
    }
    if (bg_rect.x + bg_rect.width - 1) < area.x + area.width && bg_rect.y < area.y + area.height {
        buffer[(bg_rect.x + bg_rect.width - 1, bg_rect.y)].set_char('╗').set_style(border_style);
    }
    if bg_rect.x < area.x + area.width && (bg_rect.y + bg_rect.height - 1) < area.y + area.height {
        buffer[(bg_rect.x, bg_rect.y + bg_rect.height - 1)].set_char('╚').set_style(border_style);
    }
    if (bg_rect.x + bg_rect.width - 1) < area.x + area.width && (bg_rect.y + bg_rect.height - 1) < area.y + area.height {
        buffer[(bg_rect.x + bg_rect.width - 1, bg_rect.y + bg_rect.height - 1)].set_char('╝').set_style(border_style);
    }
}

fn draw_rectangle_text(buffer: &mut ratatui::buffer::Buffer, area: Rect, bg_rect: Rect, content: &[&str], fg_color: Color) {
    for (i, line) in content.iter().enumerate() {
        let x = bg_rect.x + 3;
        let y = bg_rect.y + 2 + (i as u16);
        let style = Style::default().fg(fg_color).bg(Color::Black).add_modifier(Modifier::BOLD);
        for (col, c) in line.chars().enumerate() {
            if (x + col as u16) < area.x + area.width && y < area.y + area.height {
                let cell = &mut buffer[(x + col as u16, y)];
                cell.set_char(c);
                cell.set_style(style);
            }
        }
    }
}

pub fn draw_countdown_overlay(buffer: &mut ratatui::buffer::Buffer, area: Rect, countdown: u32, ascii: bool) {
    if ascii {
        let (content, color) = match countdown {
            4 => (ART_READY_ASCII, Color::Cyan),
            3 => (ART_3_ASCII, Color::Yellow),
            2 => (ART_2_ASCII, Color::Yellow),
            1 => (ART_1_ASCII, Color::Yellow),
            _ => (ART_GO_ASCII, Color::Green),
        };
        draw_big_text_overlay(buffer, area, content, color);
    } else {
        let (content, color) = match countdown {
            4 => (ART_READY, Color::Cyan),
            3 => (ART_3, Color::Yellow),
            2 => (ART_2, Color::Yellow),
            1 => (ART_1, Color::Yellow),
            _ => (ART_GO, Color::Green),
        };
        draw_big_text_overlay(buffer, area, content, color);
    }
}

pub fn draw_game_over_overlay(buffer: &mut ratatui::buffer::Buffer, area: Rect, _game_over_countdown: u32, ascii: bool) {
    if ascii {
        draw_big_text_overlay(buffer, area, ART_FINISH_ASCII, Color::Red);
    } else {
        draw_big_text_overlay(buffer, area, ART_FINISH, Color::Red);
    }
}
