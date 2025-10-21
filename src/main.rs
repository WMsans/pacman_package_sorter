mod backend;
mod config; 
mod db;
mod error;
mod packages;
mod tui;

use anyhow::Result;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    tui::run_tui().await?;
    Ok(())
}