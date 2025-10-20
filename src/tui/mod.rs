mod app;
pub mod app_states;
mod event;
mod terminal;
mod ui;

use crate::error::AppError;
use crate::tui::app_states::{filter_modal_state::FilterModalState, tag_modal_state::TagModalState}; 
use app::App;
use std::process::Command; 
use terminal::{init_terminal, restore_terminal};

pub fn run_tui() -> Result<(), AppError> {

    let mut app = App::new();

    loop {

        let mut terminal = init_terminal()?;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            app.state.load_packages().await;
        });

        app.filter_state = FilterModalState::new(&app.state.all_tags, &app.state.all_repos);
        app.tag_state = TagModalState::new(&app.state.all_tags);

        app.apply_filters();

        if !app.state.filtered_packages.is_empty() {
            app.selected_package.select(Some(0));
        }

        app.run(&mut terminal)?;

        restore_terminal()?;

        if let Some(command_parts) = app.command_to_run.take() {
            if command_parts.is_empty() {
                continue; 
            }

            println!("Running command: {:?}", command_parts.join(" "));
            let mut command = Command::new(&command_parts[0]);
            if command_parts.len() > 1 {
                command.args(&command_parts[1..]);
            }

            let status = command
                .status()
                .map_err(|e| AppError::Io(e))?;

            if !status.success() {
                eprintln!("\nCommand failed: {:?}", command_parts);
            }

            println!("\nCommand finished. Press Enter to continue...");
            std::io::stdin().read_line(&mut String::new())?;

        } else {

            break;
        }
    }

    Ok(())
}