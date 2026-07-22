pub use crate::clipboard::ExecutionResult;
pub use crate::models::{FocusTarget, ViewMode};

pub use add_spell_form::{AddSpellField, AddSpellForm};
pub use add_spellbook_form::AddSpellbookForm;
pub use confirm::ConfirmDialogState;
pub use feedback::{Feedback, FeedbackLevel, FlashAction};
pub use handle_key::execute_simple_mode;
pub use handle_key::filter_commands;
pub use input::InputPopupState;
pub use jobs::JobsPanelState;
pub use mode::{BrowseState, FormField, FormState, Mode, Overlay};
pub use quick_add_spell::QuickAddSpellState;
pub use render::render;
pub use spellbook_browser::SpellbookBrowserState;
pub use streaming_modal::StreamingModalState;
pub use uistate::UiState;

pub mod add_spell;
pub mod add_spell_form;
pub mod add_spellbook_form;
pub mod browse_spellbooks;
pub mod browse_spells;
pub mod confirm;
pub mod feedback;
pub mod form;
pub mod handle_key;
pub mod help;
pub mod input;
pub mod jobs;
pub mod mode;
pub mod quick_add_spell;
pub mod render;
pub mod search_overlay;
pub mod spell_list;
pub mod spellbook_browser;
pub mod streaming_modal;
pub mod uistate;

/// Default working directory captured from the environment.
/// Used as fallback for spells that don't set their own working_dir.
/// Returns None if CWD cannot be determined.
pub fn default_launch_dir() -> Option<String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok()
        .filter(|s| !s.is_empty())
}
