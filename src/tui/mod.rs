mod app;
mod event;
mod terminal;
mod ui;

use crate::error::AppError;
use app::App;
use terminal::{init_terminal, restore_terminal};

pub fn run_tui() -> Result<(), AppError> {
    let mut terminal = init_terminal()?;
    let mut app = App::new();
    app.run(&mut terminal)?;
    restore_terminal()?;
    Ok(())
}