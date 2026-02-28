use ratatui::widgets::ListState;

pub mod render;
pub mod spell_list;
pub mod spellbook_list;

pub use render::render;

pub enum Screen {
    SpellbookList,
    SpellList,
}

pub struct UiState {
    pub screen: Screen,
    pub spellbook_list_state: ListState,
    pub spell_list_state: ListState,
    pub selected_spellbook: Option<usize>,
}

impl UiState {
    pub fn new() -> Self {
        let mut spellbook_list_state = ListState::default();
        spellbook_list_state.select(Some(0)); // start with first item selected

        Self {
            screen: Screen::SpellbookList,
            spellbook_list_state,
            spell_list_state: ListState::default(),
            selected_spellbook: None,
        }
    }
}
