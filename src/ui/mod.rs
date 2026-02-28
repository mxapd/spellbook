use ratatui::widgets::ListState;

pub mod events;
pub mod render;
pub mod search_overlay;
pub mod spell_list;
pub mod spellbook_list;

pub use events::handle_event;
pub use render::render;

#[derive(PartialEq)]
pub enum Screen {
    SpellbookList,
    SpellList,
    SearchOverlay { return_to: SearchContext },
}

#[derive(PartialEq, Clone)]
pub enum SearchContext {
    SpellbookList,
    SpellList,
}

pub struct UiState {
    pub screen: Screen,
    pub spellbook_list_state: ListState,
    pub spell_list_state: ListState,
    pub selected_spellbook: Option<usize>,
    pub search_query: String,
    /// Filtered indices into the active list
    pub filtered_indices: Vec<usize>,
}

impl UiState {
    pub fn new() -> Self {
        let mut spellbook_list_state = ListState::default();
        spellbook_list_state.select(Some(0));

        Self {
            screen: Screen::SpellbookList,
            spellbook_list_state,
            spell_list_state: ListState::default(),
            selected_spellbook: None,
            search_query: String::new(),
            filtered_indices: Vec::new(),
        }
    }
}

