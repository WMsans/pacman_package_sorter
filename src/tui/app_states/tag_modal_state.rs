use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::ListState;

/// Manages the state for the tagging/untagging functionality
pub struct TagModalState {
    pub input: String,
    pub filtered_tags: Vec<String>,
    pub selection: ListState,
}

impl TagModalState {
    pub fn new(all_tags: &[String]) -> Self {
        Self {
            input: String::new(),
            filtered_tags: all_tags.to_vec(),
            selection: ListState::default(),
        }
    }

    /// Update the filtered tags based on the input.
    pub fn update_filtered_tags(&mut self, all_tags: &[String]) {
        let matcher = SkimMatcherV2::default();
        if self.input.is_empty() {
            self.filtered_tags = all_tags.to_vec();
        } else {
            self.filtered_tags = all_tags
                .iter()
                .filter(|tag| matcher.fuzzy_match(tag, &self.input).is_some())
                .cloned()
                .collect();
        }
        self.selection.select(if self.filtered_tags.is_empty() { None } else { Some(0) });
    }
    
    pub fn select_previous_tag(&mut self) {
        if self.filtered_tags.is_empty() { return; }
        let i = match self.selection.selected() {
            Some(i) => if i == 0 { self.filtered_tags.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }

    pub fn select_next_tag(&mut self) {
        if self.filtered_tags.is_empty() { return; }
        let i = match self.selection.selected() {
            Some(i) => if i >= self.filtered_tags.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.selection.select(Some(i));
        if let Some(tag) = self.filtered_tags.get(i) {
            self.input = tag.clone();
        }
    }
}