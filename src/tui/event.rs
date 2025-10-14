use crate::{db, tui::app::App, tui::app::InputMode};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
                    KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                    KeyCode::Char('s') => {
                        app.input_mode = InputMode::Sorting;
                        app.sort_selection.select(Some(0));
                    }
                    KeyCode::Char('f') => {
                        app.input_mode = InputMode::Filtering;
                    }
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
                    KeyCode::Up | KeyCode::Char('k') => app.select_previous_tag(),
                    KeyCode::Down | KeyCode::Char('j') => app.select_next_tag(),
                    KeyCode::Enter => {
                        if let Some(selected_index) = app.selected_package.selected() {
                            if let Some(selected_pkg_name) = app.filtered_packages.get(selected_index).map(|p| p.name.clone()) {
                                let tag_to_apply = app.input.trim().to_string();
                                if !tag_to_apply.is_empty() {
                                    let result = if matches!(app.input_mode, InputMode::Tagging) {
                                        db::add_tag(&selected_pkg_name, &tag_to_apply)
                                    } else {
                                        db::remove_tag(&selected_pkg_name, &tag_to_apply)
                                    };
                                    match result {
                                        Ok(msg) => {
                                            app.output.push(msg);
                                            // Find the package in the main list and update its tags
                                            if let Some(pkg_to_update) = app.packages.iter_mut().find(|p| p.name == selected_pkg_name) {
                                                if matches!(app.input_mode, InputMode::Tagging) {
                                                    if !pkg_to_update.tags.contains(&tag_to_apply) {
                                                        pkg_to_update.tags.push(tag_to_apply);
                                                        pkg_to_update.tags.sort();
                                                    }
                                                } else {
                                                    pkg_to_update.tags.retain(|t| t != &tag_to_apply);
                                                }
                                            }
                                            app.reload_tags();
                                            app.apply_filters(); // Re-apply filters to update the view
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
                        app.tag_selection.select(None);
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
                        app.tag_selection.select(None);
                    }
                    _ => {}
                },
                InputMode::Sorting => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.select_previous_sort(),
                    KeyCode::Down | KeyCode::Char('j') => app.select_next_sort(),
                    KeyCode::Enter => {
                        if let Some(selected) = app.sort_selection.selected() {
                            if let Some(sort_key) = app.sort_options.get(selected) {
                                app.sort_key = *sort_key;
                                app.sort_packages();
                            }
                        }
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Filtering => {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.filter_input.insert(app.filter_cursor_position, c);
                            app.filter_cursor_position += 1;
                        }
                        KeyCode::Backspace => {
                            if app.filter_cursor_position > 0 {
                                app.filter_cursor_position -= 1;
                                app.filter_input.remove(app.filter_cursor_position);
                            }
                        }
                        KeyCode::Left => {
                            if app.filter_cursor_position > 0 {
                                app.filter_cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if app.filter_cursor_position < app.filter_input.len() {
                                app.filter_cursor_position += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let parts = app.filter_input.split_whitespace();
                            app.include_tags.clear();
                            app.exclude_tags.clear();
                            app.include_repos.clear();
                            app.exclude_repos.clear();
                            for part in parts {
                                if part.len() > 1 {
                                    let (op, filter) = part.split_at(1);
                                    if let Some((value, type_)) = filter.rsplit_once(':') {
                                        match (op, type_) {
                                            ("+", "tag") => app.include_tags.push(value.to_string()),
                                            ("-", "tag") => app.exclude_tags.push(value.to_string()),
                                            ("+", "repo") => app.include_repos.push(value.to_string()),
                                            ("-", "repo") => app.exclude_repos.push(value.to_string()),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            app.apply_filters();
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(false)
}