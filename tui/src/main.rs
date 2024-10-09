mod tui;
mod app;
mod table;
mod test_data;

use app::App;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre::install");
    let mut terminal = tui::init()?;
    let app_result = App::new().run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}
