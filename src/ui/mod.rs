use ratatui::widgets::ListState;

pub use crate::models::ViewMode;

pub mod add_spell;
pub mod events;
pub mod render;
pub mod search_overlay;
pub mod spell_list;
pub mod spellbook_list;

pub use events::filter_commands;
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

/// Tracks the current mode within SearchOverlay
#[derive(PartialEq, Clone, Copy, Default)]
pub enum SearchMode {
    #[default]
    BrowseSpellbooks, // Default - show cards/spines
    BrowseSpells, // Show spells in selected spellbook
    AddSpell,     // Add new spell form
    AddSpellbook, // Add new spellbook form
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
    /// Whether the spellbook dropdown is currently open
    pub add_spell_dropdown_open: bool,
    /// Whether user is currently typing in a text field (disables theme toggle)
    pub is_typing: bool,
    /// Feedback message to display in AddSpell form
    pub add_spell_message: Option<(String, bool)>, // (message, is_error)
    /// Tracks whether there are unsaved changes in the form
    pub add_spell_has_unsaved: bool,
    /// Temporary feedback message (e.g., "Copied!")
    pub copy_feedback: Option<String>,
    /// Whether showing spellbook browser (true) or search results (false)
    pub search_showing_spellbooks: bool,
    /// Selected spellbook index when browsing spellbooks in search
    pub search_spellbook_index: Option<usize>,
    /// Horizontal scroll offset for spellbook spine view
    pub search_spellbook_scroll: usize,
    /// Spines per row calculated during render
    pub search_spines_per_row: usize,
    /// Last render dimensions (for resize detection)
    pub search_last_width: u16,
    pub search_last_height: u16,
    /// View mode for spellbook browser (auto/cards/spines)
    pub view_mode: ViewMode,
    /// Current mode within SearchOverlay
    pub search_mode: SearchMode,
    /// Unified items per row (used for navigation - cards or spines based on current view)
    pub search_items_per_row: usize,
    /// Add spellbook form fields
    pub add_spellbook_name: String,
    pub add_spellbook_cover: String,
    pub add_spellbook_sigil: String,
    /// Whether we're in command palette mode (searching commands after :)
    pub search_in_command_mode: bool,
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
            add_spell_dropdown_open: false,
            is_typing: false,
            add_spell_message: None,
            add_spell_has_unsaved: false,
            copy_feedback: None,
            search_showing_spellbooks: true,
            search_spellbook_index: Some(0),
            search_spellbook_scroll: 0,
            search_spines_per_row: 1,
            search_last_width: 0,
            search_last_height: 0,
            view_mode: ViewMode::default(),
            search_mode: SearchMode::BrowseSpellbooks,
            search_items_per_row: 1,
            add_spellbook_name: String::new(),
            add_spellbook_cover: String::new(),
            add_spellbook_sigil: String::new(),
            search_in_command_mode: false,
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
        self.is_typing = true;
        self.search_showing_spellbooks = true;
        self.search_spellbook_index = Some(0);
        self.search_spellbook_scroll = 0;
        self.search_spines_per_row = 1;
        self.search_last_width = 0;
        self.search_last_height = 0;
    }

    /// Call when leaving text fields (typing mode disabled)
    pub fn exit_typing_mode(&mut self) {
        self.is_typing = false;
    }

    /// Clears all form state and resets to spellbook list
    pub fn clear_add_spell_form(&mut self) {
        self.add_spell_name.clear();
        self.add_spell_command.clear();
        self.add_spell_lore.clear();
        self.add_spell_school.clear();
        self.add_spell_tags.clear();
        self.add_spell_spellbook = None;
        self.add_spell_skip_spellbook = false;
        self.add_spell_dropdown_open = false;
        self.add_spell_message = None;
        self.add_spell_has_unsaved = false;
        self.add_spell_field = AddSpellField::Name;
        self.screen = Screen::SpellbookList;
        self.exit_typing_mode();
    }

    /// Updates is_typing based on the current AddSpell field
    pub fn update_typing_state(&mut self) {
        match &self.screen {
            Screen::AddSpell => {
                self.is_typing = matches!(
                    self.add_spell_field,
                    AddSpellField::Name
                        | AddSpellField::Command
                        | AddSpellField::Lore
                        | AddSpellField::School
                        | AddSpellField::Tags
                );
            }
            Screen::SearchOverlay { .. } => {
                self.is_typing = true;
            }
            _ => {
                self.is_typing = false;
            }
        }
    }
}
