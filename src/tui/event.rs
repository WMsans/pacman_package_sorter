use crate::tui::app::App;
use crate::tui::app_states::app_state::InputMode;
use crate::tui::app_states::state::KeyEventHandler;
use crossterm::event::{self, Event};
use std::time::Duration;

pub fn handle_events(app: &mut App) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            let should_quit = match app.input_mode {
                InputMode::Normal => {
                    let mut handler = app.normal_state;
                    handler.handle_key_event(app, key.code)?
                }
                InputMode::Tagging | InputMode::Untagging => {
                    let mut handler = std::mem::take(&mut app.tag_state);
                    let result = handler.handle_key_event(app, key.code)?;
                    app.tag_state = handler;
                    result
                }
                InputMode::Sorting => {
                    let mut handler = std::mem::take(&mut app.sort_state);
                    let result = handler.handle_key_event(app, key.code)?;
                    app.sort_state = handler;
                    result
                }
                InputMode::Filtering => {
                    let mut handler = std::mem::take(&mut app.filter_state);
                    let result = handler.handle_key_event(app, key.code)?;
                    app.filter_state = handler;
                    result
                }
            };

            if should_quit {
                return Ok(true);
            }
        }
    }
    Ok(false)
}