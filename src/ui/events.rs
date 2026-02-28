use crate::state::State;
use crate::ui::{Screen, SearchContext, UiState};
use crossterm::event::KeyCode;

pub fn handle_event(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    match &ui.screen {
        Screen::SpellbookList => handle_spellbook_list(key, state, ui),
        Screen::SpellList => handle_spell_list(key, state, ui),
        Screen::SearchOverlay { .. } => handle_search(key, state, ui),
    }
}

fn handle_spellbook_list(key: KeyCode, state: &State, ui: &mut UiState) -> bool {}

fn handle_spell_list(key: KeyCode, state: &State, ui: &mut UiState) -> bool {}

fn handle_search(key: KeyCode, state: &State, ui: &mut UiState) -> bool {}

fn update_filtered(state: &State, ui: &mut UiState, ctx: &SearchContext) {}

fn copy_selected_spell(state: &State, ui: &UiState) {}
