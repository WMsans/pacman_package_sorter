mod backend;
mod db;
mod error;
mod packages;
mod tui;

use anyhow::Result;

fn main() -> Result<()> {
    tui::run_tui()?;
    Ok(())
}