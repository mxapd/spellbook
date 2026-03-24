use ratatui::widgets::ListState;

pub use crate::clipboard::ExecutionResult;
pub use crate::models::{FocusTarget, ViewMode};

pub mod add_spell;
pub mod add_spell_form;
pub mod add_spellbook_form;
pub mod confirm;
pub mod events;
pub mod footer;
pub mod help;
pub mod input;
pub mod jobs;
pub mod render;
pub mod search_overlay;
pub mod search_state;
pub mod spell_list;
pub mod spellbook_browser;
pub mod streaming_modal;

pub use add_spell_form::{AddSpellField, AddSpellForm};
pub use add_spellbook_form::{AddSpellbookField, AddSpellbookForm};
pub use confirm::ConfirmDialogState;
pub use events::filter_commands;
pub use events::handle_event;
pub use input::{InputPhase, InputPopupState};
pub use jobs::JobsPanelState;
pub use render::render;
pub use search_state::SearchState;
pub use spellbook_browser::SpellbookBrowserState;
pub use streaming_modal::StreamingModalState;

/// Application modes - represents the main view state
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum Mode {
    #[default]
    BrowseSpellbooks,
    BrowseSpells,
    AddSpell,
    EditSpell,
    AddSpellbook,
}

/// Overlays render on top of the current mode
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Overlay {
    OutputModal,
    ConfirmDialog,
    CommandPalette,
    Help,
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
    // Mode/overlay system
    pub mode: Mode,
    pub overlays: Vec<Overlay>,

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
    pub needs_redraw: bool,
    pub jobs_panel_state: JobsPanelState,
    
    // Loading state for long-running operations
    pub loading_message: Option<String>,
    pub loading_spinner: u8,
    pub confirm_dialog: Option<ConfirmDialogState>,
    pub input_popup: Option<InputPopupState>,
    pub focus: FocusTarget,
    pub jobs_sidebar_open: bool,

    // Streaming output modal state
    pub streaming_modal: StreamingModalState,
}

impl UiState {
    pub fn new(start_in_add_mode: bool) -> Self {
        let mode = if start_in_add_mode {
            Mode::AddSpell
        } else {
            Mode::BrowseSpellbooks
        };

        Self {
            mode,
            overlays: Vec::new(),
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
            needs_redraw: false,
            jobs_panel_state: JobsPanelState::default(),
            confirm_dialog: None,
            input_popup: None,
            focus: FocusTarget::Main,
            jobs_sidebar_open: false,

            streaming_modal: StreamingModalState::default(),
            
            loading_message: None,
            loading_spinner: 0,
        }
    }
    
    /// Show a loading message for long-running operations
    pub fn start_loading(&mut self, message: impl Into<String>) {
        self.loading_message = Some(message.into());
        self.loading_spinner = 0;
        self.request_redraw();
    }
    
    /// Clear the loading state
    pub fn stop_loading(&mut self) {
        self.loading_message = None;
        self.request_redraw();
    }
    
    /// Update the spinner animation (call periodically)
    pub fn tick_spinner(&mut self) {
        if self.loading_message.is_some() {
            self.loading_spinner = (self.loading_spinner + 1) % 4;
            self.request_redraw();
        }
    }
    
    /// Get the current spinner character
    pub fn spinner_char(&self) -> char {
        match self.loading_spinner {
            0 => '|',
            1 => '/',
            2 => '-',
            3 => '\\',
            _ => '|',
        }
    }
    
    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.loading_message.is_some()
    }
    
    // Mode/Overlay management
    pub fn push_overlay(&mut self, overlay: Overlay) {
        if !self.overlays.contains(&overlay) {
            self.overlays.push(overlay);
        }
    }
    
    pub fn pop_overlay(&mut self) {
        self.overlays.pop();
    }
    
    pub fn clear_overlays(&mut self) {
        self.overlays.clear();
    }
    
    pub fn has_overlay(&self, overlay: Overlay) -> bool {
        self.overlays.contains(&overlay)
    }
    
    pub fn top_overlay(&self) -> Option<Overlay> {
        self.overlays.last().copied()
    }
    
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    
    /// Check if any overlay is active
    pub fn has_any_overlay(&self) -> bool {
        !self.overlays.is_empty()
    }
    
    pub fn open_search(&mut self) {
        self.search.open();
        self.mode = Mode::BrowseSpellbooks;
        self.is_typing = true;
        self.spellbook_browser.reset();
    }

    pub fn exit_typing_mode(&mut self) {
        self.is_typing = false;
    }

    pub fn clear_add_spell_form(&mut self) {
        self.add_spell.clear();
        self.mode = Mode::BrowseSpellbooks;
        self.exit_typing_mode();
    }

    pub fn update_typing_state(&mut self) {
        match self.mode {
            Mode::AddSpell | Mode::EditSpell => {
                self.is_typing = self.add_spell.is_typing();
            }
            Mode::BrowseSpellbooks | Mode::BrowseSpells => {
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

    pub fn filtered_spellbook_indices(&self) -> &[usize] {
        &self.search.filtered_spellbook_indices
    }

    pub fn filtered_spellbook_indices_mut(&mut self) -> &mut Vec<usize> {
        &mut self.search.filtered_spellbook_indices
    }

    pub fn search_active(&self) -> bool {
        self.search.search_active
    }

    pub fn set_search_active(&mut self, value: bool) {
        self.search.search_active = value;
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
        self.output_popup = Some(result);
        self.push_overlay(Overlay::OutputModal);
    }

    pub fn hide_output_popup(&mut self) {
        self.pop_overlay();
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

    pub fn toggle_jobs_sidebar(&mut self) {
        self.jobs_sidebar_open = !self.jobs_sidebar_open;
        if self.jobs_sidebar_open {
            self.focus = FocusTarget::JobsSidebar;
            self.jobs_panel_state.selected_index = Some(0);
        } else {
            self.focus = FocusTarget::Main;
        }
    }

    pub fn cycle_focus(&mut self) {
        if !self.jobs_sidebar_open {
            return;
        }
        match self.focus {
            FocusTarget::Main => self.focus = FocusTarget::JobsSidebar,
            FocusTarget::JobsSidebar => self.focus = FocusTarget::Main,
        }
    }

    pub fn main_has_focus(&self) -> bool {
        self.focus == FocusTarget::Main
    }

    pub fn sidebar_has_focus(&self) -> bool {
        self.focus == FocusTarget::JobsSidebar
    }
    
    // Mode transition helpers
    pub fn enter_browse_spells(&mut self, spellbook_index: usize) {
        self.mode = Mode::BrowseSpells;
        self.selected_spellbook = Some(spellbook_index);
        self.spell_list_state.select(Some(0));
    }
    
    pub fn enter_browse_spellbooks(&mut self) {
        self.mode = Mode::BrowseSpellbooks;
        self.selected_spellbook = None;
    }
    
    pub fn enter_add_spell(&mut self) {
        self.mode = Mode::AddSpell;
        self.add_spell.clear();
        self.is_typing = true;
    }
    
    pub fn enter_edit_spell(&mut self, spellbook_index: usize, spell_index: usize) {
        self.mode = Mode::EditSpell;
        self.selected_spellbook = Some(spellbook_index);
        self.spell_list_state.select(Some(spell_index));
        self.is_typing = true;
    }
}
