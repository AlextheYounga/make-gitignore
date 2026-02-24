use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::app::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
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

    // Search box
    let search_text = if app.search_query.is_empty() {
        "Type to filter...".to_string()
    } else {
        app.search_query.clone()
    };
    let search_style = if app.search_query.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let search = Paragraph::new(search_text)
        .style(search_style)
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search, chunks[1]);

    // Calculate visible area
    let list_height = (chunks[2].height as usize).saturating_sub(2);

    // Get filtered languages
    let filtered_indices = app.get_filtered_indices();

    // Find cursor position in filtered list
    let cursor_in_filtered = filtered_indices
        .iter()
        .position(|&i| i == app.cursor_position)
        .unwrap_or(0);

    // Calculate scroll offset based on filtered list
    let scroll_offset = if cursor_in_filtered < app.scroll_offset {
        cursor_in_filtered
    } else if cursor_in_filtered >= app.scroll_offset + list_height {
        cursor_in_filtered.saturating_sub(list_height - 1)
    } else {
        app.scroll_offset
    };

    // Language list
    let items: Vec<ListItem> = filtered_indices
        .iter()
        .skip(scroll_offset)
        .take(list_height)
        .map(|&i| {
            let lang = &app.languages[i];
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

    let list_title = if filtered_indices.is_empty() {
        "No matches".to_string()
    } else {
        format!("Languages ({} matches)", filtered_indices.len())
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(list_title));
    f.render_widget(list, chunks[2]);

    // Footer with selection count
    let selected_count = app.selected_indices.iter().filter(|&&s| s).count();
    let footer = Paragraph::new(format!("Selected: {} language(s)", selected_count))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[3]);
}
