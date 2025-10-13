mod backend;
mod db;
mod error;
mod models;
mod tui;

use anyhow::Result;

fn main() -> Result<()> {
    tui::run_tui()?;
    Ok(())
}