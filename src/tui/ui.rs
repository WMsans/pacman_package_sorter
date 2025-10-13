use crate::tui::app::{App, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
                Constraint::Percentage(60), // Package list
                Constraint::Percentage(20), // Filter
                Constraint::Percentage(20), // Sorting
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
                Constraint::Percentage(80), // Package info
                Constraint::Percentage(20), // Actions
            ]
            .as_ref(),
        )
        .split(main_layout[1]);

    render_package_info(frame, right_layout[0], app);
    render_actions(frame, right_layout[1], app);
}

fn render_package_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .packages
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
        if let Some(package) = app.packages.get(selected) {
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
    let block = Block::default().title("Filter").borders(Borders::ALL);
    let text = format!(
        "Filter by:\n- (T)ag: {}\n- (R)epo: {}\n- (E)xplicit/(D)ependency",
        app.filter_tag.as_deref().unwrap_or("None"),
        app.filter_repo.as_deref().unwrap_or("None")
    );
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_sorting(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Sort").borders(Borders::ALL);
    let text = format!(
        "Sort by:\n- (N)ame\n- (S)ize\n- (I)nstalled Date\n\nCurrent: {:?}",
        app.sort_key
    );
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_actions(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Actions").borders(Borders::ALL);
    let text = match app.input_mode {
        InputMode::Normal => "Actions:\n- Add (A) tag\n- (U)ntag",
        InputMode::Editing => "Enter tag name, then press Enter",
    };
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}