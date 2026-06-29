mod tui;
mod game;

use std::io;
use crate::tui::Tui;
use crate::game::app::App;

fn main() -> io::Result<()> {
    let mut tui = Tui::new()?;
    tui.init()?;

    let mut app = App::new();
    app.run(&mut tui)?;

    Ok(())
}