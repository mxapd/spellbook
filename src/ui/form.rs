//! Form handler for AddSpell, EditSpell, and AddSpellbook modes
//!
//! This module contains shared form handling logic.

use crate::archivist::Archivist;
use crate::models::{RunMode, Spell};
use crate::state::State;
use crate::ui::add_spellbook_form::AddSpellbookField;
use crate::ui::{AddSpellField, BrowseState, Mode, UiState};
use crate::{log_error, log_info};
use crossterm::event::{KeyCode, KeyModifiers};

/// Handle key events for AddSpell and EditSpell modes
pub fn handle_add_spell(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Ctrl+S to save
    if key == KeyCode::Char('s') && modifiers.contains(KeyModifiers::CONTROL) {
        save_spell(state, ui);
        return false;
    }

    match key {
        KeyCode::Esc => {
            if ui.add_spell.field == AddSpellField::Spellbook && ui.add_spell.dropdown_open {
                ui.add_spell.dropdown_open = false;
            } else if ui.add_spell.has_unsaved {
                ui.add_spell.message = Some((
                    "Unsaved changes - press Esc again to discard".to_string(),
                    true,
                ));
                ui.add_spell.has_unsaved = false;
            } else {
                ui.clear_add_spell_form();
            }
            false
        }

        KeyCode::Tab => {
            ui.add_spell.dropdown_open = false;
            ui.add_spell.field = next_add_spell_field(ui.add_spell.field);
            false
        }

        KeyCode::BackTab => {
            ui.add_spell.dropdown_open = false;
            ui.add_spell.field = prev_add_spell_field(ui.add_spell.field);
            false
        }

        KeyCode::Up | KeyCode::Char('k') => {
            if ui.add_spell.field == AddSpellField::Spellbook {
                if ui.add_spell.dropdown_open {
                    if ui.add_spell.dropdown_index == 0 {
                        ui.add_spell.dropdown_open = false;
                    } else {
                        let options_count = state.codex.spellbooks.len() + 1;
                        if options_count > 0 {
                            ui.add_spell.dropdown_index -= 1;
                        }
                    }
                } else {
                    ui.add_spell.field = AddSpellField::Confirm;
                }
            } else {
                ui.add_spell.field = prev_add_spell_field(ui.add_spell.field);
            }
            false
        }

        KeyCode::Down | KeyCode::Char('j') => {
            if ui.add_spell.field == AddSpellField::Spellbook {
                if ui.add_spell.dropdown_open {
                    let options_count = state.codex.spellbooks.len() + 1;
                    if options_count > 0 {
                        ui.add_spell.dropdown_index =
                            (ui.add_spell.dropdown_index + 1) % options_count;
                    }
                } else {
                    ui.add_spell.dropdown_open = true;
                }
            } else {
                ui.add_spell.field = next_add_spell_field(ui.add_spell.field);
            }
            false
        }

        KeyCode::Left | KeyCode::Char('h') => {
            if ui.add_spell.field == AddSpellField::RunMode {
                ui.add_spell.run_mode = cycle_run_mode_left(ui.add_spell.run_mode);
            } else if ui.add_spell.field == AddSpellField::Confirm {
                ui.add_spell.confirm = !ui.add_spell.confirm;
            }
            false
        }

        KeyCode::Right | KeyCode::Char('l') => {
            if ui.add_spell.field == AddSpellField::RunMode {
                ui.add_spell.run_mode = cycle_run_mode_right(ui.add_spell.run_mode);
            } else if ui.add_spell.field == AddSpellField::Confirm {
                ui.add_spell.confirm = !ui.add_spell.confirm;
            }
            false
        }

        KeyCode::Enter => match ui.add_spell.field {
            AddSpellField::Spellbook => {
                if ui.add_spell.dropdown_open {
                    if ui.add_spell.dropdown_index >= state.codex.spellbooks.len() {
                        ui.add_spell.skip_spellbook = true;
                        ui.add_spell.spellbook_index = None;
                    } else {
                        ui.add_spell.skip_spellbook = false;
                        ui.add_spell.spellbook_index = Some(ui.add_spell.dropdown_index);
                    }
                    ui.add_spell.dropdown_open = false;
                } else {
                    ui.add_spell.dropdown_open = true;
                }
                false
            }
            AddSpellField::Confirm => {
                save_spell(state, ui);
                false
            }
            _ => {
                ui.add_spell.field = next_add_spell_field_skip_confirm(ui.add_spell.field);
                false
            }
        },

        KeyCode::Backspace => {
            match ui.add_spell.field {
                AddSpellField::Name => {
                    ui.add_spell.name.pop();
                }
                AddSpellField::Command => {
                    ui.add_spell.command.pop();
                }
                AddSpellField::Lore => {
                    ui.add_spell.lore.pop();
                }
                AddSpellField::School => {
                    ui.add_spell.school.pop();
                }
                AddSpellField::Tags => {
                    ui.add_spell.tags.pop();
                }
                AddSpellField::WorkingDir => {
                    ui.add_spell.working_dir.pop();
                }
                _ => {}
            }
            ui.add_spell.message = None;
            ui.add_spell.has_unsaved = true;
            false
        }

        KeyCode::Char(c) => {
            match ui.add_spell.field {
                AddSpellField::Name => {
                    ui.add_spell.name.push(c);
                }
                AddSpellField::Command => {
                    ui.add_spell.command.push(c);
                }
                AddSpellField::Lore => {
                    ui.add_spell.lore.push(c);
                }
                AddSpellField::School => {
                    ui.add_spell.school.push(c);
                }
                AddSpellField::Tags => {
                    ui.add_spell.tags.push(c);
                }
                AddSpellField::WorkingDir => {
                    ui.add_spell.working_dir.push(c);
                }
                _ => {}
            }
            ui.add_spell.message = None;
            ui.add_spell.has_unsaved = true;
            false
        }

        _ => false,
    }
}

/// Handle key events for AddSpellbook mode
pub fn handle_add_spellbook(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Ctrl+S to save
    if key == KeyCode::Char('s') && modifiers.contains(KeyModifiers::CONTROL) {
        save_spellbook(state, ui);
        return false;
    }

    match key {
        KeyCode::Esc => {
            if ui.add_spellbook.has_unsaved {
                ui.add_spellbook.message = Some((
                    "Unsaved changes - press Esc again to discard".to_string(),
                    true,
                ));
                ui.add_spellbook.has_unsaved = false;
            } else {
                ui.add_spellbook.clear();
                ui.mode = Mode::BrowseSpellbooks(BrowseState::default());
            }
            false
        }

        KeyCode::Tab => {
            ui.add_spellbook.next_field();
            false
        }

        KeyCode::BackTab => {
            ui.add_spellbook.prev_field();
            false
        }

        KeyCode::Up | KeyCode::Char('k') => {
            ui.add_spellbook.prev_field();
            false
        }

        KeyCode::Down | KeyCode::Char('j') | KeyCode::Enter => {
            if key == KeyCode::Enter && ui.add_spellbook.field == AddSpellbookField::Sigil {
                save_spellbook(state, ui);
            } else {
                ui.add_spellbook.next_field();
            }
            false
        }

        KeyCode::Backspace => {
            match ui.add_spellbook.field {
                AddSpellbookField::Name => {
                    ui.add_spellbook.name.pop();
                }
                AddSpellbookField::Cover => {
                    ui.add_spellbook.cover.pop();
                }
                AddSpellbookField::Sigil => {
                    ui.add_spellbook.sigil.pop();
                }
            }
            ui.add_spellbook.has_unsaved = true;
            ui.add_spellbook.message = None;
            false
        }

        KeyCode::Char(c) => {
            match ui.add_spellbook.field {
                AddSpellbookField::Name => {
                    ui.add_spellbook.name.push(c);
                }
                AddSpellbookField::Cover => {
                    ui.add_spellbook.cover.push(c);
                }
                AddSpellbookField::Sigil => {
                    ui.add_spellbook.sigil.push(c);
                }
            }
            ui.add_spellbook.has_unsaved = true;
            ui.add_spellbook.message = None;
            false
        }

        _ => false,
    }
}

/// Saves the current spell and returns to the spellbook list.
fn save_spell(state: &mut State, ui: &mut UiState) {
    // Validate required fields
    if ui.add_spell.name.trim().is_empty() {
        ui.add_spell.message = Some(("Name is required".to_string(), true));
        return;
    }

    if ui.add_spell.command.trim().is_empty() {
        ui.add_spell.message = Some(("Command is required".to_string(), true));
        return;
    }

    // Check for duplicate names (unless editing)
    let is_editing = ui.add_spell.editing_spell_id.is_some();
    let name_lower = ui.add_spell.name.to_lowercase();

    if !is_editing {
        let exists = state
            .codex
            .spells
            .iter()
            .any(|s| s.name.to_lowercase() == name_lower);
        if exists {
            ui.add_spell.message =
                Some(("A spell with this name already exists".to_string(), true));
            return;
        }
    }

    // Parse tags (glyphs)
    let glyphs: Vec<String> = ui
        .add_spell
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Create or update the spell
    let spell_id = if let Some(ref id) = ui.add_spell.editing_spell_id {
        id.clone()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let spell = Spell {
        id: spell_id,
        name: ui.add_spell.name.trim().to_string(),
        incantation: ui.add_spell.command.trim().to_string(),
        lore: ui.add_spell.lore.trim().to_string(),
        school: ui.add_spell.school.trim().to_string(),
        glyphs,
        confirm: ui.add_spell.confirm,
        run_mode: ui.add_spell.run_mode,
        working_dir: ui.add_spell.working_dir.trim().to_string(),
        favorite: false,
    };

    ui.start_loading("Saving spell...");

    // Update in-memory state
    if ui.add_spell.editing_spell_id.is_some() {
        // Update existing
        if let Some(existing) = state.codex.spells.iter_mut().find(|s| s.id == spell.id) {
            *existing = spell.clone();
        }
    } else {
        // Add new
        state.codex.spells.push(spell.clone());
    }

    // Add to spellbook if selected
    if !ui.add_spell.skip_spellbook {
        if let Some(spellbook_idx) = ui.add_spell.spellbook_index {
            if spellbook_idx < state.codex.spellbooks.len() {
                let spellbook = &mut state.codex.spellbooks[spellbook_idx];
                if !spellbook.spell_ids.contains(&spell.id) {
                    spellbook.spell_ids.push(spell.id.clone());
                }
            }
        }
    }

    // Save entire codex to disk
    match Archivist::save(&state.codex, "codex.toml") {
        Ok(_) => {
            ui.stop_loading();
            ui.add_spell.message = Some(("Spell saved!".to_string(), false));
            ui.add_spell.has_unsaved = false;
            log_info!("Spell saved: {}", spell.name);

            // Clear form and return to browse
            ui.clear_add_spell_form();
        }
        Err(e) => {
            ui.stop_loading();
            ui.add_spell.message = Some((format!("Save failed: {}", e), true));
            log_error!("Save failed: {}", e);
        }
    }
}

/// Saves the current spellbook and returns to the spellbook list.
fn save_spellbook(_state: &mut State, ui: &mut UiState) {
    if ui.add_spellbook.name.trim().is_empty() {
        ui.add_spellbook.message = Some(("Name is required".to_string(), true));
        return;
    }

    // Take ownership of values to pass to archivist
    let name = std::mem::take(&mut ui.add_spellbook.name);
    let cover = if ui.add_spellbook.cover.is_empty() {
        None
    } else {
        Some(std::mem::take(&mut ui.add_spellbook.cover))
    };
    let sigil = if ui.add_spellbook.sigil.is_empty() {
        None
    } else {
        Some(std::mem::take(&mut ui.add_spellbook.sigil))
    };

    ui.start_loading("Saving spellbook...");
    match Archivist::append_spellbook("codex.toml", name, cover, sigil) {
        Ok(_) => {
            ui.stop_loading();
            ui.add_spellbook.message = Some(("Spellbook saved!".to_string(), false));
            ui.add_spellbook.has_unsaved = false;
            log_info!("Spellbook saved");
            ui.add_spellbook.clear();
            ui.mode = Mode::BrowseSpellbooks(BrowseState::default());
        }
        Err(e) => {
            ui.stop_loading();
            ui.add_spellbook.message = Some((format!("Save failed: {}", e), true));
            log_error!("Save failed: {}", e);
        }
    }
}

/// Get next field in AddSpell form
fn next_add_spell_field(current: AddSpellField) -> AddSpellField {
    match current {
        AddSpellField::Name => AddSpellField::Command,
        AddSpellField::Command => AddSpellField::Lore,
        AddSpellField::Lore => AddSpellField::School,
        AddSpellField::School => AddSpellField::Tags,
        AddSpellField::Tags => AddSpellField::WorkingDir,
        AddSpellField::WorkingDir => AddSpellField::RunMode,
        AddSpellField::RunMode => AddSpellField::Confirm,
        AddSpellField::Confirm => AddSpellField::Spellbook,
        AddSpellField::Spellbook => AddSpellField::Name,
    }
}

/// Get previous field in AddSpell form
fn prev_add_spell_field(current: AddSpellField) -> AddSpellField {
    match current {
        AddSpellField::Name => AddSpellField::Spellbook,
        AddSpellField::Command => AddSpellField::Name,
        AddSpellField::Lore => AddSpellField::Command,
        AddSpellField::School => AddSpellField::Lore,
        AddSpellField::Tags => AddSpellField::School,
        AddSpellField::WorkingDir => AddSpellField::Tags,
        AddSpellField::RunMode => AddSpellField::WorkingDir,
        AddSpellField::Confirm => AddSpellField::RunMode,
        AddSpellField::Spellbook => AddSpellField::Confirm,
    }
}

/// Get next field, skipping Confirm (for Enter key navigation)
fn next_add_spell_field_skip_confirm(current: AddSpellField) -> AddSpellField {
    match current {
        AddSpellField::Name => AddSpellField::Command,
        AddSpellField::Command => AddSpellField::Lore,
        AddSpellField::Lore => AddSpellField::School,
        AddSpellField::School => AddSpellField::Tags,
        AddSpellField::Tags => AddSpellField::WorkingDir,
        AddSpellField::WorkingDir => AddSpellField::Spellbook,
        AddSpellField::RunMode => AddSpellField::Spellbook,
        AddSpellField::Confirm => AddSpellField::Spellbook,
        AddSpellField::Spellbook => AddSpellField::Name,
    }
}

/// Cycle run mode to the left
fn cycle_run_mode_left(current: RunMode) -> RunMode {
    match current {
        RunMode::Simple => RunMode::Background,
        RunMode::Tui => RunMode::Simple,
        RunMode::Background => RunMode::Tui,
    }
}

/// Cycle run mode to the right
fn cycle_run_mode_right(current: RunMode) -> RunMode {
    match current {
        RunMode::Simple => RunMode::Tui,
        RunMode::Tui => RunMode::Background,
        RunMode::Background => RunMode::Simple,
    }
}
