use crate::archivist::Archivist;
use crate::models::FocusTarget;
use crate::state::State;
use crate::ui::{streaming_modal, Mode, Overlay, UiState, ViewMode};
use crate::{log_debug, log_error, log_info};
use crossterm::event::{KeyCode, KeyModifiers};

// ============================================================================
// Command System - Shared across all modes
// ============================================================================

struct Command {
    aliases: Vec<&'static str>,
    description: &'static str,
    action: CommandAction,
}

#[derive(Debug)]
enum CommandAction {
    NewSpell,
    NewSpellbook,
    BrowseSpellbooks,
    BrowseSpells,
    CardsView,
    SpinesView,
    ListView,
    CycleTheme,
    Jobs,
    Help,
    Experimental,
    Export,
    Import,
    ToggleImplicitSearch,
}

fn get_commands() -> Vec<Command> {
    vec![
        Command {
            aliases: vec!["n", "new", "new spell", "add spell"],
            description: "Add a new spell",
            action: CommandAction::NewSpell,
        },
        Command {
            aliases: vec![
                "N",
                "new book",
                "new spellbook",
                "add spellbook",
                "add book",
            ],
            description: "Add a new spellbook",
            action: CommandAction::NewSpellbook,
        },
        Command {
            aliases: vec!["b", "books", "browse", "spellbooks"],
            description: "Browse spellbooks",
            action: CommandAction::BrowseSpellbooks,
        },
        Command {
            aliases: vec!["s", "spells"],
            description: "Browse spells in selected spellbook",
            action: CommandAction::BrowseSpells,
        },
        Command {
            aliases: vec!["c", "cards"],
            description: "Card view mode",
            action: CommandAction::CardsView,
        },
        Command {
            aliases: vec!["p", "spines"],
            description: "Spine view mode",
            action: CommandAction::SpinesView,
        },
        Command {
            aliases: vec!["l", "list"],
            description: "List view mode",
            action: CommandAction::ListView,
        },
        Command {
            aliases: vec!["t", "theme"],
            description: "Cycle theme",
            action: CommandAction::CycleTheme,
        },
        Command {
            aliases: vec!["?", "help"],
            description: "Show help",
            action: CommandAction::Help,
        },
        Command {
            aliases: vec!["j", "jobs"],
            description: "View running jobs",
            action: CommandAction::Jobs,
        },
        Command {
            aliases: vec!["e", "experimental"],
            description: "Toggle experimental mode",
            action: CommandAction::Experimental,
        },
        Command {
            aliases: vec!["export", "ex"],
            description: "Export codex or spellbook",
            action: CommandAction::Export,
        },
        Command {
            aliases: vec!["import", "im"],
            description: "Import spells from file",
            action: CommandAction::Import,
        },
        Command {
            aliases: vec!["implicit", "implicit-search"],
            description: "Toggle implicit search mode",
            action: CommandAction::ToggleImplicitSearch,
        },
    ]
}

/// Filter commands based on query string
pub fn filter_commands(query: &str) -> Vec<(usize, &'static str, &'static str)> {
    let query_lower = query.to_lowercase();
    let commands = get_commands();

    commands
        .into_iter()
        .enumerate()
        .filter(|(_, cmd)| {
            cmd.aliases
                .iter()
                .any(|alias| alias.to_lowercase().contains(&query_lower))
        })
        .map(|(idx, cmd)| {
            let primary = cmd.aliases[0];
            (idx, primary, cmd.description)
        })
        .collect()
}

/// Execute a command by its index in the commands list
pub fn execute_command_by_index(idx: usize, state: &mut State, ui: &mut UiState) {
    let commands = get_commands();
    if let Some(cmd) = commands.get(idx) {
        execute_command_by_action(&cmd.action, state, ui);
    }
}

/// Execute a command by its action
fn execute_command_by_action(action: &CommandAction, state: &mut State, ui: &mut UiState) {
    match action {
        CommandAction::NewSpell => {
            ui.enter_add_spell();
            log_info!("Command: new spell");
        }
        CommandAction::NewSpellbook => {
            ui.mode = Mode::AddSpellbook(crate::ui::FormState::default());

            log_info!("Command: new spellbook");
        }
        CommandAction::BrowseSpellbooks => {
            ui.enter_browse_spellbooks();
            log_info!("Command: browse spellbooks");
        }
        CommandAction::BrowseSpells => {
            if let Some(idx) = ui.selected_spellbook() {
                ui.enter_browse_spells(idx);
                log_info!("Command: browse spells");
            } else {
                ui.copy_feedback = Some("Select a spellbook first".to_string());
            }
        }
        CommandAction::CardsView => {
            state.user_settings.view_mode = ViewMode::Cards;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            log_info!("Command: cards view");
        }
        CommandAction::SpinesView => {
            state.user_settings.view_mode = ViewMode::Spines;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            log_info!("Command: spines view");
        }
        CommandAction::ListView => {
            state.user_settings.view_mode = ViewMode::List;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            log_info!("Command: list view");
        }
        CommandAction::CycleTheme => {
            state.cycle_theme();
            log_info!("Command: cycle theme");
        }
        CommandAction::Help => {
            ui.push_overlay(Overlay::Help);
            log_info!("Command: help");
        }
        CommandAction::Jobs => {
            ui.toggle_jobs_sidebar();
            log_info!("Command: jobs");
        }
        CommandAction::Experimental => {
            log_info!("Command: toggle experimental");
            ui.copy_feedback = Some("Experimental mode toggled".to_string());
        }
        CommandAction::Export => {
            log_info!("Command: export (needs arguments)");
            ui.copy_feedback = Some("Usage: :export [filename]".to_string());
        }
        CommandAction::Import => {
            log_info!("Command: import (needs arguments)");
            ui.copy_feedback = Some("Usage: :import <filename>".to_string());
        }
        CommandAction::ToggleImplicitSearch => {
            state.user_settings.implicit_search = !state.user_settings.implicit_search;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            let status = if state.user_settings.implicit_search {
                "enabled"
            } else {
                "disabled"
            };
            ui.copy_feedback = Some(format!("Implicit search {}", status));
            log_info!("Command: toggle implicit search (now {})", status);
        }
    }
}

// ============================================================================
// Main Event Router (Elm Architecture)
// ============================================================================

/// Main event handler - routes events to appropriate handlers
pub fn handle_event(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Priority 1: Active overlays (topmost first)
    if let Some(overlay) = ui.top_overlay() {
        let consumed = handle_overlay(overlay, key, state, ui, modifiers);
        if consumed {
            return false; // Event consumed by overlay
        }
    }

    // Priority 2: Jobs sidebar (if focused and open)
    if ui.jobs_sidebar_open && ui.focus == FocusTarget::JobsSidebar {
        log_debug!("Routing to jobs sidebar");
        return crate::ui::jobs::handle_jobs_key(key, ui);
    }

    // Priority 3: Global keybinds (available in all modes when not typing)
    if let Some(should_quit) = handle_global_keys(key, ui, state, modifiers) {
        return should_quit;
    }

    // Priority 4: Current mode handler (Elm-style routing)
    handle_mode(ui.mode.clone(), key, state, ui, modifiers)
}

/// Handle global keybinds available in all modes
/// Returns Some(true) to quit, Some(false) to consume without quit, None to pass through
fn handle_global_keys(
    key: KeyCode,
    ui: &mut UiState,
    state: &mut State,
    modifiers: KeyModifiers,
) -> Option<bool> {
    // Ctrl+C to quit
    if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        log_info!("Quit via Ctrl+C");
        return Some(true);
    }

    // Ctrl+Z - let terminal handle job control
    if key == KeyCode::Char('z') && modifiers.contains(KeyModifiers::CONTROL) {
        log_info!("Ctrl+Z intercepted - terminal should handle suspend");
        return Some(false);
    }

    // Alt+R to refresh codex
    if key == KeyCode::Char('r') && modifiers.contains(KeyModifiers::ALT) {
        log_info!("Alt+R detected - reloading codex");
        state.reload_codex();
        ui.copy_feedback = Some("Codex refreshed".to_string());
        ui.request_redraw();
        return Some(false);
    }

    // Tab to cycle focus when sidebar is open
    if key == KeyCode::Tab && ui.jobs_sidebar_open {
        ui.cycle_focus();
        log_debug!("Focus cycled to: {:?}", ui.focus);
        return Some(false);
    }

    // Help overlay
    if key == KeyCode::Char('?') && !ui.is_typing() {
        ui.push_overlay(Overlay::Help);
        log_info!("Help overlay opened");
        return Some(false);
    }

    None // Pass through to mode handler
}

/// Handle overlay-specific events
/// Returns true if event was consumed
fn handle_overlay(
    overlay: Overlay,
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    match overlay {
        Overlay::OutputModal => {
            // Use new streaming modal if active, otherwise fall back to legacy
            if ui.streaming_modal.streaming.is_some() || ui.streaming_modal.output.is_streaming {
                let should_close = streaming_modal::handle_streaming_modal_key(key, modifiers, ui);
                if should_close {
                    ui.pop_overlay();
                }
            } else {
                handle_output_popup(key, state, ui);
            }
            true
        }
        Overlay::ConfirmDialog => {
            handle_confirm_dialog(key, state, ui);
            true
        }
        Overlay::CommandPalette => {
            // Command palette handles its own input
            false
        }
        Overlay::Help => {
            if key == KeyCode::Esc {
                ui.pop_overlay();
            }
            true
        }
        Overlay::InputPopup => {
            if ui.input_popup.is_some() {
                let close = handle_input_popup(key, state, ui);
                if close {
                    ui.input_popup = None;
                    ui.pop_overlay();
                }
            }
            true
        }
    }
}

/// Handle mode-specific events (Elm-style "Update" function)
fn handle_mode(
    mode: Mode,
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    match mode {
        Mode::BrowseSpellbooks(_) => {
            log_debug!("Mode: BrowseSpellbooks");
            crate::ui::browse_spellbooks::handle_browse_spellbooks(key, state, ui, modifiers)
        }
        Mode::BrowseSpells(_) => {
            log_debug!("Mode: BrowseSpells");
            crate::ui::browse_spells::handle_browse_spells(key, state, ui, modifiers)
        }
        Mode::AddSpell(_) | Mode::EditSpell(_) => {
            log_debug!("Mode: {:?}", mode);
            crate::ui::form::handle_add_spell(key, state, ui, modifiers)
        }
        Mode::AddSpellbook(_) => {
            log_debug!("Mode: AddSpellbook");
            crate::ui::form::handle_add_spellbook(key, state, ui, modifiers)
        }
    }
}

// ============================================================================
// Overlay Handlers
// ============================================================================

/// Handles the output popup overlay
fn handle_output_popup(key: KeyCode, _state: &mut State, ui: &mut UiState) {
    match key {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('q') => {
            ui.hide_output_popup();
        }
        _ => {}
    }
}

/// Handles the input popup overlay
fn handle_input_popup(key: KeyCode, _state: &mut State, ui: &mut UiState) -> bool {
    let popup = match ui.input_popup.as_mut() {
        Some(p) => p,
        None => return true,
    };

    // The input popup is for placeholder substitution
    match key {
        KeyCode::Esc => {
            // Cancel input
            true
        }
        KeyCode::Char(c) => {
            // Add character to current placeholder value
            if let Some(placeholder) = popup.placeholders.get_mut(popup.placeholder_index) {
                placeholder.value.push(c);
            }
            false
        }
        KeyCode::Backspace => {
            // Remove character from current placeholder value
            if let Some(placeholder) = popup.placeholders.get_mut(popup.placeholder_index) {
                placeholder.value.pop();
            }
            false
        }
        KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
            // Move to next placeholder
            if popup.placeholder_index < popup.placeholders.len() - 1 {
                popup.placeholder_index += 1;
            }
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            // Move to previous placeholder
            if popup.placeholder_index > 0 {
                popup.placeholder_index -= 1;
            }
            false
        }
        KeyCode::Enter => {
            // Validate - execution logic would go here when input popup is used
            if popup.validate() {
                popup.resolved_command = popup.substitute();
                // Note: Command execution not yet implemented for input popup
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Handles the confirm dialog overlay
fn handle_confirm_dialog(key: KeyCode, state: &mut State, ui: &mut UiState) -> bool {
    use crate::ui::confirm::ConfirmAction;

    let should_close = match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(dialog) = ui.confirm_dialog.take() {
                match dialog.action {
                    ConfirmAction::DeleteSpell(spell) => {
                        let spell_id = spell.id.clone();
                        log_info!("Confirmed: delete spell {:?}", spell_id);
                        match state.delete_spell(&spell_id) {
                            Ok(_) => {
                                ui.copy_feedback = Some("Spell deleted".to_string());
                            }
                            Err(e) => {
                                log_error!("Failed to delete spell: {}", e);
                                ui.copy_feedback = Some(format!("Delete failed: {}", e));
                            }
                        }
                    }
                    ConfirmAction::DeleteSpellbook(spellbook_name) => {
                        log_info!("Confirmed: delete spellbook {}", spellbook_name);
                        match state.delete_spellbook(&spellbook_name) {
                            Ok(_) => {
                                ui.copy_feedback = Some("Spellbook deleted".to_string());
                            }
                            Err(e) => {
                                log_error!("Failed to delete spellbook: {}", e);
                                ui.copy_feedback = Some(format!("Delete failed: {}", e));
                            }
                        }
                    }
                    ConfirmAction::ExecuteSpell(spell) => {
                        log_info!("Confirmed: execute spell {}", spell.name);
                        // Execute the spell
                        execute_simple_mode(&spell, state, ui);
                    }
                }
            }
            true
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            log_info!("Cancelled confirmation dialog");
            ui.confirm_dialog = None;
            true
        }
        _ => false,
    };

    if should_close {
        ui.pop_overlay();
    }

    should_close
}

/// Execute a spell in simple mode: write recents, then exec() the command.
pub fn execute_simple_mode(spell: &crate::models::Spell, state: &mut State, _ui: &mut UiState) {
    use crate::models::RecentAction;

    // Step 1: Add to recents (in memory)
    state.add_recent(spell.id.clone(), spell.name.clone(), RecentAction::Run);

    // Step 2: CRITICAL - Persist recents to disk BEFORE exec()
    if let Err(e) = Archivist::save_recents(&state.recents) {
        log_error!("Failed to write recents before exec: {}", e);
    } else {
        log_info!("Recents persisted before simple mode exec");
    }

    // Step 3: Determine working directory
    let working_dir = if spell.working_dir.is_empty() {
        if state.launch_dir.is_empty() {
            None
        } else {
            Some(state.launch_dir.clone())
        }
    } else {
        Some(spell.working_dir.clone())
    };

    // Step 4: Restore terminal (clean shutdown)
    let _ = crossterm::terminal::disable_raw_mode();

    // Step 5: Execute via exec() - this replaces our process
    log_info!(
        "Executing spell '{}' in simple mode: {}",
        spell.name,
        spell.incantation
    );
    crate::invoker::exec_simple(
        &spell.incantation,
        working_dir.as_deref(),
        &state.launch_dir,
    );
}

/// Update filtered commands based on current query after ":"
pub fn update_command_filter(ui: &mut UiState) {
    let query = ui.search_query();
    let query_after_colon = query.strip_prefix(':').unwrap_or("");
    let filtered = filter_commands(query_after_colon);
    *ui.filtered_indices_mut() = filtered.iter().map(|(idx, _, _)| *idx).collect();

    if !ui.filtered_indices().is_empty() {
        ui.search_results_state().select(Some(0));
    } else {
        ui.search_results_state().select(None);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_commands_empty_query() {
        let results = filter_commands("");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_filter_commands_new_spell() {
        let results = filter_commands("n");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(_, name, _)| *name == "n"));
    }

    #[test]
    fn test_filter_commands_new_book() {
        let results = filter_commands("new book");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(_, name, _)| *name == "N"));
    }

    #[test]
    fn test_filter_commands_browse() {
        let results = filter_commands("b");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(_, name, _)| *name == "b"));
    }

    #[test]
    fn test_filter_commands_case_insensitive() {
        let results_upper = filter_commands("N");
        let results_lower = filter_commands("n");
        assert_eq!(results_upper.len(), results_lower.len());
    }

    #[test]
    fn test_filter_commands_no_match() {
        let results = filter_commands("zzzznonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_filter_commands_theme() {
        let results = filter_commands("t");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(_, name, _)| *name == "t"));
    }

    #[test]
    fn test_filter_commands_partial_match() {
        let results = filter_commands("sp");
        assert!(!results.is_empty());
    }
}
