use crate::{db, tui::app::App, tui::app::InputMode};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Up => app.select_previous(),
                    KeyCode::Down => app.select_next(),
                    KeyCode::Char('s') => app.sort_by_size(),
                    KeyCode::Char('n') => app.sort_by_name(),
                    KeyCode::Char('i') => app.sort_by_install_date(),
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Tagging;
                    }
                    KeyCode::Char('u') => {
                        app.input_mode = InputMode::Untagging;
                    }
                    _ => {}
                },
                InputMode::Tagging | InputMode::Untagging => match key.code {
                    KeyCode::Enter => {
                        if let Some(selected) = app.selected_package.selected() {
                            if let Some(pkg) = app.packages.get_mut(selected) {
                                let package_name = pkg.name.clone();
                                let tag = app.input.clone();
                                let result = if matches!(app.input_mode, InputMode::Tagging) {
                                    db::add_tag(&package_name, &tag)
                                } else {
                                    db::remove_tag(&package_name, &tag)
                                };

                                match result {
                                    Ok(msg) => {
                                        app.output.push(msg);
                                        // Update the package's tags in the app state
                                        if matches!(app.input_mode, InputMode::Tagging) {
                                            if !pkg.tags.contains(&tag) {
                                                pkg.tags.push(tag);
                                                pkg.tags.sort();
                                            }
                                        } else {
                                            pkg.tags.retain(|t| t != &tag);
                                        }
                                    }
                                    Err(e) => {
                                        app.output.push(format!("Error: {}", e));
                                    }
                                }
                            }
                        }
                        app.input.clear();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input.clear();
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(false)
}