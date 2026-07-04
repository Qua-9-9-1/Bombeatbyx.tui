mod local;
mod network;
mod screens;
mod tui;
mod ui;

use crate::local::app::App;
use crate::tui::Tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tui = Tui::new()?;
    tui.init()?;

    let mut app = App::new();
    app.run(&mut tui).await?;

    Ok(())
}
