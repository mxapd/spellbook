//! BrowseSpells mode handler
//!
//! This module handles key events when viewing spells inside a spellbook.

use crate::log_info;
use crate::models::{RecentAction, RunMode, Spell};
use crate::state::State;
use crate::ui::search_overlay::{
    get_spell_at_index, get_spell_by_index, get_spell_count_for_spellbook,
};
use crate::ui::{default_launch_dir, BrowseState, Mode, Overlay, UiState, streaming_modal};
use crossterm::event::{KeyCode, KeyModifiers};

/// Handle key events in BrowseSpells mode (inside a spellbook)
pub fn handle_browse_spells(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Get the current spellbook
    let spellbook_index = match ui.selected_spellbook() {
        Some(index) => index,
        None => {
            // No spellbook selected, go back to browse
            ui.enter_browse_spellbooks();
            return false;
        }
    };

    let spell_count = get_spell_count_for_spellbook(state, spellbook_index);

    // Handle Escape - go back to BrowseSpellbooks
    if key == KeyCode::Esc {
        ui.enter_browse_spellbooks();
        return false;
    }

    // Handle Left arrow or 'h' - go back to BrowseSpellbooks (not during search/paused)
    if key == KeyCode::Left || (key == KeyCode::Char('h') && !ui.search_active()) {
        ui.enter_browse_spellbooks();
        return false;
    }

    // '/' key - open explicit search mode
    if key == KeyCode::Char('/') && !ui.is_searching() {
        ui.open_search();
        return false;
    }

    // ':' key - open command palette directly
    if key == KeyCode::Char(':') && !ui.is_searching() {
        ui.open_search();
        if let Some(query) = ui.search_query_mut() {
            query.push(':');
        }
        ui.update_command_filter();
        return false;
    }

    // Ctrl+v - show spell details (view)
    if key == KeyCode::Char('v') && modifiers.contains(KeyModifiers::CONTROL) {
        if let Some(selected_idx) = ui.spell_list_state.selected() {
            if let Some(spellbook_index) = ui.selected_spellbook() {
                if let Some(spell) = get_spell_by_index(state, spellbook_index, selected_idx) {
                    ui.show_spell_details(spell.id);
                }
            }
        }
        return false;
    }

    // ── Navigation ──────────────────────────────────

    let is_search_active = ui.search_active(); // Searching or SearchPaused

    // Helper: navigate within search results
    let nav_search = |ui: &mut UiState, dir: i32| {
        let total = ui.filtered_indices().len();
        if total == 0 { return; }
        let current = ui.search_results_state().selected();
        let next = match current {
            None => { if dir > 0 { 0 } else { total - 1 } }
            Some(c) => {
                if dir > 0 { if c >= total - 1 { 0 } else { c + 1 } }
                else { if c == 0 { total - 1 } else { c - 1 } }
            }
        };
        ui.search_results_state().select(Some(next));
    };

    // Helper: navigate within normal spell list
    let nav_list = |ui: &mut UiState, dir: i32| {
        if spell_count == 0 { return; }
        let current = ui.spell_list_state.selected().unwrap_or(0);
        let next = if dir > 0 {
            if current >= spell_count - 1 { 0 } else { current + 1 }
        } else {
            if current == 0 { spell_count - 1 } else { current - 1 }
        };
        ui.spell_list_state.select(Some(next));
    };

    // Down / Up arrows (with or without Ctrl) and Ctrl+J/K
    match key {
        KeyCode::Down | KeyCode::Char('j') if modifiers.contains(KeyModifiers::CONTROL) => {
            if is_search_active {
                nav_search(ui, 1);
                if ui.is_searching() { ui.pause_search(); }
            } else { nav_list(ui, 1); }
            return false;
        }
        KeyCode::Up | KeyCode::Char('k') if modifiers.contains(KeyModifiers::CONTROL) => {
            if is_search_active {
                nav_search(ui, -1);
                if ui.is_searching() { ui.pause_search(); }
            } else { nav_list(ui, -1); }
            return false;
        }
        KeyCode::Down => {
            if is_search_active {
                nav_search(ui, 1);
                if ui.is_searching() { ui.pause_search(); }
            } else { nav_list(ui, 1); }
            return false;
        }
        KeyCode::Up => {
            if is_search_active {
                nav_search(ui, -1);
                if ui.is_searching() { ui.pause_search(); }
            } else { nav_list(ui, -1); }
            return false;
        }
        _ => {}
    }

    // Bare j/k navigate only when NOT searching/paused (during search they type into query)
    if key == KeyCode::Char('j') && !is_search_active {
        nav_list(ui, 1);
        return false;
    }
    if key == KeyCode::Char('k') && !is_search_active {
        nav_list(ui, -1);
        return false;
    }

    // ── Actions ──────────────────────────────────────

    // Enter - execute command or copy/execute spell
    if key == KeyCode::Enter {
        let is_command_mode = ui.search_query().starts_with(':');
        if is_command_mode {
            execute_command(state, ui);
        } else if modifiers.contains(KeyModifiers::ALT) {
            log_info!("Alt+Enter detected - executing spell");
            let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
            execute_spell_at_index(state, ui, spellbook_index, spell_idx);
        } else {
            copy_spell_at_index(
                state,
                ui,
                spellbook_index,
                ui.spell_list_state.selected().unwrap_or(0),
            );
        }
        return false;
    }

    // Action keys: work when NOT searching OR when in SearchPaused
    fn handle_action<F>(key: KeyCode, active: bool, searching: bool, f: F) -> bool
    where F: Fn() -> bool {
        if active || !searching { f() } else { false }
    }

    match key {
        KeyCode::Char('e') if !ui.is_searching() || !ui.is_searching() => {
            // Edit: works when not Searching (Idle/Viewing or SearchPaused)
            if is_search_active && !ui.is_searching() {
                // SearchPaused mode — act on search result
                if let Some(sel) = ui.search_results_state().selected() {
                    let indices = ui.filtered_indices();
                    if let Some(spell_idx) = indices.get(sel) {
                        if let Some(spell) = state.codex.spells.get(*spell_idx) {
                            for (sb_idx, sb) in state.codex.spellbooks.iter().enumerate() {
                                if sb.spell_ids.contains(&spell.id) {
                                    ui.add_spell.start_edit(spell, Some(sb_idx));
                                    ui.enter_edit_spell(sb_idx, 0);
                                    return false;
                                }
                            }
                        }
                    }
                }
            } else if !is_search_active && spell_count > 0 {
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                if let Some((spell_id, _)) = get_spell_at_index(state, spellbook_index, spell_idx) {
                    if let Some(spell) = state.get_spell(&spell_id) {
                        ui.add_spell.start_edit(spell, Some(spellbook_index));
                        ui.enter_edit_spell(spellbook_index, spell_idx);
                        log_info!("Editing spell: {}", spell.name);
                        return false;
                    }
                }
            }
            return false;
        }

        KeyCode::Char('d') if !ui.is_searching() || !ui.is_searching() => {
            if is_search_active && !ui.is_searching() {
                if let Some(sel) = ui.search_results_state().selected() {
                    let indices = ui.filtered_indices();
                    if let Some(spell_idx) = indices.get(sel) {
                        if let Some(spell) = state.codex.spells.get(*spell_idx) {
                            ui.confirm_dialog = Some(
                                crate::ui::confirm::ConfirmDialogState::delete_spell(spell.clone()),
                            );
                            ui.push_overlay(Overlay::ConfirmDialog);
                            return false;
                        }
                    }
                }
            } else if !is_search_active && spell_count > 0 {
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                if let Some((spell_id, _)) = get_spell_at_index(state, spellbook_index, spell_idx) {
                    if let Some(spell) = state.get_spell(&spell_id) {
                        ui.confirm_dialog = Some(
                            crate::ui::confirm::ConfirmDialogState::delete_spell(spell.clone()),
                        );
                        ui.push_overlay(Overlay::ConfirmDialog);
                        log_info!("Delete confirmation requested for: {}", spell_id);
                        return false;
                    }
                }
            }
            return false;
        }

        KeyCode::Char('s') if !ui.is_searching() || !ui.is_searching() => {
            if is_search_active && !ui.is_searching() {
                if let Some(sel) = ui.search_results_state().selected() {
                    let indices = ui.filtered_indices();
                    if let Some(spell_idx) = indices.get(sel) {
                        if let Some(spell) = state.codex.spells.get(*spell_idx).cloned() {
                            if spell.confirm {
                                ui.confirm_dialog = Some(
                                    crate::ui::confirm::ConfirmDialogState::execute_spell(spell, RunMode::Simple),
                                );
                                ui.push_overlay(Overlay::ConfirmDialog);
                            } else {
                                crate::ui::execute_simple_mode(&spell, state, ui);
                            }
                            return false;
                        }
                    }
                }
            } else if !is_search_active && spell_count > 0 {
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_idx) {
                    if spell.confirm {
                        ui.confirm_dialog = Some(
                            crate::ui::confirm::ConfirmDialogState::execute_spell(spell.clone(), RunMode::Simple),
                        );
                        ui.push_overlay(Overlay::ConfirmDialog);
                    } else {
                        crate::ui::execute_simple_mode(&spell, state, ui);
                    }
                    return false;
                }
            }
            return false;
        }

        _ => {}
    }

    // Handle character input
    if let KeyCode::Char(c) = key {
        if !modifiers.contains(KeyModifiers::CONTROL) {
            if ui.is_searching() {
                // Actively searching — type into query
                if let Some(query) = ui.search_query_mut() {
                    query.push(c);
                }
                if ui.search_in_command_mode() {
                    ui.update_command_filter();
                } else {
                    update_search_filter(state, ui);
                }
            } else if let Some(BrowseState::SearchPaused { .. }) = match &ui.mode {
                Mode::BrowseSpells(s) => Some(s),
                _ => None,
            } {
                // SearchPaused — resume search with this character
                ui.resume_search(Some(c));
                update_search_filter(state, ui);
            }
        }
        return false;
    }

    // Handle Backspace in search
    if key == KeyCode::Backspace && ui.search_active() {
        if ui.is_searching() {
            if let Some(query) = ui.search_query_mut() {
                query.pop();
            }
            if ui.search_query().is_empty() {
                ui.exit_typing_mode();
                ui.enter_browse_spellbooks();
            } else if ui.search_in_command_mode() {
                ui.update_command_filter();
            } else {
                update_search_filter(state, ui);
            }
        }
        return false;
    }

    false
}

/// Copy spell at index to clipboard
fn copy_spell_at_index(
    state: &mut State,
    ui: &mut UiState,
    spellbook_index: usize,
    spell_index: usize,
) {
    if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_index) {
        if crate::clipboard::copy_to_clipboard(&spell.command) {
            ui.show_success(format!("Copied: {}", spell.name));
            ui.flash(
                crate::ui::FlashAction::Spell {
                    spellbook_index,
                    spell_index,
                },
                None,
            );
            state.add_recent(spell.id, spell.name, RecentAction::Copy);
        } else {
            ui.show_error("Failed to copy to clipboard".to_string());
        }
    }
}

/// Execute spell at index
fn execute_spell_at_index(
    state: &mut State,
    ui: &mut UiState,
    spellbook_index: usize,
    spell_index: usize,
) {
    if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_index) {
        log_info!(
            "Executing spell '{}' (confirm={})",
            spell.name,
            spell.confirm
        );

        if spell.confirm {
            let run_mode = spell.run_mode;
            ui.confirm_dialog = Some(crate::ui::confirm::ConfirmDialogState::execute_spell(
                spell, run_mode,
            ));
            ui.push_overlay(Overlay::ConfirmDialog);
        } else {
            start_spell_execution(state, ui, &spell);
        }
    }
}

/// Start spell execution
fn start_spell_execution(state: &mut State, ui: &mut UiState, spell: &Spell) {
    log_info!("Starting execution for spell '{}'", spell.name);
    state.add_recent(spell.id.clone(), spell.name.clone(), RecentAction::Run);

    let result = match spell.run_mode {
        crate::models::RunMode::Simple => {
            log_info!("Using simple execution mode");
            // For simple mode, we need to save recents then exec
            crate::ui::execute_simple_mode(spell, state, ui);
            return; // exec never returns
        }
        crate::models::RunMode::Tui => {
            log_info!("Using TUI execution mode with streaming");
            let working_dir = if spell.working_dir.is_empty() {
                default_launch_dir()
            } else {
                Some(spell.working_dir.clone())
            };
            let ld = default_launch_dir().unwrap_or_default();
            match streaming_modal::start_tui_execution(
                ui,
                spell.command.clone(),
                Some(spell.name.clone()),
                working_dir,
                ld,
            ) {
                Ok(_) => Ok(crate::clipboard::ExecutionResult {
                    command: spell.command.clone(),
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: Some(0),
                    full_stdout: String::new(),
                    full_stderr: String::new(),
                    pid: None,
                    spell_name: Some(spell.name.clone()),
                }),
                Err(e) => Err(e),
            }
        }
        crate::models::RunMode::Background => {
            log_info!("Using background execution mode");
            match crate::invoker::start_spell(
                spell.name.clone(),
                spell.command.clone(),
                if spell.working_dir.is_empty() {
                    None
                } else {
                    Some(spell.working_dir.clone())
                },
            ) {
                Ok(job_id) => Ok(crate::clipboard::ExecutionResult {
                    command: spell.command.clone(),
                    stdout: format!("Job {} started", job_id),
                    stderr: String::new(),
                    exit_code: Some(0),
                    full_stdout: format!("Job {} started", job_id),
                    full_stderr: String::new(),
                    pid: Some(job_id as u32),
                    spell_name: Some(spell.name.clone()),
                }),
                Err(e) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )),
            }
        }
    };

    match result {
        Ok(exec_result) => {
            log_info!("Spell '{}' executed successfully", spell.name);
            if !exec_result.stdout.is_empty() {
                ui.show_info(exec_result.stdout.lines().next().unwrap_or("").to_string());
            } else {
                ui.show_success(format!("Executed: {}", spell.name));
            }
            ui.show_output_popup(exec_result);
        }
        Err(e) => {
            log_info!("Failed to execute spell '{}': {}", spell.name, e);
            ui.show_error(format!("Failed: {}", e));
        }
    }
}

/// Filters all spells by the current search query.
pub fn update_search_filter(state: &State, ui: &mut UiState) {
    let query = ui.search_query().to_lowercase();

    if query.is_empty() {
        ui.filtered_indices_mut().clear();
        ui.search_results_state().select(None);
        return;
    }

    // Filter spells that match the query in name, description, category, or tags
    let indices: Vec<usize> = state
        .codex
        .spells
        .iter()
        .enumerate()
        .filter(|(_, spell)| {
            spell.name.to_lowercase().contains(&query)
                || spell.description.to_lowercase().contains(&query)
                || spell.category.to_lowercase().contains(&query)
                || spell
                    .tags
                    .iter()
                    .any(|g| g.to_lowercase().contains(&query))
        })
        .map(|(idx, _)| idx)
        .collect();

    *ui.filtered_indices_mut() = indices;
}

/// Execute the selected command
fn execute_command(state: &mut State, ui: &mut UiState) {
    let query = ui.search_query().to_string();
    let query_after_colon = query.strip_prefix(':').unwrap_or("");
    let filtered = crate::ui::filter_commands(query_after_colon);
    let selected = ui.search_results_state().selected().unwrap_or(0);

    if let Some((cmd_idx, _, _)) = filtered.get(selected) {
        ui.execute_command_by_index(*cmd_idx, state);
    } else {
        ui.show_error(format!("Unknown command: {}", query_after_colon));
        log_info!("Unknown command: {}", query_after_colon);
    }
    ui.exit_typing_mode();
}
