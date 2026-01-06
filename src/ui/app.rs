pub struct App {
    pub(super) languages: Vec<String>,
    pub(super) selected_indices: Vec<bool>,
    pub(super) cursor_position: usize,
    pub(super) scroll_offset: usize,
    pub(super) search_query: String,
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
            search_query: String::new(),
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

    pub fn get_filtered_indices(&self) -> Vec<usize> {
        if self.search_query.is_empty() {
            (0..self.languages.len()).collect()
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.languages
                .iter()
                .enumerate()
                .filter(|(_, lang)| lang.to_lowercase().contains(&query_lower))
                .map(|(i, _)| i)
                .collect()
        }
    }

    pub fn add_to_search(&mut self, c: char) {
        self.search_query.push(c);
        self.reset_cursor_if_needed();
    }

    pub fn backspace_search(&mut self) {
        self.search_query.pop();
        self.reset_cursor_if_needed();
    }

    fn reset_cursor_if_needed(&mut self) {
        let filtered = self.get_filtered_indices();
        if filtered.is_empty() {
            self.cursor_position = 0;
        } else if self.cursor_position >= self.languages.len() || !filtered.contains(&self.cursor_position) {
            // Current cursor is not in filtered list, move to first filtered item
            self.cursor_position = filtered[0];
        }
        self.scroll_offset = 0;
    }

    pub fn move_cursor_up(&mut self) {
        let filtered = self.get_filtered_indices();
        if let Some(pos) = filtered.iter().position(|&i| i == self.cursor_position) {
            if pos > 0 {
                self.cursor_position = filtered[pos - 1];
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        let filtered = self.get_filtered_indices();
        if let Some(pos) = filtered.iter().position(|&i| i == self.cursor_position) {
            if pos < filtered.len() - 1 {
                self.cursor_position = filtered[pos + 1];
            }
        }
    }

    pub fn toggle_selection(&mut self) {
        if self.cursor_position < self.selected_indices.len() {
            self.selected_indices[self.cursor_position] =
                !self.selected_indices[self.cursor_position];
        }
    }

    pub fn update_scroll(&mut self, visible_height: usize) {
        // Ensure cursor is visible
        if self.cursor_position < self.scroll_offset {
            self.scroll_offset = self.cursor_position;
        } else if self.cursor_position >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor_position - visible_height + 1;
        }
    }
}