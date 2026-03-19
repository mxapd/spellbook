use crate::clipboard;
use crate::log_debug;
use crate::persistence::Archivist;
use crate::state::State;
use crate::ui::{AddSpellField, Screen, SearchContext, UiState};
use crossterm::event::{KeyCode, KeyModifiers};

/// Main event handler - routes key events to the appropriate screen handler.
pub fn handle_event(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Handle quit keys globally - but only Esc when we're at the root screen
    if key == KeyCode::Char('q') {
        return true;
    }

    // Handle theme cycling with 't' - disabled while typing
    if key == KeyCode::Char('t') && !ui.is_typing {
        state.cycle_theme();
        return false;
    }

    match &ui.screen {
        Screen::SpellbookList => {
            log_debug!("Screen: SpellbookList");
            handle_spellbook_list(key, state, ui)
        }
        Screen::SpellList => {
            log_debug!("Screen: SpellList");
            handle_spell_list(key, state, ui)
        }
        Screen::SearchOverlay { .. } => {
            log_debug!("Screen: SearchOverlay");
            handle_search(key, state, ui)
        }
        Screen::AddSpell => {
            log_debug!("Screen: AddSpell");
            handle_add_spell(key, state, ui, modifiers)
        }
    }
}

/// Handles key events on the spellbook list (home screen).
fn handle_spellbook_list(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    match key {
        KeyCode::Esc => true,
        KeyCode::Down | KeyCode::Char('k') => {
            let spellbook_count = state.codex.spellbooks.len();

            if spellbook_count > 0 {
                let current = ui.spellbook_list_state.selected().unwrap_or(0);
                let next = if current >= spellbook_count - 1 {
                    0
                } else {
                    current + 1
                };
                ui.spellbook_list_state.select(Some(next));
            }
            false
        }

        KeyCode::Up | KeyCode::Char('j') => {
            let spellbook_count = state.codex.spellbooks.len();

            if spellbook_count > 0 {
                let current = ui.spellbook_list_state.selected().unwrap_or(0);
                let prev = if current == 0 {
                    spellbook_count - 1
                } else {
                    current - 1
                };
                ui.spellbook_list_state.select(Some(prev));
            }
            false
        }

        KeyCode::Enter => {
            if let Some(selected) = ui.spellbook_list_state.selected() {
                ui.selected_spellbook = Some(selected);
                ui.screen = Screen::SpellList;
                ui.spell_list_state.select(Some(0));
            }
            false
        }

        // Open search overlay
        KeyCode::Char('/') => {
            ui.open_search(SearchContext::SpellbookList);
            false
        }

        _ => false,
    }
}

/// Handles key events on the spell list (inside a spellbook).
fn handle_spell_list(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    let spellbook_index = match ui.selected_spellbook {
        Some(index) => index,
        None => return false,
    };
    let spellbook = &state.codex.spellbooks[spellbook_index];
    let spell_count = spellbook.spell_ids.len();

    if spell_count == 0 {
        return false;
    }

    match key {
        KeyCode::Esc => {
            ui.screen = Screen::SpellbookList;
            ui.spell_list_state.select(None);
            false
        }
        KeyCode::Down | KeyCode::Char('k') => {
            let current = ui.spell_list_state.selected().unwrap_or(0);
            let next = if current >= spell_count - 1 {
                0
            } else {
                current + 1
            };
            ui.spell_list_state.select(Some(next));
            false
        }
        KeyCode::Up | KeyCode::Char('j') => {
            let current = ui.spell_list_state.selected().unwrap_or(0);
            let prev = if current == 0 {
                spell_count - 1
            } else {
                current - 1
            };
            ui.spell_list_state.select(Some(prev));
            false
        }
        // Copy selected spell to clipboard (standardized to Enter)
        KeyCode::Enter => {
            copy_selected_spell(state, ui);
            false
        }

        // Open search overlay
        KeyCode::Char('/') => {
            ui.open_search(SearchContext::SpellList);
            false
        }

        _ => false,
    }
}

/// Handles key events in the search overlay.
fn handle_search(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    // Close search on Escape - return to the screen we came from
    if key == KeyCode::Esc {
        ui.screen = match ui.search_return_to {
            SearchContext::SpellbookList => Screen::SpellbookList,
            SearchContext::SpellList => Screen::SpellList,
        };
        ui.exit_typing_mode();
        return false;
    }

    // Copy selected search result to clipboard
    if key == KeyCode::Enter {
        copy_search_result(state, ui);
        return false;
    }

    // Navigate search results
    if key == KeyCode::Down || key == KeyCode::Char('k') {
        let count = ui.filtered_indices.len();
        if count > 0 {
            let current = ui.search_list_state.selected().unwrap_or(0);
            let next = if current >= count - 1 { 0 } else { current + 1 };
            ui.search_list_state.select(Some(next));
        }
        return false;
    }

    if key == KeyCode::Up || key == KeyCode::Char('j') {
        let count = ui.filtered_indices.len();
        if count > 0 {
            let current = ui.search_list_state.selected().unwrap_or(0);
            let prev = if current == 0 { count - 1 } else { current - 1 };
            ui.search_list_state.select(Some(prev));
        }
        return false;
    }

    // Handle character input for search query (ignore '/' since it's the search key)
    if let KeyCode::Char(c) = key {
        if c != '/' {
            ui.search_query.push(c);
            update_search_filter(state, ui);
        }
        return false;
    }

    // Handle backspace
    if key == KeyCode::Backspace {
        ui.search_query.pop();
        update_search_filter(state, ui);
        return false;
    }

    false
}

/// Filters all spells by the current search query.
/// Searches across spell name, lore, and glyphs (case-insensitive).
fn update_search_filter(state: &State, ui: &mut UiState) {
    let query = ui.search_query.to_lowercase();

    if query.is_empty() {
        ui.filtered_indices.clear();
        ui.search_list_state.select(None);
        return;
    }

    // Filter spells that match the query in name, lore, or glyphs
    ui.filtered_indices = state
        .codex
        .spells
        .iter()
        .enumerate()
        .filter(|(_, spell)| {
            spell.name.to_lowercase().contains(&query)
                || spell.lore.to_lowercase().contains(&query)
                || spell
                    .glyphs
                    .iter()
                    .any(|g| g.to_lowercase().contains(&query))
        })
        .map(|(idx, _)| idx)
        .collect();

    // Select first result if we have matches
    if !ui.filtered_indices.is_empty() {
        ui.search_list_state.select(Some(0));
    } else {
        ui.search_list_state.select(None);
    }
}

/// Copies the selected spell from a spellbook to the clipboard.
fn copy_selected_spell(state: &State, ui: &mut UiState) {
    let spellbook_index = match ui.selected_spellbook {
        Some(index) => index,
        None => return,
    };
    let spell_index = match ui.spell_list_state.selected() {
        Some(index) => index,
        None => return,
    };

    let spellbook = &state.codex.spellbooks[spellbook_index];
    let spell_id = match spellbook.spell_ids.get(spell_index) {
        Some(id) => id,
        None => return,
    };

    let spell = match state.codex.spells.iter().find(|s| s.id == *spell_id) {
        Some(s) => s,
        None => return,
    };

    let _ = clipboard::copy_to_clipboard(&spell.incantation);
}

/// Copies the selected search result to the clipboard.
fn copy_search_result(state: &State, ui: &mut UiState) {
    let selected_idx = match ui.search_list_state.selected() {
        Some(i) => i,
        None => return,
    };

    let spell_idx = match ui.filtered_indices.get(selected_idx) {
        Some(&i) => i,
        None => return,
    };

    let spell = match state.codex.spells.get(spell_idx) {
        Some(s) => s,
        None => return,
    };

    let _ = clipboard::copy_to_clipboard(&spell.incantation);
}

/// Saves the current spell and returns to the spellbook list.
fn save_spell(state: &State, ui: &mut UiState) {
    if !ui.add_spell_name.is_empty() && !ui.add_spell_command.is_empty() {
        let tags: Vec<String> = ui
            .add_spell_tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let spell = crate::models::Spell {
            id: 0,
            name: ui.add_spell_name.clone(),
            incantation: ui.add_spell_command.clone(),
            lore: ui.add_spell_lore.clone(),
            school: ui.add_spell_school.clone(),
            glyphs: tags,
        };

        let spellbook_name = if ui.add_spell_skip_spellbook {
            None
        } else {
            ui.add_spell_spellbook
                .and_then(|i| state.codex.spellbooks.get(i))
                .map(|b| b.name.clone())
        };

        if let Err(e) = Archivist::append_spell("codex.toml", &spell, spellbook_name.as_deref()) {
            eprintln!("Error saving spell: {}", e);
        }
    }

    ui.add_spell_name.clear();
    ui.add_spell_command.clear();
    ui.add_spell_lore.clear();
    ui.add_spell_school.clear();
    ui.add_spell_tags.clear();
    ui.add_spell_spellbook = None;
    ui.add_spell_skip_spellbook = false;
    ui.add_spell_dropdown_open = false;
    ui.screen = Screen::SpellbookList;
    ui.exit_typing_mode();
}

/// Handles key events in the Add Spell screen.
fn handle_add_spell(
    key: KeyCode,
    state: &State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    if key == KeyCode::Char('s') && modifiers.contains(KeyModifiers::CONTROL) {
        save_spell(state, ui);
        return false;
    }

    match key {
        KeyCode::Esc => {
            if ui.add_spell_field == AddSpellField::Spellbook && ui.add_spell_dropdown_open {
                ui.add_spell_dropdown_open = false;
            } else {
                ui.screen = Screen::SpellbookList;
                ui.exit_typing_mode();
            }
            false
        }
        KeyCode::Tab => {
            ui.add_spell_dropdown_open = false;
            ui.add_spell_field = match ui.add_spell_field {
                AddSpellField::Name => AddSpellField::Command,
                AddSpellField::Command => AddSpellField::Lore,
                AddSpellField::Lore => AddSpellField::School,
                AddSpellField::School => AddSpellField::Tags,
                AddSpellField::Tags => AddSpellField::Spellbook,
                AddSpellField::Spellbook => AddSpellField::Name,
            };
            ui.update_typing_state();
            false
        }
        KeyCode::Up => {
            if ui.add_spell_field == AddSpellField::Spellbook {
                if ui.add_spell_dropdown_open {
                    if ui.add_spell_dropdown_index == 0 {
                        ui.add_spell_dropdown_open = false;
                    } else {
                        let options_count = state.codex.spellbooks.len() + 1;
                        if options_count > 0 {
                            ui.add_spell_dropdown_index -= 1;
                        }
                    }
                } else {
                    ui.add_spell_field = AddSpellField::Tags;
                    ui.update_typing_state();
                }
            } else {
                ui.add_spell_field = match ui.add_spell_field {
                    AddSpellField::Command => AddSpellField::Name,
                    AddSpellField::Lore => AddSpellField::Command,
                    AddSpellField::School => AddSpellField::Lore,
                    AddSpellField::Tags => AddSpellField::School,
                    AddSpellField::Spellbook => AddSpellField::Tags,
                    AddSpellField::Name => AddSpellField::Spellbook,
                };
                ui.update_typing_state();
            }
            false
        }
        KeyCode::Down => {
            if ui.add_spell_field == AddSpellField::Spellbook {
                if ui.add_spell_dropdown_open {
                    let options_count = state.codex.spellbooks.len() + 1;
                    if options_count > 0 {
                        ui.add_spell_dropdown_index =
                            (ui.add_spell_dropdown_index + 1) % options_count;
                    }
                } else {
                    ui.add_spell_dropdown_open = true;
                }
            } else {
                ui.add_spell_field = match ui.add_spell_field {
                    AddSpellField::Name => AddSpellField::Command,
                    AddSpellField::Command => AddSpellField::Lore,
                    AddSpellField::Lore => AddSpellField::School,
                    AddSpellField::School => AddSpellField::Tags,
                    AddSpellField::Tags => AddSpellField::Spellbook,
                    _ => ui.add_spell_field,
                };
                ui.update_typing_state();
            }
            false
        }
        KeyCode::Enter => match ui.add_spell_field {
            AddSpellField::Spellbook => {
                if ui.add_spell_dropdown_open {
                    if ui.add_spell_dropdown_index >= state.codex.spellbooks.len() {
                        ui.add_spell_skip_spellbook = true;
                        ui.add_spell_spellbook = None;
                    } else {
                        ui.add_spell_skip_spellbook = false;
                        ui.add_spell_spellbook = Some(ui.add_spell_dropdown_index);
                    }
                    ui.add_spell_dropdown_open = false;
                } else {
                    ui.add_spell_dropdown_open = true;
                }
                false
            }
            _ => {
                ui.add_spell_field = match ui.add_spell_field {
                    AddSpellField::Name => AddSpellField::Command,
                    AddSpellField::Command => AddSpellField::Lore,
                    AddSpellField::Lore => AddSpellField::School,
                    AddSpellField::School => AddSpellField::Tags,
                    AddSpellField::Tags => AddSpellField::Spellbook,
                    _ => ui.add_spell_field,
                };
                ui.update_typing_state();
                false
            }
        },
        KeyCode::Backspace => {
            match ui.add_spell_field {
                AddSpellField::Name => {
                    ui.add_spell_name.pop();
                }
                AddSpellField::Command => {
                    ui.add_spell_command.pop();
                }
                AddSpellField::Lore => {
                    ui.add_spell_lore.pop();
                }
                AddSpellField::School => {
                    ui.add_spell_school.pop();
                }
                AddSpellField::Tags => {
                    ui.add_spell_tags.pop();
                }
                _ => {}
            }
            false
        }
        KeyCode::Char(c) => {
            match ui.add_spell_field {
                AddSpellField::Name => {
                    ui.add_spell_name.push(c);
                }
                AddSpellField::Command => {
                    ui.add_spell_command.push(c);
                }
                AddSpellField::Lore => {
                    ui.add_spell_lore.push(c);
                }
                AddSpellField::School => {
                    ui.add_spell_school.push(c);
                }
                AddSpellField::Tags => {
                    ui.add_spell_tags.push(c);
                }
                _ => {}
            }
            false
        }
        _ => false,
    }
}
