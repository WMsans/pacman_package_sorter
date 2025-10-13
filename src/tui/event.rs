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
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('u') => {
                        if let Some(selected) = app.selected_package.selected() {
                            if let Some(pkg) = app.packages.get(selected) {
                                if let Some(tag) = pkg.tags.first() {
                                    db::remove_tag(&pkg.name, tag).unwrap();
                                }
                            }
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if let Some(selected) = app.selected_package.selected() {
                            if let Some(pkg) = app.packages.get(selected) {
                                db::add_tag(&pkg.name, &app.input).unwrap();
                                app.input.clear();
                                app.input_mode = InputMode::Normal;
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(false)
}