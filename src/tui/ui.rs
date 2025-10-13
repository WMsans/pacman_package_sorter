use crate::tui::app::{App, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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

    // Render modal last to appear on top
    if let InputMode::Tagging | InputMode::Untagging = app.input_mode {
        render_modal(frame, app);
    }
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
        InputMode::Tagging => "Enter tag to add, then press Enter",
        InputMode::Untagging => "Enter tag to remove, then press Enter",
    };
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

// New function
fn render_output_window(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Output").borders(Borders::ALL);
    let text = app.output.join("\n");
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

// New function
fn render_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, frame.area());
    let title = if let InputMode::Tagging = app.input_mode { "Add Tag" } else { "Remove Tag" };
    let block = Block::default().title(title).borders(Borders::ALL);
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(block, area);

    // We need to calculate the inner area for the input paragraph
    let inner_area = Layout::default()
        .margin(1)
        .constraints([Constraint::Percentage(100)])
        .split(area)[0];

    frame.render_widget(input, inner_area);
}

/// helper function to create a centered rect using up certain percentage of the screen
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