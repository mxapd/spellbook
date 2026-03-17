use ratatui::widgets::ListState;

pub mod add_spell;
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
    AddSpell,
}

#[derive(PartialEq, Clone)]
pub enum SearchContext {
    SpellbookList,
    SpellList,
}

/// Tracks the current input field in AddSpell screen
#[derive(PartialEq, Clone, Copy)]
pub enum AddSpellField {
    Name,
    Command,
    Lore,
    School,
    Tags,
    Spellbook,
    Save,
    Cancel,
}

pub struct UiState {
    pub screen: Screen,
    pub spellbook_list_state: ListState,
    pub spell_list_state: ListState,
    pub search_list_state: ListState,
    pub selected_spellbook: Option<usize>,
    pub search_query: String,
    /// Filtered indices into the active list
    pub filtered_indices: Vec<usize>,
    /// Tracks which screen to return to when search closes
    pub search_return_to: SearchContext,
    /// Current add spell form field
    pub add_spell_field: AddSpellField,
    /// Add spell form values
    pub add_spell_name: String,
    pub add_spell_command: String,
    pub add_spell_lore: String,
    pub add_spell_school: String,
    pub add_spell_tags: String,
    pub add_spell_spellbook: Option<usize>,
    pub add_spell_skip_spellbook: bool,
    pub add_spell_dropdown_index: usize,
}

impl UiState {
    pub fn new(start_in_add_mode: bool) -> Self {
        let mut spellbook_list_state = ListState::default();
        spellbook_list_state.select(Some(0));

        let mut search_list_state = ListState::default();
        search_list_state.select(Some(0));

        let screen = if start_in_add_mode {
            Screen::AddSpell
        } else {
            Screen::SpellbookList
        };

        Self {
            screen,
            spellbook_list_state,
            spell_list_state: ListState::default(),
            search_list_state,
            selected_spellbook: None,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            search_return_to: SearchContext::SpellbookList,
            add_spell_field: AddSpellField::Name,
            add_spell_name: String::new(),
            add_spell_command: String::new(),
            add_spell_lore: String::new(),
            add_spell_school: String::new(),
            add_spell_tags: String::new(),
            add_spell_spellbook: None,
            add_spell_skip_spellbook: false,
            add_spell_dropdown_index: 0,
        }
    }

    /// Opens the search overlay from the current screen.
    /// The return_to context determines which screen we return to when search closes.
    pub fn open_search(&mut self, return_to: SearchContext) {
        self.search_query.clear();
        self.filtered_indices.clear();
        self.search_list_state.select(Some(0));
        self.search_return_to = return_to.clone();
        self.screen = Screen::SearchOverlay { return_to };
    }
}
