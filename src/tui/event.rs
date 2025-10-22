use crate::tui::app::App;
use crate::tui::app_states::app_state::InputMode;
use crate::tui::app_states::state::KeyEventHandler;
use crossterm::event::{self, Event, MouseEventKind}; 
use std::time::Duration; 

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {

    if event::poll(Duration::from_millis(100))? {

        match event::read()? {
            Event::Key(key) => {

                let should_quit = match app.input_mode {
                    InputMode::Normal => {
                        let mut handler = app.normal_state;
                        handler.handle_key_event(app, key)? 
                    }
                    InputMode::Tagging | InputMode::Untagging => {
                        let mut handler = std::mem::take(&mut app.tag_state);
                        let result = handler.handle_key_event(app, key)?; 
                        app.tag_state = handler;

                        if let InputMode::Normal = app.input_mode {
                            app.apply_filters();
                        }

                        result
                    }
                    InputMode::Sorting => {
                        let mut handler = std::mem::take(&mut app.sort_state);
                        let result = handler.handle_key_event(app, key)?; 
                        app.sort_state = handler;

                        if let InputMode::Normal = app.input_mode {
                            app.apply_filters();
                        }

                        result
                    }
                    InputMode::Filtering => {
                        let mut handler = std::mem::take(&mut app.filter_state);
                        let result = handler.handle_key_event(app, key)?; 
                        app.filter_state = handler;

                        if let InputMode::Normal = app.input_mode {
                            app.apply_filters();
                        }

                        result
                    }
                    InputMode::Searching => {
                        let mut handler = app.search_state;
                        let result = handler.handle_key_event(app, key)?; 
                        result
                    }
                    InputMode::Showing => {
                        let mut handler = std::mem::take(&mut app.show_mode_state);
                        let result = handler.handle_key_event(app, key)?; 
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
                    }
                };

                if should_quit {
                    return Ok(true);
                }
            }
            Event::Mouse(mouse_event) => {

                if let InputMode::Normal = app.input_mode {

                    // Check for package list scroll
                    let pkg_area = app.package_list_area;
                    let is_inside_pkg_list = mouse_event.row >= pkg_area.y
                        && mouse_event.row < pkg_area.y + pkg_area.height
                        && mouse_event.column >= pkg_area.x
                        && mouse_event.column < pkg_area.x + pkg_area.width;

                    if is_inside_pkg_list {
                        match mouse_event.kind {
                            MouseEventKind::ScrollUp => {
                                app.select_previous_package();
                            }
                            MouseEventKind::ScrollDown => {
                                app.select_next_package();
                            }
                            _ => {}
                        }
                    }

                    // Check for output log scroll
                    let log_area = app.output_log_area;
                    let is_inside_log = mouse_event.row >= log_area.y
                        && mouse_event.row < log_area.y + log_area.height
                        && mouse_event.column >= log_area.x
                        && mouse_event.column < log_area.x + log_area.width;

                    if is_inside_log {
                        match mouse_event.kind {
                            MouseEventKind::ScrollUp => {
                                app.output.scroll_up(1);
                            }
                            MouseEventKind::ScrollDown => {
                                app.output.scroll_down(1);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {} 
        }
    }

    Ok(false)
}