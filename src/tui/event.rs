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
                    KeyCode::Char('u') => app.sort_by_update_date(),
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Tagging;
                        app.update_filtered_tags();
                        app.tag_selection.select(Some(0));
                        if let Some(tag) = app.filtered_tags.get(0) {
                            app.input = tag.clone();
                        }
                    }
                    KeyCode::Char('d') => {
                        app.input_mode = InputMode::Untagging;
                        app.update_filtered_tags();
                        app.tag_selection.select(Some(0));
                        if let Some(tag) = app.filtered_tags.get(0) {
                            app.input = tag.clone();
                        }
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

                                let tag_to_apply = app.input.trim().to_string();


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
                        app.update_filtered_tags();
                        app.input_mode = InputMode::Normal;
                        app.tag_selection.select(None); // Deselect tag
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.update_filtered_tags();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.update_filtered_tags();
                    }
                    KeyCode::Esc => {
                        app.input.clear();
                        app.update_filtered_tags();
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