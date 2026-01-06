use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io;

pub struct App {
    languages: Vec<String>,
    selected_indices: Vec<bool>,
    cursor_position: usize,
    scroll_offset: usize,
}

impl App {
    pub fn new(mut languages: Vec<String>) -> Self {
        languages.sort();
        let len = languages.len();
        Self {
            languages,
            selected_indices: vec![false; len],
            cursor_position: 0,
            scroll_offset: 0,
        }
    }

    pub fn get_selected_languages(&self) -> Vec<String> {
        self.selected_indices
            .iter()
            .enumerate()
            .filter(|(_, selected)| **selected)
            .map(|(i, _)| self.languages[i].clone())
            .collect()
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_position < self.languages.len() - 1 {
            self.cursor_position += 1;
        }
    }

    fn toggle_selection(&mut self) {
        if self.cursor_position < self.selected_indices.len() {
            self.selected_indices[self.cursor_position] =
                !self.selected_indices[self.cursor_position];
        }
    }

    fn update_scroll(&mut self, visible_height: usize) {
        // Ensure cursor is visible
        if self.cursor_position < self.scroll_offset {
            self.scroll_offset = self.cursor_position;
        } else if self.cursor_position >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor_position - visible_height + 1;
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new(
        "Select .gitignore templates (Space to toggle, Enter to confirm, Esc to cancel)",
    )
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Calculate visible area
    let list_height = (chunks[1].height as usize).saturating_sub(2); // Account for borders
    app.update_scroll(list_height);

    // Language list
    let items: Vec<ListItem> = app
        .languages
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(list_height)
        .map(|(i, lang)| {
            let is_selected = app.selected_indices[i];
            let is_cursor = i == app.cursor_position;

            let checkbox = if is_selected { "[✓]" } else { "[ ]" };
            let content = format!("{} {}", checkbox, lang);

            let style = if is_cursor {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
        "Languages ({}/{})",
        app.languages.len(),
        app.languages.len()
    )));
    f.render_widget(list, chunks[1]);

    // Footer with selection count
    let selected_count = app.selected_indices.iter().filter(|&&s| s).count();
    let footer = Paragraph::new(format!("Selected: {} language(s)", selected_count))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

pub fn run_ui(languages: Vec<String>) -> io::Result<Option<Vec<String>>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(languages);
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<Option<Vec<String>>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(None); // User cancelled
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.move_cursor_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.move_cursor_down();
                    }
                    KeyCode::Char(' ') => {
                        app.toggle_selection();
                    }
                    KeyCode::Enter => {
                        let selected = app.get_selected_languages();
                        return Ok(Some(selected));
                    }
                    _ => {}
                }
            }
        }
    }
}
