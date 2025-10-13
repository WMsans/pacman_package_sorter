use std::io::{self, stdout, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::backend;
use crate::models::Package;

pub fn run_tui() -> io::Result<()> {
    let mut terminal = init_terminal()?;
    let mut app = App::new();
    app.run(&mut terminal)?;
    restore_terminal()
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

struct App {
    packages: Vec<Package>,
    selected_package: Option<usize>,
}

impl App {
    fn new() -> Self {
        App {
            packages: Vec::new(),
            selected_package: None,
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.packages = backend::get_all_packages().await.unwrap_or_default();
        });

        if !self.packages.is_empty() {
            self.selected_package = Some(0);
        }

        loop {
            terminal.draw(|f| self.ui(f))?;
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up => self.select_previous(),
                        KeyCode::Down => self.select_next(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn select_previous(&mut self) {
        if let Some(selected) = self.selected_package {
            if selected > 0 {
                self.selected_package = Some(selected - 1);
            }
        }
    }

    fn select_next(&mut self) {
        if let Some(selected) = self.selected_package {
            if selected < self.packages.len() - 1 {
                self.selected_package = Some(selected + 1);
            }
        }
    }

    fn ui(&self, frame: &mut Frame) {
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

        self.render_package_list(frame, left_layout[0]);
        self.render_filters(frame, left_layout[1]);
        self.render_sorting(frame, left_layout[2]);

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

        self.render_package_info(frame, right_layout[0]);
        self.render_actions(frame, right_layout[1]);
    }

    fn render_package_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .packages
            .iter()
            .map(|p| ListItem::new(p.name.clone()))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Packages"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(self.selected_package);

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_package_info(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Package Information")
            .borders(Borders::ALL);

        let info_text = if let Some(selected) = self.selected_package {
            if let Some(package) = self.packages.get(selected) {
                // In a real application, you would run `pacman -Qi` here
                // For now, we'll just display the info we have.
                format!(
                    "Name: {}\nVersion: {}\nRepository: {:?}\nDescription: {}\nInstalled: {}\nSize: {:.2} MiB",
                    package.name,
                    package.version,
                    package.repository,
                    package.description,
                    package.install_date.format("%Y-%m-%d"),
                    package.size
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

    fn render_filters(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("Filter").borders(Borders::ALL);
        let text = "Filter by:\n- (T)ag\n- (R)epo\n- (E)xplicit/(D)ependency";
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_sorting(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("Sort").borders(Borders::ALL);
        let text = "Sort by:\n- (N)ame\n- (S)ize\n- (I)nstalled Date";
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_actions(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("Actions").borders(Borders::ALL);
        let text = "Actions:\n- Add (A) tag\n- (U)ntag";
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
}