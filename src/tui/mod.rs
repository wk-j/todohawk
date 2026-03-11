mod event;
mod run;
mod ui;

pub use run::run_tui;

use std::collections::HashSet;

use crate::types::{Tag, TodoItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Search,
    Detail,
}

pub struct App {
    pub items: Vec<TodoItem>,
    pub filtered_items: Vec<usize>,
    pub selected: usize,
    pub active_tag_filters: HashSet<Tag>,
    pub search_query: String,
    pub mode: AppMode,
}

impl App {
    pub fn new(items: Vec<TodoItem>) -> Self {
        let filtered_items: Vec<usize> = (0..items.len()).collect();
        Self {
            items,
            filtered_items,
            selected: 0,
            active_tag_filters: HashSet::new(),
            search_query: String::new(),
            mode: AppMode::Normal,
        }
    }

    pub fn apply_filters(&mut self) {
        self.filtered_items = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if !self.active_tag_filters.is_empty()
                    && !self.active_tag_filters.contains(&item.tag)
                {
                    return false;
                }
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    let matches = item.message.to_lowercase().contains(&query)
                        || item.file.to_string_lossy().to_lowercase().contains(&query)
                        || item
                            .author
                            .as_ref()
                            .is_some_and(|a| a.to_lowercase().contains(&query));
                    if !matches {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        // Clamp selection to valid range
        if self.filtered_items.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered_items.len() {
            self.selected = self.filtered_items.len() - 1;
        }
    }

    pub fn toggle_tag_filter(&mut self, tag: Tag) {
        if self.active_tag_filters.contains(&tag) {
            self.active_tag_filters.remove(&tag);
        } else {
            self.active_tag_filters.insert(tag);
        }
        self.apply_filters();
    }

    pub fn next(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.filtered_items.len() - 1);
        }
    }

    pub fn selected_item(&self) -> Option<&TodoItem> {
        let &idx = self.filtered_items.get(self.selected)?;
        self.items.get(idx)
    }
}
