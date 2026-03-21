use ratatui::widgets::ListState;

pub use crate::clipboard::ExecutionResult;
pub use crate::models::ViewMode;

pub mod add_spell;
pub mod add_spell_form;
pub mod add_spellbook_form;
pub mod confirm;
pub mod events;
pub mod input;
pub mod jobs;
pub mod render;
pub mod search_overlay;
pub mod search_state;
pub mod spell_list;
pub mod spellbook_browser;

pub use add_spell_form::{AddSpellField, AddSpellForm};
pub use add_spellbook_form::AddSpellbookForm;
pub use confirm::ConfirmDialogState;
pub use events::filter_commands;
pub use events::handle_event;
pub use input::{InputPhase, InputPopupState};
pub use jobs::JobsPanelState;
pub use render::render;
pub use search_state::SearchState;
pub use spellbook_browser::SpellbookBrowserState;

#[derive(PartialEq, Clone, Copy)]
pub enum Screen {
    SpellList,
    SearchOverlay,
    AddSpell,
    OutputPopup,
    JobsPanel,
    ConfirmDialog,
    InputPopup,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum SearchMode {
    #[default]
    BrowseSpellbooks,
    BrowseSpells,
    AddSpell,
    AddSpellbook,
}

pub struct UiState {
    pub screen: Screen,
    pub spell_list_state: ListState,
    pub selected_spellbook: Option<usize>,
    pub is_typing: bool,
    pub copy_feedback: Option<String>,
    pub search_mode: SearchMode,
    pub view_mode: ViewMode,
    pub search: SearchState,
    pub spellbook_browser: SpellbookBrowserState,
    pub add_spell: AddSpellForm,
    pub add_spellbook: AddSpellbookForm,
    pub output_popup: Option<ExecutionResult>,
    pub previous_screen: Option<Screen>,
    pub needs_redraw: bool,
    pub jobs_panel_state: JobsPanelState,
    pub confirm_dialog: Option<ConfirmDialogState>,
    pub input_popup: Option<InputPopupState>,
}

impl UiState {
    pub fn new(start_in_add_mode: bool) -> Self {
        let screen = if start_in_add_mode {
            Screen::AddSpell
        } else {
            Screen::SearchOverlay
        };

        Self {
            screen,
            spell_list_state: ListState::default(),
            selected_spellbook: None,
            is_typing: false,
            copy_feedback: None,
            search_mode: SearchMode::default(),
            view_mode: ViewMode::default(),
            search: SearchState::default(),
            spellbook_browser: SpellbookBrowserState::default(),
            add_spell: AddSpellForm::default(),
            add_spellbook: AddSpellbookForm::default(),
            output_popup: None,
            previous_screen: None,
            needs_redraw: false,
            jobs_panel_state: JobsPanelState::default(),
            confirm_dialog: None,
            input_popup: None,
        }
    }

    pub fn open_search(&mut self) {
        self.search.open();
        self.screen = Screen::SearchOverlay;
        self.is_typing = true;
        self.spellbook_browser.reset();
    }

    pub fn exit_typing_mode(&mut self) {
        self.is_typing = false;
    }

    pub fn clear_add_spell_form(&mut self) {
        self.add_spell.clear();
        self.screen = Screen::SearchOverlay;
        self.exit_typing_mode();
    }

    pub fn update_typing_state(&mut self) {
        match &self.screen {
            Screen::AddSpell => {
                self.is_typing = self.add_spell.is_typing();
            }
            Screen::SearchOverlay => {
                self.is_typing = true;
            }
            _ => {
                self.is_typing = false;
            }
        }
    }

    pub fn search_query(&self) -> &str {
        &self.search.query
    }

    pub fn search_query_mut(&mut self) -> &mut String {
        &mut self.search.query
    }

    pub fn filtered_indices(&self) -> &[usize] {
        &self.search.filtered_indices
    }

    pub fn filtered_indices_mut(&mut self) -> &mut Vec<usize> {
        &mut self.search.filtered_indices
    }

    pub fn search_results_state(&mut self) -> &mut ListState {
        &mut self.search.results_state
    }

    pub fn showing_spellbooks(&self) -> bool {
        self.search.showing_spellbooks
    }

    pub fn set_showing_spellbooks(&mut self, value: bool) {
        self.search.showing_spellbooks = value;
    }

    pub fn search_in_command_mode(&self) -> bool {
        self.search.in_command_mode
    }

    pub fn set_search_in_command_mode(&mut self, value: bool) {
        self.search.in_command_mode = value;
    }

    pub fn search_spellbook_index(&self) -> Option<usize> {
        self.spellbook_browser.index
    }

    pub fn set_search_spellbook_index(&mut self, value: Option<usize>) {
        self.spellbook_browser.index = value;
    }

    pub fn search_spellbook_scroll(&self) -> usize {
        self.spellbook_browser.scroll
    }

    pub fn set_search_spellbook_scroll(&mut self, value: usize) {
        self.spellbook_browser.scroll = value;
    }

    pub fn search_spines_per_row(&self) -> usize {
        self.spellbook_browser.spines_per_row
    }

    pub fn set_search_spines_per_row(&mut self, value: usize) {
        self.spellbook_browser.spines_per_row = value;
    }

    pub fn search_last_width(&self) -> u16 {
        self.spellbook_browser.last_width
    }

    pub fn set_search_last_width(&mut self, value: u16) {
        self.spellbook_browser.last_width = value;
    }

    pub fn search_last_height(&self) -> u16 {
        self.spellbook_browser.last_height
    }

    pub fn set_search_last_height(&mut self, value: u16) {
        self.spellbook_browser.last_height = value;
    }

    pub fn search_items_per_row(&self) -> usize {
        self.spellbook_browser.items_per_row
    }

    pub fn set_search_items_per_row(&mut self, value: usize) {
        self.spellbook_browser.items_per_row = value;
    }

    pub fn show_output_popup(&mut self, result: ExecutionResult) {
        self.previous_screen = Some(self.screen.clone());
        self.output_popup = Some(result);
        self.screen = Screen::OutputPopup;
    }

    pub fn hide_output_popup(&mut self) {
        if let Some(prev) = self.previous_screen.take() {
            self.screen = prev;
        } else {
            self.screen = Screen::SearchOverlay;
        }
        self.output_popup = None;
    }

    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn clear_redraw_flag(&mut self) -> bool {
        let needs = self.needs_redraw;
        self.needs_redraw = false;
        needs
    }
}
