mod tui;
mod app;
mod table;
mod test_data;
mod command;
mod cli;

use crate::cli::{get_config, Cli};
use app::App;
use clap::Parser;
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = get_config(cli);

    color_eyre::install().expect("color_eyre::install");
    let mut terminal = tui::init()?;
    let app_result = App::new(config).run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}
