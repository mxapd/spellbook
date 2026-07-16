//! BrowseSpells mode handler
//!
//! This module handles key events when viewing spells inside a spellbook.

use crate::log_info;
use crate::models::{RecentAction, RunMode, Spell};
use crate::state::State;
use crate::ui::search_overlay::{
    get_spell_at_index, get_spell_by_index, get_spell_count_for_spellbook,
};
use crate::ui::{Overlay, UiState, events, streaming_modal};
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

    // Handle Left arrow or 'h' - go back to BrowseSpellbooks
    if key == KeyCode::Left || key == KeyCode::Char('h') {
        ui.enter_browse_spellbooks();
        return false;
    }

    // '/' key - open explicit search mode
    if key == KeyCode::Char('/') {
        ui.open_search();
        return false;
    }

    // ':' key - open command palette directly
    if key == KeyCode::Char(':') {
        ui.open_search();
        if let Some(query) = ui.search_query_mut() {
            query.push(':');
        }
        crate::ui::events::update_command_filter(ui);
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

    // Handle spell list navigation
    if spell_count > 0 {
        match key {
            // Navigate down (arrow or vim j)
            KeyCode::Down | KeyCode::Char('j') => {
                let current = ui.spell_list_state.selected().unwrap_or(0);
                let next = if current >= spell_count - 1 {
                    0
                } else {
                    current + 1
                };
                ui.spell_list_state.select(Some(next));
                return false;
            }

            // Navigate up (arrow or vim k)
            KeyCode::Up | KeyCode::Char('k') => {
                let current = ui.spell_list_state.selected().unwrap_or(0);
                let prev = if current == 0 {
                    spell_count - 1
                } else {
                    current - 1
                };
                ui.spell_list_state.select(Some(prev));
                return false;
            }

            // Enter - execute command if in command mode, otherwise copy/execute spell
            KeyCode::Enter => {
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

            // 'e' key - edit the selected spell (works with Ctrl in search mode too)
            KeyCode::Char('e')
                if !ui.is_searching() || modifiers.contains(KeyModifiers::CONTROL) =>
            {
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

            // 'd' key - delete the selected spell (with confirmation)
            KeyCode::Char('d')
                if !ui.is_searching() || modifiers.contains(KeyModifiers::CONTROL) =>
            {
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                if let Some((spell_id, _spell_name)) =
                    get_spell_at_index(state, spellbook_index, spell_idx)
                {
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

            // 'f' key - toggle favorite
            KeyCode::Char('f')
                if !ui.is_searching() || modifiers.contains(KeyModifiers::CONTROL) =>
            {
                let spell_index = ui.spell_list_state.selected().unwrap_or(0);
                if let Some((spell_id, _)) = get_spell_at_index(state, spellbook_index, spell_index)
                {
                    if let Some(spell) = state.codex.spells.iter_mut().find(|s| s.id == *spell_id) {
                        spell.favorite = !spell.favorite;
                        let status = if spell.favorite {
                            "added to"
                        } else {
                            "removed from"
                        };
                        ui.show_success(format!("Spell {} favorites", status));
                        ui.flash(
                            crate::ui::FlashAction::Spell {
                                spellbook_index,
                                spell_index: ui.spell_list_state.selected().unwrap_or(0),
                            },
                            None,
                        );
                    }
                }
                return false;
            }

            // 's' - simple execution (exit TUI and run via exec)
            KeyCode::Char('s')
                if !ui.is_searching() || modifiers.contains(KeyModifiers::CONTROL) =>
            {
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_idx) {
                    if spell.confirm {
                        // Show confirmation dialog first
                        ui.confirm_dialog =
                            Some(crate::ui::confirm::ConfirmDialogState::execute_spell(
                                spell.clone(),
                                RunMode::Simple,
                            ));
                        ui.push_overlay(Overlay::ConfirmDialog);
                        return false;
                    }

                    // Execute in simple mode
                    crate::ui::events::execute_simple_mode(&spell, state, ui);
                }
                return false;
            }

            // Ctrl+r - TUI execution with streaming
            KeyCode::Char('r') if modifiers.contains(KeyModifiers::CONTROL) => {
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_idx) {
                    if spell.confirm {
                        ui.confirm_dialog =
                            Some(crate::ui::confirm::ConfirmDialogState::execute_spell(
                                spell.clone(),
                                RunMode::Tui,
                            ));
                        ui.push_overlay(Overlay::ConfirmDialog);
                        return false;
                    }
                    log_info!(
                        "Ctrl+r: Executing spell '{}' in TUI mode with streaming",
                        spell.name
                    );
                    state.add_recent(spell.id.clone(), spell.name.clone(), RecentAction::Run);
                    let working_dir = if spell.working_dir.is_empty() {
                        if state.launch_dir.is_empty() {
                            None
                        } else {
                            Some(state.launch_dir.clone())
                        }
                    } else {
                        Some(spell.working_dir.clone())
                    };
                    if let Err(e) = streaming_modal::start_tui_execution(
                        ui,
                        spell.incantation.clone(),
                        Some(spell.name.clone()),
                        working_dir,
                        state.launch_dir.clone(),
                    ) {
                        ui.show_error(format!("Failed to start TUI mode: {}", e));
                    }
                }
                return false;
            }

            // Ctrl+b - background execution
            KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_idx) {
                    if spell.confirm {
                        ui.confirm_dialog =
                            Some(crate::ui::confirm::ConfirmDialogState::execute_spell(
                                spell.clone(),
                                RunMode::Background,
                            ));
                        ui.push_overlay(Overlay::ConfirmDialog);
                        return false;
                    }
                    log_info!("Ctrl+b: Starting spell '{}' in background", spell.name);
                    match crate::invoker::start_spell(
                        spell.name.clone(),
                        spell.incantation.clone(),
                        if spell.working_dir.is_empty() {
                            if state.launch_dir.is_empty() {
                                None
                            } else {
                                Some(state.launch_dir.clone())
                            }
                        } else {
                            Some(spell.working_dir.clone())
                        },
                    ) {
                        Ok(job_id) => {
                            ui.show_success(format!("Job {} started: {}", job_id, spell.name));
                            ui.flash(
                                crate::ui::FlashAction::Spell {
                                    spellbook_index,
                                    spell_index: ui.spell_list_state.selected().unwrap_or(0),
                                },
                                None,
                            );
                            ui.open_jobs_sidebar(); // Auto-open sidebar when job starts
                            state.add_recent(
                                spell.id.clone(),
                                spell.name.clone(),
                                RecentAction::Run,
                            );
                        }
                        Err(e) => {
                            ui.show_error(format!("Failed to start: {}", e));
                        }
                    }
                }
                return false;
            }

            _ => {}
        }
    }

    // Handle character input for search/filter - only if already in search mode and not with Ctrl
    if let KeyCode::Char(c) = key {
        if ui.is_searching() && !modifiers.contains(KeyModifiers::CONTROL) {
            if let Some(query) = ui.search_query_mut() {
                query.push(c);
            }
            if ui.search_in_command_mode() {
                crate::ui::events::update_command_filter(ui);
            } else {
                update_search_filter(state, ui);
            }
        }
        return false;
    }

    // Handle Backspace in search
    if key == KeyCode::Backspace {
        if let Some(query) = ui.search_query_mut() {
            query.pop();
        }
        if ui.search_query().is_empty() {
            ui.exit_typing_mode();
            ui.enter_browse_spellbooks();
        } else if ui.search_in_command_mode() {
            crate::ui::events::update_command_filter(ui);
        } else {
            update_search_filter(state, ui);
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
        if crate::clipboard::copy_to_clipboard(&spell.incantation) {
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
            crate::ui::events::execute_simple_mode(spell, state, ui);
            return; // exec never returns
        }
        crate::models::RunMode::Tui => {
            log_info!("Using TUI execution mode with streaming");
            let working_dir = if spell.working_dir.is_empty() {
                if state.launch_dir.is_empty() {
                    None
                } else {
                    Some(state.launch_dir.clone())
                }
            } else {
                Some(spell.working_dir.clone())
            };
            match streaming_modal::start_tui_execution(
                ui,
                spell.incantation.clone(),
                Some(spell.name.clone()),
                working_dir,
                state.launch_dir.clone(),
            ) {
                Ok(_) => Ok(crate::clipboard::ExecutionResult {
                    command: spell.incantation.clone(),
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
                spell.incantation.clone(),
                if spell.working_dir.is_empty() {
                    None
                } else {
                    Some(spell.working_dir.clone())
                },
            ) {
                Ok(job_id) => Ok(crate::clipboard::ExecutionResult {
                    command: spell.incantation.clone(),
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

    // Filter spells that match the query in name, lore, school, or glyphs
    let indices: Vec<usize> = state
        .codex
        .spells
        .iter()
        .enumerate()
        .filter(|(_, spell)| {
            spell.name.to_lowercase().contains(&query)
                || spell.lore.to_lowercase().contains(&query)
                || spell.school.to_lowercase().contains(&query)
                || spell
                    .glyphs
                    .iter()
                    .any(|g| g.to_lowercase().contains(&query))
        })
        .map(|(idx, _)| idx)
        .collect();

    *ui.filtered_indices_mut() = indices;

    // Select first result
    if !ui.filtered_indices().is_empty() {
        ui.search_results_state().select(Some(0));
    } else {
        ui.search_results_state().select(None);
    }
}

/// Execute the selected command
fn execute_command(state: &mut State, ui: &mut UiState) {
    let query = ui.search_query().to_string();
    let query_after_colon = query.strip_prefix(':').unwrap_or("");
    let filtered = events::filter_commands(query_after_colon);
    let selected = ui.search_results_state().selected().unwrap_or(0);

    if let Some((cmd_idx, _, _)) = filtered.get(selected) {
        events::execute_command_by_index(*cmd_idx, state, ui);
    } else {
        ui.show_error(format!("Unknown command: {}", query_after_colon));
        log_info!("Unknown command: {}", query_after_colon);
    }
    ui.exit_typing_mode();
}
