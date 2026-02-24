use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::io;

use super::app::App;
use super::render::ui;

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

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(None);
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
                KeyCode::Backspace => {
                    app.backspace_search();
                }
                KeyCode::Char(c) if !c.is_control() => {
                    app.add_to_search(c);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
