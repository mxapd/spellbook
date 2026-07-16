use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

pub use crate::clipboard::ExecutionResult;
pub use crate::models::{FocusTarget, ViewMode};

pub mod add_spell;
pub mod add_spell_form;
pub mod add_spellbook_form;
pub mod browse_spellbooks;
pub mod browse_spells;
pub mod confirm;
pub mod events;

pub mod form;
pub mod help;
pub mod input;
pub mod jobs;
pub mod quick_add_spell;
pub mod render;
pub mod search_overlay;
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
pub use quick_add_spell::QuickAddSpellState;
pub use render::render;
pub use spellbook_browser::SpellbookBrowserState;
pub use streaming_modal::StreamingModalState;

/// Application modes - represents the main view state (Elm "Model")
/// Each variant contains its own state - no more parallel state machines
#[derive(PartialEq, Clone, Debug)]
pub enum Mode {
    BrowseSpellbooks(BrowseState),
    BrowseSpells(BrowseState),
    AddSpell(FormState),
    EditSpell(FormState),
    AddSpellbook(FormState),
}

impl Default for Mode {
    fn default() -> Self {
        Mode::BrowseSpellbooks(BrowseState::default())
    }
}

/// State for browse modes (Elm Model composition)
/// This is the SINGLE SOURCE OF TRUTH for browse-mode state
#[derive(PartialEq, Clone, Debug)]
pub enum BrowseState {
    Idle {
        filtered_spellbook_indices: Vec<usize>,
    },
    Searching {
        query: String,
        filtered_indices: Vec<usize>,
        filtered_spellbook_indices: Vec<usize>,
        results_state: ratatui::widgets::ListState,
    },
    Viewing {
        spellbook_index: usize,
        spell_list_state: ratatui::widgets::ListState,
    },
}

impl Default for BrowseState {
    fn default() -> Self {
        BrowseState::Idle {
            filtered_spellbook_indices: Vec::new(),
        }
    }
}

/// State for form modes (Elm Model composition)
#[derive(PartialEq, Clone, Debug, Default)]
pub enum FormState {
    #[default]
    Idle,
    Editing(FormField), // which field has focus
}

#[derive(PartialEq, Clone, Debug)]
pub enum FormField {
    Name,
    Incantation,
    Lore,
    School,
    Glyphs,
    WorkingDir,
    RunMode,
    Confirm,
}

/// Overlays render on top of the current mode
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Overlay {
    OutputModal,
    ConfirmDialog,
    CommandPalette,
    Help,
    InputPopup,
    SpellDetails,
    QuickAddSpell,
}

/// Visual severity level for footer feedback messages.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FeedbackLevel {
    Success,
    Error,
    Info,
}

/// A timed feedback message shown in the footer/status bar.
#[derive(PartialEq, Clone, Debug)]
pub struct Feedback {
    pub message: String,
    pub level: FeedbackLevel,
    pub until: Instant,
}

impl Feedback {
    pub fn paragraph<'a>(
        &self,
        theme: &crate::models::RatatuiColors,
    ) -> ratatui::widgets::Paragraph<'a> {
        let color = match self.level {
            FeedbackLevel::Success => ratatui::style::Color::Green,
            FeedbackLevel::Error => ratatui::style::Color::Red,
            FeedbackLevel::Info => theme.accent,
        };
        let single_line = self
            .message
            .lines()
            .next()
            .unwrap_or(&self.message)
            .to_string();
        ratatui::widgets::Paragraph::new(single_line)
            .style(ratatui::style::Style::new().fg(color).bg(theme.bg))
            .alignment(ratatui::layout::Alignment::Center)
    }
}

/// A brief visual flash on a specific action target.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FlashAction {
    Spell {
        spellbook_index: usize,
        spell_index: usize,
    },
    Spellbook {
        spellbook_index: usize,
    },
    SearchResult {
        index: usize,
    },
}

pub struct UiState {
    // Mode/overlay system
    pub mode: Mode,
    pub overlays: Vec<Overlay>,

    pub spell_list_state: ListState,
    pub feedback: Option<Feedback>,
    pub flash_action: Option<(FlashAction, Instant)>,
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

    // Quick-add spell overlay state
    pub quick_add_spell: Option<QuickAddSpellState>,

    // Streaming output modal state
    pub streaming_modal: StreamingModalState,

    // Spell details popup state
    pub spell_details_spell_id: Option<String>,

    // Search cursor state for blinking effect
    pub search_cursor_visible: bool,
    pub search_cursor_tick: u64,

    // Quit flag
    pub should_quit: bool,
}

impl UiState {
    pub fn new(start_in_add_mode: bool) -> Self {
        let mode = if start_in_add_mode {
            Mode::AddSpell(FormState::default())
        } else {
            Mode::BrowseSpellbooks(BrowseState::default())
        };

        Self {
            mode,
            overlays: Vec::new(),
            spell_list_state: ListState::default(),
            feedback: None,
            flash_action: None,
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

            quick_add_spell: None,

            streaming_modal: StreamingModalState::default(),
            spell_details_spell_id: None,

            loading_message: None,
            loading_spinner: 0,

            search_cursor_visible: true,
            search_cursor_tick: 0,

            should_quit: false,
        }
    }

    /// Show spell details popup
    pub fn show_spell_details(&mut self, spell_id: String) {
        self.spell_details_spell_id = Some(spell_id);
        self.push_overlay(Overlay::SpellDetails);
    }

    /// Hide spell details popup
    pub fn hide_spell_details(&mut self) {
        self.pop_overlay();
        self.spell_details_spell_id = None;
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

    /// Show a timed feedback message in the footer/status bar.
    /// Default duration is 1.5 seconds.
    pub fn show_feedback(
        &mut self,
        message: impl Into<String>,
        level: FeedbackLevel,
        duration: Option<Duration>,
    ) {
        self.feedback = Some(Feedback {
            message: message.into(),
            level,
            until: Instant::now() + duration.unwrap_or(Duration::from_millis(1500)),
        });
        self.request_redraw();
    }

    /// Show a success feedback message (default 1.5s).
    pub fn show_success(&mut self, message: impl Into<String>) {
        self.show_feedback(message, FeedbackLevel::Success, None);
    }

    /// Show an error feedback message (default 1.5s).
    pub fn show_error(&mut self, message: impl Into<String>) {
        self.show_feedback(message, FeedbackLevel::Error, None);
    }

    /// Show an info feedback message (default 1.5s).
    pub fn show_info(&mut self, message: impl Into<String>) {
        self.show_feedback(message, FeedbackLevel::Info, None);
    }

    /// Age out expired feedback and flash actions. Call on every tick.
    pub fn tick_feedback(&mut self) {
        let now = Instant::now();
        let mut changed = false;
        if self.feedback.as_ref().is_some_and(|f| f.until <= now) {
            self.feedback = None;
            changed = true;
        }
        if self
            .flash_action
            .as_ref()
            .is_some_and(|(_, until)| *until <= now)
        {
            self.flash_action = None;
            changed = true;
        }
        if changed {
            self.request_redraw();
        }
    }

    /// Flash a specific action target for the given duration (default 400ms).
    pub fn flash(&mut self, action: FlashAction, duration: Option<Duration>) {
        self.flash_action = Some((
            action,
            Instant::now() + duration.unwrap_or(Duration::from_millis(400)),
        ));
        self.request_redraw();
    }

    /// Tick the search cursor (call periodically, e.g., every 50ms)
    pub fn tick_search_cursor(&mut self) {
        self.search_cursor_tick += 1;
        // Medium blink: toggle every 5 ticks (~250ms at 50ms tick rate)
        if self.search_cursor_tick % 5 == 0 {
            self.search_cursor_visible = !self.search_cursor_visible;
            self.request_redraw();
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
        // Transition to searching state (Elm-style: query lives in Mode)
        self.start_search();
        // Only reset browser if currently in BrowseSpellbooks
        if matches!(self.mode, Mode::BrowseSpellbooks(_)) {
            self.spellbook_browser.reset();
        }
    }

    pub fn exit_typing_mode(&mut self) {
        // Transition out of searching state (Elm-style)
        self.stop_search();
    }

    pub fn clear_add_spell_form(&mut self) {
        self.add_spell.clear();
        self.mode = Mode::BrowseSpellbooks(BrowseState::default());
        self.exit_typing_mode();
    }

    /// Check if user is in typing mode (derived from Mode)
    pub fn is_typing(&self) -> bool {
        match &self.mode {
            Mode::AddSpell(_) | Mode::EditSpell(_) => self.add_spell.is_typing(),
            Mode::BrowseSpellbooks(BrowseState::Searching { .. }) => true,
            Mode::BrowseSpells(BrowseState::Searching { .. }) => true,
            _ => false,
        }
    }

    /// Get search query from Mode (Elm-style: query lives in BrowseState)
    pub fn search_query(&self) -> String {
        match &self.mode {
            Mode::BrowseSpellbooks(state) | Mode::BrowseSpells(state) => match state {
                BrowseState::Searching { query, .. } => query.clone(),
                BrowseState::Idle { .. } | BrowseState::Viewing { .. } => String::new(),
            },
            _ => String::new(),
        }
    }

    /// Get mutable reference to search query from Mode (Elm-style)
    /// Only works when in Searching state
    pub fn search_query_mut(&mut self) -> Option<&mut String> {
        match &mut self.mode {
            Mode::BrowseSpellbooks(BrowseState::Searching { query, .. })
            | Mode::BrowseSpells(BrowseState::Searching { query, .. }) => Some(query),
            _ => None,
        }
    }

    /// Start searching - transition from Idle to Searching state
    pub fn start_search(&mut self) {
        match &mut self.mode {
            Mode::BrowseSpellbooks(state) => {
                *state = BrowseState::Searching {
                    query: String::new(),
                    filtered_indices: Vec::new(),
                    filtered_spellbook_indices: Vec::new(),
                    results_state: ratatui::widgets::ListState::default(),
                };
            }
            Mode::BrowseSpells(state) => {
                *state = BrowseState::Searching {
                    query: String::new(),
                    filtered_indices: Vec::new(),
                    filtered_spellbook_indices: Vec::new(),
                    results_state: ratatui::widgets::ListState::default(),
                };
            }
            _ => {}
        }
    }

    /// Stop searching - transition from Searching to Idle state
    pub fn stop_search(&mut self) {
        match &mut self.mode {
            Mode::BrowseSpellbooks(state) => {
                *state = BrowseState::Idle {
                    filtered_spellbook_indices: Vec::new(),
                };
            }
            Mode::BrowseSpells(state) => {
                *state = BrowseState::Idle {
                    filtered_spellbook_indices: Vec::new(),
                };
            }
            _ => {}
        }
    }

    /// Check if in searching state
    pub fn is_searching(&self) -> bool {
        matches!(
            self.mode,
            Mode::BrowseSpellbooks(BrowseState::Searching { .. })
                | Mode::BrowseSpells(BrowseState::Searching { .. })
        )
    }

    /// Get filtered spell indices (from BrowseState)
    pub fn filtered_indices(&self) -> &[usize] {
        match &self.mode {
            Mode::BrowseSpellbooks(state) | Mode::BrowseSpells(state) => match state {
                BrowseState::Searching {
                    filtered_indices, ..
                } => filtered_indices,
                BrowseState::Idle { .. } | BrowseState::Viewing { .. } => &[],
            },
            _ => &[],
        }
    }

    /// Get mutable filtered spell indices (from BrowseState)
    pub fn filtered_indices_mut(&mut self) -> &mut Vec<usize> {
        match &mut self.mode {
            Mode::BrowseSpellbooks(BrowseState::Searching {
                filtered_indices, ..
            })
            | Mode::BrowseSpells(BrowseState::Searching {
                filtered_indices, ..
            }) => filtered_indices,
            _ => panic!("Cannot get mutable filtered_indices when not in Searching state"),
        }
    }

    /// Get filtered spellbook indices (from BrowseState)
    pub fn filtered_spellbook_indices(&self) -> &[usize] {
        match &self.mode {
            Mode::BrowseSpellbooks(state) | Mode::BrowseSpells(state) => match state {
                BrowseState::Idle {
                    filtered_spellbook_indices,
                } => filtered_spellbook_indices,
                BrowseState::Searching {
                    filtered_spellbook_indices,
                    ..
                } => filtered_spellbook_indices,
                BrowseState::Viewing { .. } => &[],
            },
            _ => &[],
        }
    }

    /// Get mutable filtered spellbook indices (from BrowseState)
    pub fn filtered_spellbook_indices_mut(&mut self) -> &mut Vec<usize> {
        match &mut self.mode {
            Mode::BrowseSpellbooks(BrowseState::Idle {
                filtered_spellbook_indices,
            })
            | Mode::BrowseSpells(BrowseState::Idle {
                filtered_spellbook_indices,
            }) => filtered_spellbook_indices,
            Mode::BrowseSpellbooks(BrowseState::Searching {
                filtered_spellbook_indices,
                ..
            })
            | Mode::BrowseSpells(BrowseState::Searching {
                filtered_spellbook_indices,
                ..
            }) => filtered_spellbook_indices,
            _ => panic!("Cannot get mutable filtered_spellbook_indices when not in browse mode"),
        }
    }

    /// Check if search is active (derived from BrowseState)
    pub fn search_active(&self) -> bool {
        self.is_searching()
    }

    /// Get search results state (from BrowseState)
    pub fn search_results_state(&mut self) -> &mut ListState {
        match &mut self.mode {
            Mode::BrowseSpellbooks(BrowseState::Searching { results_state, .. })
            | Mode::BrowseSpells(BrowseState::Searching { results_state, .. }) => results_state,
            _ => panic!("Cannot get results_state when not in Searching state"),
        }
    }

    /// Check if showing spellbooks list (derived from Mode)
    pub fn showing_spellbooks(&self) -> bool {
        matches!(self.mode, Mode::BrowseSpellbooks(_))
    }

    /// Get selected spellbook index (from BrowseState::Viewing)
    pub fn selected_spellbook(&self) -> Option<usize> {
        match &self.mode {
            Mode::BrowseSpells(BrowseState::Viewing {
                spellbook_index, ..
            }) => Some(*spellbook_index),
            _ => None,
        }
    }

    /// Set selected spellbook - transition to Viewing state
    pub fn set_selected_spellbook(&mut self, index: usize) {
        self.mode = Mode::BrowseSpells(BrowseState::Viewing {
            spellbook_index: index,
            spell_list_state: ratatui::widgets::ListState::default(),
        });
    }

    /// Clear selected spellbook - transition to Idle state
    pub fn clear_selected_spellbook(&mut self) {
        self.mode = Mode::BrowseSpells(BrowseState::Idle {
            filtered_spellbook_indices: Vec::new(),
        });
    }

    /// Get spell list state (from BrowseState::Viewing)
    pub fn spell_list_state(&self) -> Option<&ratatui::widgets::ListState> {
        match &self.mode {
            Mode::BrowseSpells(BrowseState::Viewing {
                spell_list_state, ..
            }) => Some(spell_list_state),
            _ => None,
        }
    }

    /// Get mutable spell list state (from BrowseState::Viewing)
    pub fn spell_list_state_mut(&mut self) -> Option<&mut ratatui::widgets::ListState> {
        match &mut self.mode {
            Mode::BrowseSpells(BrowseState::Viewing {
                spell_list_state, ..
            }) => Some(spell_list_state),
            _ => None,
        }
    }

    pub fn set_showing_spellbooks(&mut self, value: bool) {
        // This is now derived - we change mode instead
        if value {
            self.mode = Mode::BrowseSpellbooks(BrowseState::default());
        } else if let Some(idx) = self.selected_spellbook() {
            self.set_selected_spellbook(idx);
        }
    }

    /// Check if in command mode (query starts with ':')
    pub fn search_in_command_mode(&self) -> bool {
        self.search_query().starts_with(':')
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

    pub fn open_jobs_sidebar(&mut self) {
        if !self.jobs_sidebar_open {
            self.jobs_sidebar_open = true;
            // Keep focus on Main - user can Tab to sidebar if needed
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
        self.mode = Mode::BrowseSpells(BrowseState::Viewing {
            spellbook_index,
            spell_list_state: ratatui::widgets::ListState::default(),
        });
        self.spell_list_state.select(Some(0));
    }

    pub fn enter_browse_spellbooks(&mut self) {
        self.mode = Mode::BrowseSpellbooks(BrowseState::default());
    }

    pub fn enter_add_spell(&mut self) {
        self.mode = Mode::AddSpell(FormState::default());
        self.add_spell.clear();
    }

    pub fn enter_edit_spell(&mut self, _spellbook_index: usize, spell_index: usize) {
        self.mode = Mode::EditSpell(FormState::default());
        self.spell_list_state.select(Some(spell_index));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_feedback_default_is_none() {
        let ui = UiState::new(false);
        assert!(ui.feedback.is_none());
        assert!(ui.flash_action.is_none());
    }

    #[test]
    fn test_show_success_sets_feedback() {
        let mut ui = UiState::new(false);
        ui.show_success("copied");
        let fb = ui.feedback.as_ref().expect("feedback should be set");
        assert_eq!(fb.message, "copied");
        assert_eq!(fb.level, FeedbackLevel::Success);
        assert!(fb.until > Instant::now());
    }

    #[test]
    fn test_show_error_sets_feedback() {
        let mut ui = UiState::new(false);
        ui.show_error("failed");
        let fb = ui.feedback.as_ref().expect("feedback should be set");
        assert_eq!(fb.message, "failed");
        assert_eq!(fb.level, FeedbackLevel::Error);
    }

    #[test]
    fn test_show_info_sets_feedback() {
        let mut ui = UiState::new(false);
        ui.show_info("hint");
        let fb = ui.feedback.as_ref().expect("feedback should be set");
        assert_eq!(fb.message, "hint");
        assert_eq!(fb.level, FeedbackLevel::Info);
    }

    #[test]
    fn test_show_feedback_custom_duration() {
        let mut ui = UiState::new(false);
        ui.show_feedback(
            "short",
            FeedbackLevel::Info,
            Some(Duration::from_millis(10)),
        );
        let fb = ui.feedback.as_ref().expect("feedback should be set");
        assert!(fb.until <= Instant::now() + Duration::from_millis(20));
        assert!(fb.until > Instant::now());
    }

    #[test]
    fn test_tick_feedback_clears_expired() {
        let mut ui = UiState::new(false);
        ui.show_feedback(
            "expired",
            FeedbackLevel::Info,
            Some(Duration::from_millis(1)),
        );
        thread::sleep(Duration::from_millis(5));
        ui.tick_feedback();
        assert!(ui.feedback.is_none());
    }

    #[test]
    fn test_tick_feedback_keeps_active() {
        let mut ui = UiState::new(false);
        ui.show_success("active");
        ui.tick_feedback();
        assert!(ui.feedback.is_some());
    }

    #[test]
    fn test_flash_sets_flash_action() {
        let mut ui = UiState::new(false);
        ui.flash(
            FlashAction::Spell {
                spellbook_index: 0,
                spell_index: 2,
            },
            None,
        );
        let (action, until) = ui.flash_action.as_ref().expect("flash should be set");
        assert_eq!(
            *action,
            FlashAction::Spell {
                spellbook_index: 0,
                spell_index: 2
            }
        );
        assert!(*until > Instant::now());
    }

    #[test]
    fn test_tick_feedback_clears_expired_flash() {
        let mut ui = UiState::new(false);
        ui.flash(
            FlashAction::Spellbook { spellbook_index: 0 },
            Some(Duration::from_millis(1)),
        );
        thread::sleep(Duration::from_millis(5));
        ui.tick_feedback();
        assert!(ui.flash_action.is_none());
    }
}
