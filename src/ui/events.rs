use crate::state::State;
use crate::ui::{Screen, SearchContext, UiState};
use crossterm::event::KeyCode;

pub fn handle_event(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    if key == KeyCode::Char('q') || key == KeyCode::Esc {
        return true;
    }

    match &ui.screen {
        Screen::SpellbookList => handle_spellbook_list(key, state, ui),
        Screen::SpellList => handle_spell_list(key, state, ui),
        Screen::SearchOverlay { .. } => handle_search(key, state, ui),
    }
}

fn handle_spellbook_list(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Down | KeyCode::Char('k') => {
            let spellbook_count = state.codex.spellbooks.len();

            if spellbook_count > 0 {
                let next = ui
                    .spellbook_list_state
                    .selected()
                    .map(|s| (s + 1) % spellbook_count)
                    .unwrap_or(0);
                ui.spellbook_list_state.select(Some(next));
            }
            false
        }

        KeyCode::Up | KeyCode::Char('j') => {
            let spellbook_count = state.codex.spellbooks.len();

            if spellbook_count > 0 {
                let prev = ui
                    .spellbook_list_state
                    .selected()
                    .map(|s| (s - 1) % spellbook_count)
                    .unwrap_or(0);
                ui.spellbook_list_state.select(Some(prev));
            }
            false
        }

        KeyCode::Enter => {
            if let Some(selected) = ui.spellbook_list_state.selected() {
                ui.selected_spellbook = Some(selected);
                ui.screen = Screen::SpellList;
            }
            false
        }

        _ => false,
    }
}

fn handle_spell_list(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    false
}

fn handle_search(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    false
}

fn update_filtered(state: &State, ui: &mut UiState, ctx: &SearchContext) {}

fn copy_selected_spell(state: &State, ui: &UiState) {}
