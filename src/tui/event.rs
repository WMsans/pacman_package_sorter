use crate::tui::app::App;
use crate::tui::app_states::app_state::InputMode;
use crate::tui::app_states::state::KeyEventHandler;
use crossterm::event::{self, Event}; 
use std::time::Duration;

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? { // 'key' is the full KeyEvent
            let should_quit = match app.input_mode {
                InputMode::Normal => {
                                let mut handler = app.normal_state;
                                handler.handle_key_event(app, key)? // Pass key
                            }
                InputMode::Tagging | InputMode::Untagging => {
                                let mut handler = std::mem::take(&mut app.tag_state);
                                let result = handler.handle_key_event(app, key)?; // Pass key
                                app.tag_state = handler;

                                if let InputMode::Normal = app.input_mode {
                                    app.apply_filters();
                                }

                                result
                            }
                InputMode::Sorting => {
                                let mut handler = std::mem::take(&mut app.sort_state);
                                let result = handler.handle_key_event(app, key)?; // Pass key
                                app.sort_state = handler;

                                if let InputMode::Normal = app.input_mode {
                                    app.apply_filters();
                                }

                                result
                            }
                InputMode::Filtering => {
                                let mut handler = std::mem::take(&mut app.filter_state);
                                let result = handler.handle_key_event(app, key)?; // Pass key
                                app.filter_state = handler;

                                if let InputMode::Normal = app.input_mode {
                                    app.apply_filters();
                                }

                                result
                            }
                InputMode::Searching => {
                                let mut handler = app.search_state;
                                let result = handler.handle_key_event(app, key)?; // Pass key
                                result
                            }
                InputMode::Showing => {
                                let mut handler = std::mem::take(&mut app.show_mode_state);
                                let result = handler.handle_key_event(app, key)?; // Pass key
                                app.show_mode_state = handler;

                                if let InputMode::Normal = app.input_mode {
                                    app.apply_filters();
                                }

                                result
                            }
                InputMode::Action => {
                    let mut handler = std::mem::take(&mut app.action_state);
                    let result = handler.handle_key_event(app, key)?;
                    app.action_state = handler;

                    if let InputMode::Normal = app.input_mode {
                        app.apply_filters();
                    }
                    result
                },
            };

            if should_quit {
                return Ok(true);
            }
        }
    }
    Ok(false)
}