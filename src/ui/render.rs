use crate::state::State;
use crate::ui::{Screen, UiState, search_overlay, spell_list, spellbook_list};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    match &ui.screen {
        Screen::SpellbookList => {
            spellbook_list::render(frame, state, ui);
        }
        Screen::SpellList => {
            spell_list::render(frame, state, ui);
        }
        Screen::SearchOverlay { .. } => {
            search_overlay::render(frame, ui);
        }
    }
}

