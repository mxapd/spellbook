use crate::state::State;
use crate::ui::{add_spell, search_overlay, spell_list, spellbook_list, Screen, UiState};
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
            search_overlay::render(frame, state, ui);
        }
        Screen::AddSpell => {
            add_spell::render(frame, state, ui);
        }
    }
}
