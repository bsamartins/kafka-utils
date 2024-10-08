mod tui;
mod app;

use app::App;
use color_eyre::eyre::{Result, WrapErr};
use ratatui::style::Stylize;
use ratatui::widgets::Widget;

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre::install");
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}
