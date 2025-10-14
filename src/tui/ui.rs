use crate::tui::app::{App, FilterFocus, InputMode};
use crate::backend::FilterState;
use ratatui::layout::Position;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(frame.area());

    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(80), // Package list
                Constraint::Percentage(10), // Filter
                Constraint::Percentage(10), // Sorting
            ]
            .as_ref(),
        )
        .split(main_layout[0]);

    render_package_list(frame, left_layout[0], app);
    render_filters(frame, left_layout[1], app);
    render_sorting(frame, left_layout[2], app);

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
        .split(main_layout[1]);

    render_package_info(frame, right_layout[0], app);
    render_actions(frame, right_layout[1], app);
    render_output_window(frame, right_layout[2], app);

    if let InputMode::Tagging | InputMode::Untagging = app.input_mode {
        render_modal(frame, app);
    }
    if let InputMode::Sorting = app.input_mode {
        render_sort_modal(frame, app);
    }
    if let InputMode::Filtering = app.input_mode {
        render_filter_modal(frame, app);
    }
}

fn render_package_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .filtered_packages
        .iter()
        .map(|p| ListItem::new(p.name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Packages"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.selected_package);
}

fn render_package_info(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("Package Information")
        .borders(Borders::ALL);

    let info_text = if let Some(selected) = app.selected_package.selected() {
        if let Some(package) = app.filtered_packages.get(selected) {
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

fn render_filters(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Filter (f)").borders(Borders::ALL);
    let include_tags: Vec<String> = app
        .tag_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Include)
        .map(|(k, _)| k.clone())
        .collect();
    let exclude_tags: Vec<String> = app
        .tag_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Exclude)
        .map(|(k, _)| k.clone())
        .collect();
    let include_repos: Vec<String> = app
        .repo_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Include)
        .map(|(k, _)| k.clone())
        .collect();
    let exclude_repos: Vec<String> = app
        .repo_filters
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
    let text = format!("Current: {}", app.sort_key);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_actions(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Actions").borders(Borders::ALL);
    let text = match app.input_mode {
        InputMode::Normal => "Actions:\n- Add (A) tag\n- (D)elete tag",
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

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    frame.render_widget(input, modal_layout[0]);

    let tag_items: Vec<ListItem> = app.filtered_tags.iter().map(|t| ListItem::new(t.clone())).collect();

    let tags_list = List::new(tag_items)
        .block(Block::default().borders(Borders::ALL).title("Existing Tags"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(tags_list, modal_layout[1], &mut app.tag_selection);
}

fn render_sort_modal(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, frame.area());
    let block = Block::default().title("Sort by").borders(Borders::ALL);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .sort_options
        .iter()
        .map(|key| ListItem::new(key.to_string()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area.inner(Margin { horizontal: 1, vertical: 1 }), &mut app.sort_selection);
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

    let input = Paragraph::new(app.filter_input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(match app.filter_focus {
                    FilterFocus::Search => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        );
    frame.render_widget(input, chunks[0]);
    if let FilterFocus::Search = app.filter_focus {
        frame.set_cursor_position(
                Position{
                    x: chunks[0].x + app.filter_cursor_position as u16 + 1,
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
        .filtered_tags
        .iter()
        .map(|t| {
            let state = app.tag_filters.get(t).cloned().unwrap_or_default();
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
                .border_style(match app.filter_focus {
                    FilterFocus::Tags => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");
    frame.render_stateful_widget(tags_list, list_chunks[0], &mut app.tag_filter_selection);

    // Repos list
    let repo_items: Vec<ListItem> = app
        .filtered_repos
        .iter()
        .map(|r| {
            let state = app.repo_filters.get(r).cloned().unwrap_or_default();
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
                .border_style(match app.filter_focus {
                    FilterFocus::Repos => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");
    frame.render_stateful_widget(repos_list, list_chunks[1], &mut app.repo_filter_selection);
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