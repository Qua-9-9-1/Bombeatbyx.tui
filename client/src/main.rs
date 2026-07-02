mod local;
mod screens;
mod tui;
mod ui;

use crate::local::app::App;
use crate::tui::Tui;
use std::io;

fn main() -> io::Result<()> {
    let mut tui = Tui::new()?;
    tui.init()?;

    let mut app = App::new();
    app.run(&mut tui)?;

    Ok(())
}
