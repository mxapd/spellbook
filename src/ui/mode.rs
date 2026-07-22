/// Application modes — represents the main view state (Elm "Model")
/// Each variant contains its own state — no more parallel state machines
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
        spellbook_index: Option<usize>,
    },
    /// Like Searching but typing is paused — same fields, same rendering,
    /// but `is_searching()` returns false so bare keys act as actions.
    SearchPaused {
        query: String,
        filtered_indices: Vec<usize>,
        filtered_spellbook_indices: Vec<usize>,
        results_state: ratatui::widgets::ListState,
        spellbook_index: Option<usize>,
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
    Command,
    Description,
    Category,
    Tags,
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
