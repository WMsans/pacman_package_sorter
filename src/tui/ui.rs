use crate::tui::app::{App};
use crate::backend::FilterState;
use crate::tui::app_states::app_state::{ActionModalFocus, FilterFocus, InputMode, TagModalFocus};
use ratatui::layout::Position;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0), // Main content
            Constraint::Length(3), // Search bar
        ].as_ref())
        .split(frame.area());
    
    let content_area = main_layout[0];
    let search_area = main_layout[1];
    
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(content_area); 

    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(70), // Package list
                Constraint::Percentage(10), // Show Mode (NEW)
                Constraint::Percentage(10), // Filter
                Constraint::Percentage(10), // Sorting
            ]
            .as_ref(),
        )
        .split(horizontal_layout[0]); 

    render_package_list(frame, left_layout[0], app);
    render_show_mode(frame, left_layout[1], app); 
    render_filters(frame, left_layout[2], app); // Index changed
    render_sorting(frame, left_layout[3], app); // Index changed

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(70), // Package info
                Constraint::Percentage(15), // Actions
                Constraint::Percentage(15), // Output
            ]
            .as_ref(),
        )
        // [FIX] This should split horizontal_layout[1], not main_layout[1]
        .split(horizontal_layout[1]);

    render_package_info(frame, right_layout[0], app);
    render_actions(frame, right_layout[1], app);
    render_output_window(frame, right_layout[2], app);

    render_search_bar(frame, search_area, app);

    match app.input_mode {
        InputMode::Tagging | InputMode::Untagging => render_modal(frame, app),
        InputMode::Sorting => render_sort_modal(frame, app),
        InputMode::Filtering => render_filter_modal(frame, app),
        InputMode::Showing => render_show_mode_modal(frame, app),
        InputMode::Action => render_action_modal(frame, app),
        _ => {}
    }
}

fn render_package_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .state.filtered_packages
        .iter()
        .map(|p| ListItem::new(p.name.clone()))
        .collect();

    let title = if app.is_loading {
        "Packages (Loading...)"
    } else {
        "Packages"
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title)) // --- MODIFIED ---
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.selected_package);
}

fn render_package_info(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("Package Information")
        .borders(Borders::ALL);

    let info_text = if let Some(selected) = app.selected_package.selected() {
        if let Some(package) = app.state.filtered_packages.get(selected) {
            format!(
                "Name: {}\nVersion: {}\nRepository: {:?}\nDescription: {}\nInstalled: {}\nSize: {:.2} MiB\nTags: {}",
                package.name,
                package.version,
                package.repository,
                package.description,
                package.install_date.format("%Y-%m-%d"),
                package.size,
                package.tags.join(", ")
            )
        } else {
            "No package selected".to_string()
        }
    } else {
        "No packages found".to_string()
    };

    let paragraph = Paragraph::new(info_text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_show_mode(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Show Mode (v)").borders(Borders::ALL);
    let text = format!("Current: {}", app.show_mode_state.active_show_mode);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_filters(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Filter (f)").borders(Borders::ALL);
    let include_tags: Vec<String> = app
        .filter_state.tag_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Include)
        .map(|(k, _)| k.clone())
        .collect();
    let exclude_tags: Vec<String> = app
        .filter_state.tag_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Exclude)
        .map(|(k, _)| k.clone())
        .collect();
    let include_repos: Vec<String> = app
        .filter_state.repo_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Include)
        .map(|(k, _)| k.clone())
        .collect();
    let exclude_repos: Vec<String> = app
        .filter_state.repo_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Exclude)
        .map(|(k, _)| k.clone())
        .collect();

    let text = format!(
        "Include Tags: {}\nExclude Tags: {}\nInclude Repos: {}\nExclude Repos: {}",
        include_tags.join(", "),
        exclude_tags.join(", "),
        include_repos.join(", "),
        exclude_repos.join(", "),
    );
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_sorting(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Sort (s)").borders(Borders::ALL);
    let text = format!("Current: {}", app.sort_state.active_sort_key);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_actions(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Actions (?)").borders(Borders::ALL);
    let text = match app.input_mode {
        InputMode::Normal => "Actions:\n- (a)dd tag\n- (d)elete tag\n- (?) all actions",
        InputMode::Tagging => "Enter tag to add, then press Enter",
        InputMode::Untagging => "Enter tag to remove, then press Enter",
        _ => "",
    };
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_output_window(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Output").borders(Borders::ALL);
    let text = app.output.join("\n");
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_modal(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, frame.area());
    let title = if let InputMode::Tagging = app.input_mode { "Add Tag" } else { "Remove Tag" };
    let block = Block::default().title(title).borders(Borders::ALL);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let modal_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let input = Paragraph::new(app.tag_state.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input").border_style(match app.tag_state.focus {
                    TagModalFocus::Input => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),);
    frame.render_widget(input, modal_layout[0]);

    let tag_items: Vec<ListItem> = app.tag_state.filtered_tags.iter().map(|t| ListItem::new(t.clone())).collect();

    let tags_list = List::new(tag_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Existing Tags")
                // Add this border_style call
                .border_style(match app.tag_state.focus {
                    TagModalFocus::List => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(tags_list, modal_layout[1], &mut app.tag_state.selection);
}

fn render_show_mode_modal(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, frame.area());
    let block = Block::default().title("Show Mode").borders(Borders::ALL);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .show_mode_state.options
        .iter()
        .map(|key| ListItem::new(key.to_string()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area.inner(Margin { horizontal: 1, vertical: 1 }), &mut app.show_mode_state.selection);
}

fn render_sort_modal(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, frame.area());
    let block = Block::default().title("Sort by").borders(Borders::ALL);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .sort_state.options
        .iter()
        .map(|key| ListItem::new(key.to_string()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area.inner(Margin { horizontal: 1, vertical: 1 }), &mut app.sort_state.selection);
}

fn render_filter_modal(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 80, frame.area());
    let block = Block::default().title("Filter by").borders(Borders::ALL);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Input for fuzzy search
                Constraint::Min(0),    // Lists
            ]
            .as_ref(),
        )
        .split(area);

    let input = Paragraph::new(app.filter_state.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(match app.filter_state.focus {
                    FilterFocus::Search => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        );
    frame.render_widget(input, chunks[0]);
    if let FilterFocus::Search = app.filter_state.focus {
        frame.set_cursor_position(
                Position{
                    x: chunks[0].x + app.filter_state.cursor_position as u16 + 1,
                    y: chunks[0].y + 1,
                }
        );
    }

    let list_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // Tags list
    let tag_items: Vec<ListItem> = app
        .filter_state.filtered_tags
        .iter()
        .map(|t| {
            let state = app.filter_state.tag_filters.get(t).cloned().unwrap_or_default();
            let prefix = match state {
                FilterState::Include => "[+] ",
                FilterState::Exclude => "[-] ",
                FilterState::Ignore => "[ ] ",
            };
            ListItem::new(format!("{}{}", prefix, t))
        })
        .collect();

    let tags_list = List::new(tag_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tags")
                .border_style(match app.filter_state.focus {
                    FilterFocus::Tags => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");
    frame.render_stateful_widget(tags_list, list_chunks[0], &mut app.filter_state.tag_selection);

    // Repos list
    let repo_items: Vec<ListItem> = app
        .filter_state.filtered_repos
        .iter()
        .map(|r| {
            let state = app.filter_state.repo_filters.get(r).cloned().unwrap_or_default();
            let prefix = match state {
                FilterState::Include => "[+] ",
                FilterState::Exclude => "[-] ",
                FilterState::Ignore => "[ ] ",
            };
            ListItem::new(format!("{}{}", prefix, r))
        })
        .collect();

    let repos_list = List::new(repo_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Repositories")
                .border_style(match app.filter_state.focus {
                    FilterFocus::Repos => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");
    frame.render_stateful_widget(repos_list, list_chunks[1], &mut app.filter_state.repo_selection);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn render_search_bar(frame: &mut Frame, area: Rect, app: &App) {
    let input = Paragraph::new(app.search_input.as_str())
        .style(match app.input_mode {
            InputMode::Searching => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search (/)")
                .border_style(match app.input_mode {
                    InputMode::Searching => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        );
    frame.render_widget(input, area);

    if let InputMode::Searching = app.input_mode {
        frame.set_cursor_position(Position {
            x: area.x + app.search_cursor_position as u16 + 1,
            y: area.y + 1,
        });
    }
}
fn render_action_modal(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, frame.area());
    let title = "Actions";
    let block = Block::default().title(title).borders(Borders::ALL);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let modal_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Input
                Constraint::Min(0),    // List
            ]
            .as_ref(),
        )
        .split(area);

    let input = Paragraph::new(app.action_state.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search").border_style(match app.action_state.focus {
                    ActionModalFocus::Input => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),);
    frame.render_widget(input, modal_layout[0]);

    let action_items: Vec<ListItem> = app.action_state.filtered_options.iter().map(|t| ListItem::new(t.clone())).collect();

    let actions_list = List::new(action_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .border_style(match app.action_state.focus {
                    ActionModalFocus::List => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(actions_list, modal_layout[1], &mut app.action_state.selection);
}