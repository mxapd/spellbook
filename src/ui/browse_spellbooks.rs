//! BrowseSpellbooks mode handler
//!
//! This module handles key events when browsing the list of spellbooks.

use crate::archivist::Archivist;
use crate::log_error;
use crate::log_info;
use crate::state::State;
use crate::ui::browse_spells::update_search_filter;
use crate::ui::search_overlay::{CardDirection, find_nearest_card, total_spellbook_count};
use crate::ui::{Overlay, UiState};
use crossterm::event::{KeyCode, KeyModifiers};

/// Handle key events in BrowseSpellbooks mode
pub fn handle_browse_spellbooks(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Handle Escape - exit search if active
    if key == KeyCode::Esc {
        if ui.is_searching() {
            ui.exit_typing_mode();
        }
        ui.set_showing_spellbooks(true);
        ui.set_search_spellbook_index(Some(0));
        return false;
    }

    let spellbook_count = total_spellbook_count(state);
    let is_searching = ui.is_searching();
    let has_query = !ui.search_query().is_empty();

    // If in search mode with query, handle search navigation
    if is_searching || has_query {
        return handle_search_navigation(key, modifiers, state, ui);
    }

    // Handle spellbook browser navigation (card view)
    if ui.showing_spellbooks() {
        // Enter opens the selected spellbook
        if key == KeyCode::Enter {
            if let Some(idx) = ui.search_spellbook_index() {
                if idx < spellbook_count {
                    ui.enter_browse_spells(idx);
                    return false;
                }
            }
        }

        let cards_per_row = ui.search_items_per_row().max(1);
        let scroll = ui.search_spellbook_scroll();

        // Card dimensions (must match render_spellbook_cards)
        let card_gap = 2;
        let card_width = 14;
        let card_height = 10;

        // Calculate grid offset
        let card_unit = card_width + card_gap;
        let total_grid_width = cards_per_row * card_unit - card_gap;
        let available_width = 80;
        let grid_offset = ((available_width as i32 - total_grid_width as i32) / 2).max(0) as u16;

        // Navigate with nearest-neighbor in cards view (arrows or vim hjkl)
        match key {
            KeyCode::Right | KeyCode::Char('l') => {
                if spellbook_count > 0 {
                    let current = ui.search_spellbook_index().unwrap_or(0);
                    let next = find_nearest_card(
                        current,
                        CardDirection::Right,
                        spellbook_count,
                        cards_per_row,
                        card_width,
                        card_height,
                        card_gap,
                        grid_offset,
                    );
                    ui.set_search_spellbook_index(Some(next));

                    if next >= scroll + cards_per_row {
                        ui.set_search_spellbook_scroll((next + 1).saturating_sub(cards_per_row));
                    }
                }
                return false;
            }

            KeyCode::Left | KeyCode::Char('h') => {
                if spellbook_count > 0 {
                    let current = ui.search_spellbook_index().unwrap_or(0);
                    let prev = find_nearest_card(
                        current,
                        CardDirection::Left,
                        spellbook_count,
                        cards_per_row,
                        card_width,
                        card_height,
                        card_gap,
                        grid_offset,
                    );
                    ui.set_search_spellbook_index(Some(prev));

                    if prev < scroll {
                        ui.set_search_spellbook_scroll(prev);
                    }
                }
                return false;
            }

            KeyCode::Down | KeyCode::Char('j') => {
                if spellbook_count > 0 {
                    let current = ui.search_spellbook_index().unwrap_or(0);
                    let next = find_nearest_card(
                        current,
                        CardDirection::Down,
                        spellbook_count,
                        cards_per_row,
                        card_width,
                        card_height,
                        card_gap,
                        grid_offset,
                    );
                    ui.set_search_spellbook_index(Some(next));

                    let visible_rows = (spellbook_count.saturating_sub(scroll) + cards_per_row - 1)
                        / cards_per_row;
                    if next >= scroll + visible_rows * cards_per_row {
                        ui.set_search_spellbook_scroll(scroll + cards_per_row);
                    }
                }
                return false;
            }

            KeyCode::Up | KeyCode::Char('k') => {
                if spellbook_count > 0 {
                    let current = ui.search_spellbook_index().unwrap_or(0);
                    let prev = find_nearest_card(
                        current,
                        CardDirection::Up,
                        spellbook_count,
                        cards_per_row,
                        card_width,
                        card_height,
                        card_gap,
                        grid_offset,
                    );
                    ui.set_search_spellbook_index(Some(prev));

                    if prev < scroll {
                        ui.set_search_spellbook_scroll(prev);
                    }
                }
                return false;
            }

            _ => {}
        }

        // Ctrl+D - Delete spellbook
        if key == KeyCode::Char('d') && modifiers.contains(KeyModifiers::CONTROL) {
            if let Some(idx) = ui.search_spellbook_index() {
                if idx < spellbook_count {
                    use crate::ui::search_overlay::get_spellbook_item;
                    if let Some(item) = get_spellbook_item(state, idx) {
                        // Don't allow deleting virtual spellbooks
                        if item.is_virtual() {
                            ui.show_error("Cannot delete virtual spellbooks".to_string());
                            return false;
                        }
                        let name = item.name();
                        ui.confirm_dialog = Some(
                            crate::ui::confirm::ConfirmDialogState::delete_spellbook(name),
                        );
                        ui.push_overlay(Overlay::ConfirmDialog);
                    }
                }
            }
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
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();

            if selected < indices.len() {
                let spell_idx = indices[selected];
                if let Some(spell) = state.codex.spells.get(spell_idx) {
                    ui.show_spell_details(spell.id.clone());
                }
            }
            return false;
        }

        // Ctrl+e - edit selected spell (works in both search and normal mode)
        if key == KeyCode::Char('e') && modifiers.contains(KeyModifiers::CONTROL) {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();

            if selected < indices.len() {
                let spell_idx = indices[selected];
                if let Some(spell) = state.codex.spells.get(spell_idx) {
                    // Find which spellbook contains this spell
                    for (sb_idx, sb) in state.codex.spellbooks.iter().enumerate() {
                        if sb.spell_ids.contains(&spell.id) {
                            ui.add_spell.start_edit(spell, Some(sb_idx));
                            ui.enter_edit_spell(sb_idx, 0);
                            return false;
                        }
                    }
                    // If spell not in any spellbook, edit without spellbook
                    ui.add_spell.start_edit(spell, None);
                    ui.enter_edit_spell(0, 0);
                }
            }
            return false;
        }

        // Ctrl+d - delete selected spell (works in both search and normal mode)
        if key == KeyCode::Char('d') && modifiers.contains(KeyModifiers::CONTROL) {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();

            if selected < indices.len() {
                let spell_idx = indices[selected];
                if let Some(spell) = state.codex.spells.get(spell_idx) {
                    ui.confirm_dialog = Some(crate::ui::confirm::ConfirmDialogState::delete_spell(
                        spell.clone(),
                    ));
                    ui.push_overlay(Overlay::ConfirmDialog);
                }
            }
            return false;
        }

        // Ctrl+f - toggle favorite (works in both search and normal mode)
        if key == KeyCode::Char('f') && modifiers.contains(KeyModifiers::CONTROL) {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();

            if selected < indices.len() {
                let spell_idx = indices[selected];
                let new_favorite = if let Some(spell) = state.codex.spells.get_mut(spell_idx) {
                    spell.favorite = !spell.favorite;
                    spell.favorite
                } else {
                    return false;
                };
                if let Err(e) = Archivist::save(&state.codex, "codex.toml") {
                    log_error!("Failed to save codex: {}", e);
                }
                let status = if new_favorite {
                    "added to"
                } else {
                    "removed from"
                };
                ui.show_success(format!("Spell {} favorites", status));
                ui.flash(
                    crate::ui::FlashAction::SearchResult { index: selected },
                    None,
                );
            }
            return false;
        }

        // Any character input - only if already in search mode and not with Ctrl
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
    }

    false
}

/// Handle navigation when in search mode
fn handle_search_navigation(
    key: KeyCode,
    modifiers: KeyModifiers,
    state: &mut State,
    ui: &mut UiState,
) -> bool {
    let is_command_mode = ui.search_query().starts_with(':');

    // Enter - execute command if in command mode, otherwise copy selected spell
    if key == KeyCode::Enter {
        if is_command_mode {
            execute_command(state, ui);
        } else {
            // Copy selected spell to clipboard
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();

            if selected < indices.len() {
                let spell_idx = indices[selected];
                if let Some(spell) = state.codex.spells.get(spell_idx) {
                    if crate::clipboard::copy_to_clipboard(&spell.incantation) {
                        ui.show_success(format!("Copied: {}", spell.name));
                        ui.flash(
                            crate::ui::FlashAction::SearchResult { index: selected },
                            None,
                        );
                        state.add_recent(
                            spell.id.clone(),
                            spell.name.clone(),
                            crate::models::RecentAction::Copy,
                        );
                    } else {
                        ui.show_error("Failed to copy to clipboard".to_string());
                    }
                }
            }
        }
        return false;
    }

    // Handle Ctrl+j/Ctrl+k for navigation
    if modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char('j') = key {
            if is_command_mode {
                navigate_command_results(ui, 1);
            } else {
                navigate_search_results(ui, 1);
            }
            return false;
        }
        if let KeyCode::Char('k') = key {
            if is_command_mode {
                navigate_command_results(ui, -1);
            } else {
                navigate_search_results(ui, -1);
            }
            return false;
        }

        // Ctrl+e - edit selected spell
        if let KeyCode::Char('e') = key {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();
            if selected < indices.len() {
                let spell_idx = indices[selected];
                if let Some(spell) = state.codex.spells.get(spell_idx) {
                    for (sb_idx, sb) in state.codex.spellbooks.iter().enumerate() {
                        if sb.spell_ids.contains(&spell.id) {
                            ui.add_spell.start_edit(spell, Some(sb_idx));
                            ui.enter_edit_spell(sb_idx, 0);
                            return false;
                        }
                    }
                    ui.add_spell.start_edit(spell, None);
                    ui.enter_edit_spell(0, 0);
                }
            }
            return false;
        }

        // Ctrl+d - delete selected spell
        if let KeyCode::Char('d') = key {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();
            if selected < indices.len() {
                let spell_idx = indices[selected];
                if let Some(spell) = state.codex.spells.get(spell_idx) {
                    ui.confirm_dialog = Some(crate::ui::confirm::ConfirmDialogState::delete_spell(
                        spell.clone(),
                    ));
                    ui.push_overlay(Overlay::ConfirmDialog);
                }
            }
            return false;
        }

        // Ctrl+f - toggle favorite
        if let KeyCode::Char('f') = key {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();
            if selected < indices.len() {
                let spell_idx = indices[selected];
                let new_favorite = if let Some(spell) = state.codex.spells.get_mut(spell_idx) {
                    spell.favorite = !spell.favorite;
                    spell.favorite
                } else {
                    return false;
                };
                if let Err(e) = Archivist::save(&state.codex, "codex.toml") {
                    log_error!("Failed to save codex: {}", e);
                }
                let status = if new_favorite {
                    "added to"
                } else {
                    "removed from"
                };
                ui.show_success(format!("Spell {} favorites", status));
                ui.flash(
                    crate::ui::FlashAction::SearchResult { index: selected },
                    None,
                );
            }
            return false;
        }

        // 's' - simple execution (exit TUI and run via exec)
        if let KeyCode::Char('s') = key {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();
            if selected < indices.len() {
                let spell_idx = indices[selected];
                if let Some(spell) = state.codex.spells.get(spell_idx).cloned() {
                    if spell.confirm {
                        ui.confirm_dialog =
                            Some(crate::ui::confirm::ConfirmDialogState::execute_spell(
                                spell,
                                crate::models::RunMode::Simple,
                            ));
                        ui.push_overlay(Overlay::ConfirmDialog);
                        return false;
                    }
                    crate::ui::events::execute_simple_mode(&spell, state, ui);
                }
            }
            return false;
        }

        // Ctrl+r - TUI execution with streaming
        if let KeyCode::Char('r') = key {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();
            if selected < indices.len() {
                let spell_idx = indices[selected];
                let spell_opt = state.codex.spells.get(spell_idx).cloned();
                if let Some(spell) = spell_opt {
                    if spell.confirm {
                        ui.confirm_dialog =
                            Some(crate::ui::confirm::ConfirmDialogState::execute_spell(
                                spell,
                                crate::models::RunMode::Tui,
                            ));
                        ui.push_overlay(Overlay::ConfirmDialog);
                        return false;
                    }
                    log_info!("Ctrl+r: Executing spell '{}' in TUI mode", spell.name);
                    let spell_id = spell.id.clone();
                    let spell_name = spell.name.clone();
                    state.add_recent(spell_id, spell_name, crate::models::RecentAction::Run);
                    let working_dir = if spell.working_dir.is_empty() {
                        if state.launch_dir.is_empty() {
                            None
                        } else {
                            Some(state.launch_dir.clone())
                        }
                    } else {
                        Some(spell.working_dir.clone())
                    };
                    if let Err(e) = crate::ui::streaming_modal::start_tui_execution(
                        ui,
                        spell.incantation.clone(),
                        Some(spell.name.clone()),
                        working_dir,
                        state.launch_dir.clone(),
                    ) {
                        ui.show_error(format!("Failed to start TUI mode: {}", e));
                    }
                }
            }
            return false;
        }

        // Ctrl+b - background execution
        if let KeyCode::Char('b') = key {
            let selected = ui.search_results_state().selected().unwrap_or(0);
            let indices = ui.filtered_indices();
            if selected < indices.len() {
                let spell_idx = indices[selected];
                let spell_opt = state.codex.spells.get(spell_idx).cloned();
                if let Some(spell) = spell_opt {
                    if spell.confirm {
                        ui.confirm_dialog =
                            Some(crate::ui::confirm::ConfirmDialogState::execute_spell(
                                spell,
                                crate::models::RunMode::Background,
                            ));
                        ui.push_overlay(Overlay::ConfirmDialog);
                        return false;
                    }
                    log_info!("Ctrl+b: Starting spell '{}' in background", spell.name);
                    let spell_id = spell.id.clone();
                    let spell_name = spell.name.clone();
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
                            ui.open_jobs_sidebar();
                            state.add_recent(
                                spell_id,
                                spell_name,
                                crate::models::RecentAction::Run,
                            );
                        }
                        Err(e) => {
                            ui.show_error(format!("Failed to start background job: {}", e));
                        }
                    }
                }
            }
            return false;
        }
    }

    // Handle character input - only type in search mode when NOT holding Ctrl
    if let KeyCode::Char(c) = key
        && !modifiers.contains(KeyModifiers::CONTROL)
    {
        if let Some(query) = ui.search_query_mut() {
            query.push(c);
        }

        if ui.search_in_command_mode() {
            crate::ui::events::update_command_filter(ui);
        } else {
            update_search_filter(state, ui);
        }
        return false;
    }

    // Navigate results (arrows only)
    match key {
        KeyCode::Down => {
            if is_command_mode {
                navigate_command_results(ui, 1);
            } else {
                navigate_search_results(ui, 1);
            }
            return false;
        }

        KeyCode::Up => {
            if is_command_mode {
                navigate_command_results(ui, -1);
            } else {
                navigate_search_results(ui, -1);
            }
            return false;
        }

        _ => {}
    }

    // Handle backspace
    if key == KeyCode::Backspace {
        let was_empty = ui.search_query().is_empty();

        if was_empty {
            // Query was already empty, exit search mode
            ui.exit_typing_mode();
            ui.set_showing_spellbooks(true);
            ui.set_search_spellbook_index(Some(0));
        } else {
            // Remove last character
            if let Some(query) = ui.search_query_mut() {
                query.pop();
            }

            // Update filter (even if now empty, stay in search mode)
            if ui.search_in_command_mode() {
                crate::ui::events::update_command_filter(ui);
            } else {
                update_search_filter(state, ui);
            }
        }
        return false;
    }

    false
}

/// Execute the selected command
fn execute_command(state: &mut State, ui: &mut UiState) {
    use crate::ui::events::{execute_command_by_index, filter_commands};

    let query = ui.search_query().to_string();
    let query_after_colon = query.strip_prefix(':').unwrap_or("");
    let filtered = filter_commands(query_after_colon);
    let selected = ui.search_results_state().selected().unwrap_or(0);

    if let Some((cmd_idx, _, _)) = filtered.get(selected) {
        execute_command_by_index(*cmd_idx, state, ui);
    } else {
        // No matching command found
        ui.show_error(format!("Unknown command: {}", query_after_colon));
        log_info!("Unknown command: {}", query_after_colon);
    }
    ui.exit_typing_mode();
}

/// Navigate command results
fn navigate_command_results(ui: &mut UiState, direction: i32) {
    use crate::ui::events::filter_commands;

    let query = ui.search_query();
    let query_after_colon = query.strip_prefix(':').unwrap_or("");
    let filtered = filter_commands(query_after_colon);
    let count = filtered.len();

    if count > 0 {
        let current = ui.search_results_state().selected().unwrap_or(0);
        let next = if direction > 0 {
            if current >= count - 1 { 0 } else { current + 1 }
        } else {
            if current == 0 { count - 1 } else { current - 1 }
        };
        ui.search_results_state().select(Some(next));
    }
}

/// Navigate search results
fn navigate_search_results(ui: &mut UiState, direction: i32) {
    let count = ui.filtered_indices().len();

    if count > 0 {
        let current = ui.search_results_state().selected().unwrap_or(0);
        let next = if direction > 0 {
            if current >= count - 1 { 0 } else { current + 1 }
        } else {
            if current == 0 { count - 1 } else { current - 1 }
        };
        ui.search_results_state().select(Some(next));
    }
}
