mod app;
pub mod app_states;
mod event;
mod terminal;
mod ui;

use crate::error::AppError;
use crate::tui::app_states::
    app_state::LoadedData
;
use app::App;
use terminal::{init_terminal, restore_terminal};

use tokio::sync::mpsc;

pub async fn run_tui() -> Result<(), AppError> {

    let mut app;

    loop {
        let (tx, rx) = mpsc::channel(1);
        app = App::new(rx);

        let mut terminal = init_terminal()?;

        tokio::spawn(async move {
            let packages = crate::packages::pacman::get_all_packages()
                .await
                .unwrap_or_default();
            let available_packages =
                crate::packages::pacman::get_all_available_packages().unwrap_or_default();
            let all_repos = crate::backend::get_all_repos(&available_packages);
            let orphan_package_names =
                crate::packages::pacman::get_orphan_package_names().unwrap_or_default();

            let loaded_data = LoadedData {
                packages,
                available_packages,
                all_repos,
                orphan_package_names,
            };
            // Send data to the main loop
            let _ = tx.send(loaded_data).await;
        });

        app.run(&mut terminal)?;

        restore_terminal()?;

        if let Some(command_parts) = app.command_to_run.take() {
            if command_parts.is_empty() {
                continue; 
            }

            println!("Running command: {:?}", command_parts.join(" "));
            let mut command = tokio::process::Command::new(&command_parts[0]);
            if command_parts.len() > 1 {
                command.args(&command_parts[1..]);
            }

            let status = command.status().await.map_err(|e| AppError::Io(e))?;

            if !status.success() {
                eprintln!("\nCommand failed: {:?}", command_parts);
            }

            println!("\nCommand finished. Press Enter to continue...");
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut stdin = BufReader::new(tokio::io::stdin());
            let mut buf = String::new();
            stdin.read_line(&mut buf).await?;

        } else {

            break;
        }
    }

    Ok(())
}