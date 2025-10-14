use crate::{db, tui::app::App, tui::app::FilterFocus, tui::app::InputMode};
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
                        app.update_filtered_filter_options();
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
                            if let Some(selected_pkg_name) =
                                app.filtered_packages.get(selected_index).map(|p| p.name.clone())
                            {
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
                                            if let Some(pkg_to_update) =
                                                app.packages.iter_mut().find(|p| p.name == selected_pkg_name)
                                            {
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
                InputMode::Filtering => match key.code {
                    KeyCode::Char(c) => {
                        app.filter_input.insert(app.filter_cursor_position, c);
                        app.filter_cursor_position += 1;
                        app.update_filtered_filter_options();
                    }
                    KeyCode::Backspace => {
                        if app.filter_cursor_position > 0 {
                            app.filter_cursor_position -= 1;
                            app.filter_input.remove(app.filter_cursor_position);
                            app.update_filtered_filter_options();
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
                    KeyCode::Up | KeyCode::Char('k') => match app.filter_focus {
                        FilterFocus::Tags => {
                            let i = match app.tag_filter_selection.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        app.filtered_tags.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            app.tag_filter_selection.select(Some(i));
                        }
                        FilterFocus::Repos => {
                            let i = match app.repo_filter_selection.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        app.filtered_repos.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            app.repo_filter_selection.select(Some(i));
                        }
                    },
                    KeyCode::Down | KeyCode::Char('j') => match app.filter_focus {
                        FilterFocus::Tags => {
                            let i = match app.tag_filter_selection.selected() {
                                Some(i) => {
                                    if i >= app.filtered_tags.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            app.tag_filter_selection.select(Some(i));
                        }
                        FilterFocus::Repos => {
                            let i = match app.repo_filter_selection.selected() {
                                Some(i) => {
                                    if i >= app.filtered_repos.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            app.repo_filter_selection.select(Some(i));
                        }
                    },
                    KeyCode::Tab => {
                        app.filter_focus = match app.filter_focus {
                            FilterFocus::Tags => FilterFocus::Repos,
                            FilterFocus::Repos => FilterFocus::Tags,
                        }
                    }
                    KeyCode::Enter => {
                        app.cycle_current_filter();
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.apply_filters();
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(false)
}