use ratatui::widgets::ListState;

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub filtered_indices: Vec<usize>,
    pub filtered_spellbook_indices: Vec<usize>,
    pub results_state: ListState,
    pub showing_spellbooks: bool,
    pub in_command_mode: bool,
    pub search_active: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            filtered_indices: Vec::new(),
            filtered_spellbook_indices: Vec::new(),
            results_state: ListState::default(),
            showing_spellbooks: true,
            in_command_mode: false,
            search_active: false,
        }
    }
}

impl SearchState {
    pub fn open(&mut self) {
        self.query.clear();
        self.filtered_indices.clear();
        self.filtered_spellbook_indices.clear();
        self.results_state.select(Some(0));
        self.showing_spellbooks = true;
        self.in_command_mode = false;
        self.search_active = true;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.filtered_indices.clear();
        self.filtered_spellbook_indices.clear();
        self.results_state.select(None);
        self.search_active = false;
    }

    pub fn activate_search(&mut self) {
        self.search_active = true;
    }

    pub fn deactivate_search(&mut self) {
        self.query.clear();
        self.filtered_indices.clear();
        self.filtered_spellbook_indices.clear();
        self.search_active = false;
    }
}
