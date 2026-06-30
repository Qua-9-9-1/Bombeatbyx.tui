use ratatui::prelude::*;
use std::io::{self, Stdout};
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        Ok(Self { terminal })
    }

    pub fn init(&mut self) -> io::Result<bool> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;

        execute!(stdout, EnterAlternateScreen)?;

        let enhanced_active = execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        ).is_ok();

        let panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = disable_raw_mode();
            panic_hook(panic_info);
        }));

        self.terminal.clear()?;
        Ok(enhanced_active)
    }

    pub fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}