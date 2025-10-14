use crate::{db, tui::app::{App, FilterFocus, InputMode}};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Up | KeyCode::Char('k') => app.select_previous_package(),
                    KeyCode::Down | KeyCode::Char('j') => app.select_next_package(),
                    KeyCode::Char('s') => {
                        app.input_mode = InputMode::Sorting;
                        app.sort_state.selection.select(Some(0));
                    }
                    KeyCode::Char('f') => {
                        app.input_mode = InputMode::Filtering;
                        app.filter_state.update_filtered_options(&app.state.all_tags, &app.state.all_repos);
                    }
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Tagging;
                        app.tag_state.update_filtered_tags(&app.state.all_tags);
                        app.tag_state.selection.select(Some(0));
                        if let Some(tag) = app.tag_state.filtered_tags.get(0) {
                            app.tag_state.input = tag.clone();
                        }
                    }
                    KeyCode::Char('d') => {
                        app.input_mode = InputMode::Untagging;
                        app.tag_state.update_filtered_tags(&app.state.all_tags);
                        app.tag_state.selection.select(Some(0));
                        if let Some(tag) = app.tag_state.filtered_tags.get(0) {
                            app.tag_state.input = tag.clone();
                        }
                    }
                    _ => {}
                },
                InputMode::Tagging | InputMode::Untagging => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.tag_state.select_previous_tag(),
                    KeyCode::Down | KeyCode::Char('j') => app.tag_state.select_next_tag(),
                    KeyCode::Enter => {
                        if let Some(selected_index) = app.selected_package.selected() {
                            if let Some(selected_pkg_name) =
                                app.state.filtered_packages.get(selected_index).map(|p| p.name.clone())
                            {
                                let tag_to_apply = app.tag_state.input.trim().to_string();
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
                                                app.state.packages.iter_mut().find(|p| p.name == selected_pkg_name)
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
                        app.tag_state.input.clear();
                        app.tag_state.update_filtered_tags(&app.state.all_tags);
                        app.input_mode = InputMode::Normal;
                        app.tag_state.selection.select(None);
                    }
                    KeyCode::Char(c) => {
                        app.tag_state.input.push(c);
                        app.tag_state.update_filtered_tags(&app.state.all_tags);
                    }
                    KeyCode::Backspace => {
                        app.tag_state.input.pop();
                        app.tag_state.update_filtered_tags(&app.state.all_tags);
                    }
                    KeyCode::Esc => {
                        app.tag_state.input.clear();
                        app.tag_state.update_filtered_tags(&app.state.all_tags);
                        app.input_mode = InputMode::Normal;
                        app.tag_state.selection.select(None);
                    }
                    _ => {}
                },
                InputMode::Sorting => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.sort_state.select_previous(),
                    KeyCode::Down | KeyCode::Char('j') => app.sort_state.select_next(),
                    KeyCode::Enter => {
                        if let Some(selected) = app.sort_state.selection.selected() {
                            if let Some(sort_key) = app.sort_state.options.get(selected) {
                                app.sort_state.active_sort_key = *sort_key;
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
                    match app.filter_state.focus {
                        FilterFocus::Search => match key.code {
                            KeyCode::Char(c) => {
                                app.filter_state.input.insert(app.filter_state.cursor_position, c);
                                app.filter_state.cursor_position += 1;
                                app.filter_state.update_filtered_options(&app.state.all_tags, &app.state.all_repos);
                            }
                            KeyCode::Backspace => {
                                if app.filter_state.cursor_position > 0 {
                                    app.filter_state.cursor_position -= 1;
                                    app.filter_state.input.remove(app.filter_state.cursor_position);
                                    app.filter_state.update_filtered_options(&app.state.all_tags, &app.state.all_repos);
                                }
                            }
                            KeyCode::Left => {
                                if app.filter_state.cursor_position > 0 {
                                    app.filter_state.cursor_position -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if app.filter_state.cursor_position < app.filter_state.input.len() {
                                    app.filter_state.cursor_position += 1;
                                }
                            }
                            KeyCode::Tab => {
                                app.filter_state.focus = FilterFocus::Tags;
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.apply_filters();
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                        FilterFocus::Tags | FilterFocus::Repos => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                if let FilterFocus::Tags = app.filter_state.focus {
                                    let i = match app.filter_state.tag_selection.selected() {
                                        Some(i) => if i >= app.filter_state.filtered_tags.len() - 1 { 0 } else { i + 1 },
                                        None => 0,
                                    };
                                    app.filter_state.tag_selection.select(Some(i));
                                } else {
                                    let i = match app.filter_state.repo_selection.selected() {
                                        Some(i) => if i >= app.filter_state.filtered_repos.len() - 1 { 0 } else { i + 1 },
                                        None => 0,
                                    };
                                    app.filter_state.repo_selection.select(Some(i));
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if let FilterFocus::Tags = app.filter_state.focus {
                                    let i = match app.filter_state.tag_selection.selected() {
                                        Some(i) => if i == 0 { app.filter_state.filtered_tags.len() - 1 } else { i - 1 },
                                        None => 0,
                                    };
                                    app.filter_state.tag_selection.select(Some(i));
                                } else {
                                    let i = match app.filter_state.repo_selection.selected() {
                                        Some(i) => if i == 0 { app.filter_state.filtered_repos.len() - 1 } else { i - 1 },
                                        None => 0,
                                    };
                                    app.filter_state.repo_selection.select(Some(i));
                                }
                            }
                            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                                app.filter_state.cycle_filter_state(true);
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                app.filter_state.cycle_filter_state(false);
                            }
                            KeyCode::Tab => {
                                app.filter_state.focus = match app.filter_state.focus {
                                    FilterFocus::Tags => FilterFocus::Repos,
                                    FilterFocus::Repos => FilterFocus::Search,
                                    _ => FilterFocus::Search,
                                }
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
        }
    }
    Ok(false)
}