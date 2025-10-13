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
                        app.tag_selection.select(Some(0));
                    }
                    KeyCode::Char('u') => {
                        app.input_mode = InputMode::Untagging;
                        app.tag_selection.select(Some(0));
                    }
                    _ => {}
                },
                InputMode::Tagging | InputMode::Untagging => match key.code {
                    KeyCode::Up => app.select_previous_tag(),
                    KeyCode::Down => app.select_next_tag(),
                    KeyCode::Enter => {
                        if let Some(selected) = app.selected_package.selected() {
                            if let Some(pkg) = app.packages.get_mut(selected) {
                                let package_name = pkg.name.clone();

                                // If a tag is selected in the list, use it. Otherwise, use the input field.
                                let tag_to_apply = if !app.input.is_empty() {
                                    app.input.clone()
                                } else if let Some(selected_tag_index) = app.tag_selection.selected() {
                                    app.all_tags.get(selected_tag_index).cloned().unwrap_or_default()
                                } else {
                                    String::new()
                                };


                                if !tag_to_apply.is_empty() {
                                    let result = if matches!(app.input_mode, InputMode::Tagging) {
                                        db::add_tag(&package_name, &tag_to_apply)
                                    } else {
                                        db::remove_tag(&package_name, &tag_to_apply)
                                    };

                                    match result {
                                        Ok(msg) => {
                                            app.output.push(msg);
                                            // Update the package's tags in the app state
                                            if matches!(app.input_mode, InputMode::Tagging) {
                                                if !pkg.tags.contains(&tag_to_apply) {
                                                    pkg.tags.push(tag_to_apply);
                                                    pkg.tags.sort();
                                                }
                                            } else {
                                                pkg.tags.retain(|t| t != &tag_to_apply);
                                            }
                                            app.reload_tags(); // Reload tags after modification
                                        }
                                        Err(e) => {
                                            app.output.push(format!("Error: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                        app.input.clear();
                        app.input_mode = InputMode::Normal;
                        app.tag_selection.select(None); // Deselect tag
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
                        app.tag_selection.select(None); // Deselect tag
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(false)
}