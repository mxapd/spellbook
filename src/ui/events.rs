use crate::archivist::{Archivist, MergeStrategy};
use crate::clipboard;
use crate::models::FocusTarget;
use crate::state::State;
use crate::ui::search_overlay::{find_nearest_card, total_spellbook_count, CardDirection};
use crate::ui::{streaming_modal, Mode, Overlay, SearchMode, UiState, ViewMode};
use crate::{log_debug, log_error, log_info, log_warn};
use crossterm::event::{KeyCode, KeyModifiers};

struct Command {
    aliases: Vec<&'static str>,
    description: &'static str,
    action: CommandAction,
}

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
            description: "Import from file",
            action: CommandAction::Import,
        },
    ]
}

pub fn filter_commands(query: &str) -> Vec<(usize, &'static str, &'static str)> {
    let query_lower = query.to_lowercase();
    let query_lower = query_lower.trim();

    get_commands()
        .into_iter()
        .enumerate()
        .filter(|(_, cmd)| {
            cmd.aliases
                .iter()
                .any(|alias| alias.to_lowercase().starts_with(&query_lower))
        })
        .map(|(idx, cmd)| (idx, cmd.aliases[0], cmd.description))
        .collect()
}

fn execute_command_by_index(idx: usize, state: &mut State, ui: &mut UiState) {
    let commands = get_commands();
    if let Some(cmd) = commands.get(idx) {
        execute_command_by_action(&cmd.action, state, ui);
    }
}

fn execute_command_by_action(action: &CommandAction, state: &mut State, ui: &mut UiState) {
    match action {
        CommandAction::NewSpell => {
            ui.search_mode = SearchMode::AddSpell;
            ui.add_spell.field = crate::ui::AddSpellField::Name;
            ui.is_typing = true;
            log_info!("Command: new spell");
        }
        CommandAction::NewSpellbook => {
            ui.search_mode = SearchMode::AddSpellbook;
            ui.is_typing = true;
            log_info!("Command: new spellbook");
        }
        CommandAction::BrowseSpellbooks => {
            ui.search_mode = SearchMode::BrowseSpellbooks;
            ui.set_showing_spellbooks(true);
            ui.selected_spellbook = None;
            log_info!("Command: browse");
        }
        CommandAction::BrowseSpells => {
            if let Some(idx) = ui.selected_spellbook {
                ui.search_mode = SearchMode::BrowseSpells;
                ui.spell_list_state.select(Some(0));
                log_info!("Command: spells");
            } else {
                ui.copy_feedback = Some("Select a spellbook first".to_string());
            }
        }
        CommandAction::CardsView => {
            state.user_settings.view_mode = ViewMode::Cards;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            ui.view_mode = ViewMode::Cards;
            log_info!("Command: cards view");
        }
        CommandAction::SpinesView => {
            state.user_settings.view_mode = ViewMode::Spines;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            ui.view_mode = ViewMode::Spines;
            log_info!("Command: spines view");
        }
        CommandAction::ListView => {
            state.user_settings.view_mode = ViewMode::List;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            ui.view_mode = ViewMode::List;
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
            log_info!("Command: jobs (sidebar: {})", ui.jobs_sidebar_open);
        }
        CommandAction::Experimental => {
            state.user_settings.experimental_mode = !state.user_settings.experimental_mode;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            let status = if state.user_settings.experimental_mode {
                "on"
            } else {
                "off"
            };
            ui.copy_feedback = Some(format!("Experimental mode: {}", status));
            log_info!("Command: experimental mode {}", status);
        }
        CommandAction::Export => {
            ui.copy_feedback = Some("Usage: :export [filename] or :export <spellbook>".to_string());
            log_info!("Command: export (showing usage)");
        }
        CommandAction::Import => {
            ui.copy_feedback = Some("Usage: :import <filename>".to_string());
            log_info!("Command: import (showing usage)");
        }
    }
}

/// Main event handler - implements event priority: Overlay → Sidebar → Mode → Global
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
        // If overlay didn't consume, fall through to next priority
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

    // Priority 4: Current mode handler
    handle_mode(ui.mode, key, state, ui, modifiers)
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

    // Theme cycling with 't' - disabled while typing
    if key == KeyCode::Char('t') && !ui.is_typing {
        state.cycle_theme();
        return Some(false);
    }

    // View mode cycling with 'v' - disabled while typing
    if key == KeyCode::Char('v') && !ui.is_typing {
        state.cycle_view_mode();
        return Some(false);
    }

    // Tab to cycle focus when sidebar is open
    if key == KeyCode::Tab && ui.jobs_sidebar_open {
        ui.cycle_focus();
        log_debug!("Focus cycled to: {:?}", ui.focus);
        return Some(false);
    }

    // Help overlay
    if key == KeyCode::Char('?') && !ui.is_typing {
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

/// Handle mode-specific events
fn handle_mode(
    mode: Mode,
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Sync view_mode from state to ui for render functions
    ui.view_mode = state.user_settings.view_mode;

    match mode {
        Mode::BrowseSpellbooks => {
            log_debug!("Mode: BrowseSpellbooks");
            // Use search overlay handler for now (they're equivalent)
            handle_search(key, state, ui, modifiers)
        }
        Mode::BrowseSpells => {
            log_debug!("Mode: BrowseSpells");
            // Map to legacy spell list handler
            handle_spell_list(key, state, ui, modifiers)
        }
        Mode::AddSpell | Mode::EditSpell => {
            log_debug!("Mode: {:?}", mode);
            handle_add_spell(key, state, ui, modifiers)
        }
        Mode::AddSpellbook => {
            log_debug!("Mode: AddSpellbook");
            handle_add_spellbook(key, state, ui, modifiers)
        }
    }
}

fn handle_help(_key: KeyCode, _ui: &mut UiState) -> bool {
    // Help overlay is now handled via the Overlay system in handle_overlay
    // This function is kept for compatibility but should not be called directly
    false
}

/// Handles key events on the spell list (inside a spellbook).
fn handle_spell_list(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    ui.copy_feedback = None;

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
            ui.mode = Mode::BrowseSpellbooks;
            ui.spell_list_state.select(None);
            false
        }
        KeyCode::Down => {
            let current = ui.spell_list_state.selected().unwrap_or(0);
            let next = if current >= spell_count - 1 {
                0
            } else {
                current + 1
            };
            ui.spell_list_state.select(Some(next));
            false
        }
        KeyCode::Up => {
            let current = ui.spell_list_state.selected().unwrap_or(0);
            let prev = if current == 0 {
                spell_count - 1
            } else {
                current - 1
            };
            ui.spell_list_state.select(Some(prev));
            false
        }
        // Copy or execute selected spell
        KeyCode::Enter => {
            if modifiers.contains(KeyModifiers::ALT) {
                log_info!("Alt+Enter detected in spell list - executing spell");
                execute_spell_at_index(
                    state,
                    ui,
                    spellbook_index,
                    ui.spell_list_state.selected().unwrap_or(0),
                );
            } else {
                copy_spell_at_index(
                    state,
                    ui,
                    spellbook_index,
                    ui.spell_list_state.selected().unwrap_or(0),
                );
            }
            false
        }

        // Open search overlay
        KeyCode::Char('/') => {
            ui.open_search();
            false
        }

        // Toggle favorite
        KeyCode::Char('f') | KeyCode::Char('F') => {
            let spell_index = ui.spell_list_state.selected().unwrap_or(0);
            if let Some(spell_id) = spellbook.spell_ids.get(spell_index) {
                if let Some(spell) = state.codex.spells.iter_mut().find(|s| s.id == *spell_id) {
                    spell.favorite = !spell.favorite;
                    let status = if spell.favorite {
                        "added to"
                    } else {
                        "removed from"
                    };
                    ui.copy_feedback = Some(format!("Spell {} favorites", status));
                }
            }
            false
        }

        _ => false,
    }
}

/// Handles key events in the search overlay.
fn handle_search(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    ui.copy_feedback = None;

    // Close search on Escape - handle different modes
    if key == KeyCode::Esc {
        match ui.search_mode {
            SearchMode::BrowseSpellbooks => {
                ui.search.clear();
                ui.set_showing_spellbooks(true);
                ui.set_search_spellbook_index(Some(0));
                ui.set_search_in_command_mode(false);
                ui.exit_typing_mode();
            }
            SearchMode::BrowseSpells => {
                ui.search_mode = SearchMode::BrowseSpellbooks;
                ui.selected_spellbook = None;
            }
            SearchMode::AddSpell => {
                ui.search_mode = SearchMode::BrowseSpellbooks;
                ui.add_spell.clear();
                ui.exit_typing_mode();
            }
            SearchMode::AddSpellbook => {
                ui.search_mode = SearchMode::BrowseSpellbooks;
                ui.add_spellbook.clear();
                ui.exit_typing_mode();
            }
        }
        return false;
    }

    // Handle spell list navigation in BrowseSpells mode
    if ui.search_mode == SearchMode::BrowseSpells {
        log_info!("BrowseSpells mode handler");
        let spellbook_index = match ui.selected_spellbook {
            Some(index) => index,
            None => return false,
        };
        let spell_count = get_spell_count_for_spellbook(state, spellbook_index);
        log_info!(
            "spell_count: {}, selected: {:?}",
            spell_count,
            ui.spell_list_state.selected()
        );

        // Enter - copy or execute the selected spell
        if key == KeyCode::Enter && spell_count > 0 {
            if modifiers.contains(KeyModifiers::ALT) {
                log_info!("Alt+Enter detected in BrowseSpells mode - executing spell");
                let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
                log_info!(
                    "Calling execute_spell_at_index with book_idx={}, spell_idx={}",
                    spellbook_index,
                    spell_idx
                );
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

        // Navigate spell list with wrapping (arrow keys only)
        if key == KeyCode::Down {
            if spell_count > 0 {
                let current = ui.spell_list_state.selected().unwrap_or(0);
                let next = if current >= spell_count - 1 {
                    0
                } else {
                    current + 1
                };
                ui.spell_list_state.select(Some(next));
            }
            return false;
        }

        if key == KeyCode::Up {
            if spell_count > 0 {
                let current = ui.spell_list_state.selected().unwrap_or(0);
                let prev = if current == 0 {
                    spell_count - 1
                } else {
                    current - 1
                };
                ui.spell_list_state.select(Some(prev));
            }
            return false;
        }

        // Left - return to spellbook browsing mode
        if key == KeyCode::Left {
            ui.search_mode = SearchMode::BrowseSpellbooks;
            ui.selected_spellbook = None;
            return false;
        }

        // 'e' key - edit the selected spell
        if key == KeyCode::Char('e') && spell_count > 0 {
            let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
            if let Some((spell_id, _)) = get_spell_at_index(state, spellbook_index, spell_idx) {
                if let Some(spell) = state.get_spell(&spell_id) {
                    ui.add_spell.start_edit(spell, Some(spellbook_index));
                    ui.search_mode = SearchMode::AddSpell;
                    ui.mode = Mode::EditSpell;
                    log_info!("Editing spell: {}", spell.name);
                    return false;
                }
            }
        }

        // 'd' key - delete the selected spell (with confirmation)
        if key == KeyCode::Char('d') && spell_count > 0 {
            let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
            if let Some((spell_id, _spell_name)) =
                get_spell_at_index(state, spellbook_index, spell_idx)
            {
                if let Some(spell) = state.get_spell(&spell_id) {
                    ui.confirm_dialog = Some(crate::ui::confirm::ConfirmDialogState::delete_spell(spell.clone()));
                    ui.push_overlay(Overlay::ConfirmDialog);
                    log_info!("Delete confirmation requested for: {}", spell_id);
                    return false;
                }
            }
        }

        // Right - page down through spell list
        if key == KeyCode::Right {
            if spell_count > 0 {
                let page_size = 10;
                let current = ui.spell_list_state.selected().unwrap_or(0);
                let next = if current + page_size >= spell_count {
                    0
                } else {
                    current + page_size
                };
                ui.spell_list_state.select(Some(next));
            }
            return false;
        }

        // 's' - simple execution (exit TUI and run via exec)
        if key == KeyCode::Char('s') && !modifiers.contains(KeyModifiers::CONTROL) && spell_count > 0 {
            let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
            if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_idx) {
                if spell.confirm {
                    // Show confirmation dialog first
                    ui.confirm_dialog = Some(crate::ui::confirm::ConfirmDialogState::execute_spell(spell.clone()));
                    ui.push_overlay(Overlay::ConfirmDialog);
                    return false;
                }
                
                // Execute in simple mode
                execute_simple_mode(&spell, state, ui);
            }
            return false;
        }

        // Ctrl+r - TUI execution with streaming
        if key == KeyCode::Char('r') && modifiers.contains(KeyModifiers::CONTROL) && spell_count > 0 {
            let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
            if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_idx) {
                log_info!("Ctrl+r: Executing spell '{}' in TUI mode with streaming", spell.name);
                state.add_recent(
                    spell.id.clone(),
                    spell.name.clone(),
                    crate::models::RecentAction::Run,
                );
                let working_dir = if spell.working_dir.is_empty() {
                    None
                } else {
                    Some(spell.working_dir.clone())
                };
                if let Err(e) = streaming_modal::start_tui_execution(
                    ui,
                    spell.incantation.clone(),
                    Some(spell.name.clone()),
                    working_dir,
                ) {
                    ui.copy_feedback = Some(format!("Failed to start TUI mode: {}", e));
                }
            }
            return false;
        }

        // Ctrl+b - background execution
        if key == KeyCode::Char('b') && modifiers.contains(KeyModifiers::CONTROL) && spell_count > 0 {
            let spell_idx = ui.spell_list_state.selected().unwrap_or(0);
            if let Some(spell) = get_spell_by_index(state, spellbook_index, spell_idx) {
                log_info!("Ctrl+b: Starting spell '{}' in background", spell.name);
                match crate::invoker::start_spell(
                    spell.name.clone(),
                    spell.incantation.clone(),
                    if spell.working_dir.is_empty() {
                        None
                    } else {
                        Some(spell.working_dir.clone())
                    },
                ) {
                    Ok(job_id) => {
                        ui.copy_feedback = Some(format!("Job {} started: {}", job_id, spell.name));
                        state.add_recent(
                            spell.id.clone(),
                            spell.name.clone(),
                            crate::models::RecentAction::Run,
                        );
                    }
                    Err(e) => {
                        ui.copy_feedback = Some(format!("Failed to start: {}", e));
                    }
                }
            }
            return false;
        }

        return false;
    }

    // Handle spellbook browser navigation
    if ui.search_query().is_empty() && ui.showing_spellbooks() {
        let spellbook_count = total_spellbook_count(state);

        // Enter opens the selected spellbook - stay in search overlay
        if key == KeyCode::Enter {
            if let Some(idx) = ui.search_spellbook_index() {
                if idx < spellbook_count {
                    ui.selected_spellbook = Some(idx);
                    ui.spell_list_state.select(Some(0));
                    ui.search_mode = SearchMode::BrowseSpells;
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

        // Calculate grid offset (must match render_spellbook_cards)
        let card_unit = card_width + card_gap;
        let total_grid_width = cards_per_row * card_unit - card_gap;
        let available_width = 80; // Approximate - actual width varies
        let grid_offset = ((available_width as i32 - total_grid_width as i32) / 2).max(0) as u16;

        // Navigate with nearest-neighbor in cards view (arrow keys only)
        if key == KeyCode::Right {
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

        if key == KeyCode::Left {
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

        if key == KeyCode::Down {
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

                let visible_rows =
                    (spellbook_count.saturating_sub(scroll) + cards_per_row - 1) / cards_per_row;
                if next >= scroll + visible_rows * cards_per_row {
                    ui.set_search_spellbook_scroll(scroll + cards_per_row);
                }
            }
            return false;
        }

        if key == KeyCode::Up {
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

        // Shift+D - Delete spellbook
        if key == KeyCode::Char('D') {
            if let Some(idx) = ui.search_spellbook_index() {
                if idx < spellbook_count {
                    use crate::ui::search_overlay::get_spellbook_item;
                    if let Some(item) = get_spellbook_item(state, idx) {
                        // Don't allow deleting virtual spellbooks
                        if item.is_virtual() {
                            ui.copy_feedback = Some("Cannot delete virtual spellbooks".to_string());
                            return false;
                        }
                        let name = item.name();
                        ui.confirm_dialog = Some(crate::ui::confirm::ConfirmDialogState::delete_spellbook(name));
                        ui.push_overlay(Overlay::ConfirmDialog);
                    }
                }
            }
            return false;
        }

        // Any character input - filter based on current context
        if let KeyCode::Char(c) = key {
            ui.search_query_mut().push(c);
            ui.set_search_in_command_mode(ui.search_query().starts_with(':'));
            ui.search.search_active = true;
            if ui.search_in_command_mode() {
                update_command_filter(ui);
            } else if ui.showing_spellbooks() {
                update_spellbook_filter(state, ui);
            } else {
                update_search_filter(state, ui);
            }
            return false;
        }

        return false;
    }

    // Search mode (when there's a query or we've switched to it)

    // Check if we're in command mode (query starts with :)
    let is_command_mode = ui.search_query().starts_with(':');

    // Enter - execute command if in command mode, otherwise copy result
    if key == KeyCode::Enter {
        // Check for Alt+Shift+Enter to run as background job
        if modifiers.contains(KeyModifiers::ALT) && modifiers.contains(KeyModifiers::SHIFT) {
            log_info!("Alt+Shift+Enter detected - executing spell as background job");
            execute_search_result(state, ui, true);
            return false;
        }

        // Check for Alt+Enter to execute
        if modifiers.contains(KeyModifiers::ALT) {
            log_info!("Alt+Enter detected - executing spell");
            execute_search_result(state, ui, false);
            return false;
        }

        if is_command_mode {
            // Execute the selected command from the filtered list
            let query = ui.search_query().to_string();
            let query_after_colon = query.strip_prefix(':').unwrap_or("");
            let filtered = filter_commands(query_after_colon);
            let selected = ui.search_results_state().selected().unwrap_or(0);
            if let Some((cmd_idx, _, _)) = filtered.get(selected) {
                execute_command_by_index(*cmd_idx, state, ui);
                ui.search.clear();
                ui.set_showing_spellbooks(true);
                ui.set_search_in_command_mode(false);
                return false;
            }
            // If no selection matches, try to execute by exact match
            let cmd = query_after_colon.trim().to_string();
            if !cmd.is_empty() {
                execute_command_legacy(&cmd, state, ui);
                ui.search.clear();
                ui.set_showing_spellbooks(true);
                ui.set_search_in_command_mode(false);
            }
            return false;
        }
        // Not a command - copy result as before
        if !is_command_mode {
            copy_search_result(state, ui);
        }
        return false;
    }

    // Navigate command list or search results (arrow keys only)
    if key == KeyCode::Down {
        if is_command_mode {
            let query_after_colon = &ui.search_query()[1..];
            let filtered = filter_commands(query_after_colon);
            let count = filtered.len();
            if count > 0 {
                let current = ui.search_results_state().selected().unwrap_or(0);
                let next = if current >= count - 1 { 0 } else { current + 1 };
                ui.search_results_state().select(Some(next));
            }
        } else {
            let count = ui.filtered_indices().len();
            if count > 0 {
                let current = ui.search_results_state().selected().unwrap_or(0);
                let next = if current >= count - 1 { 0 } else { current + 1 };
                ui.search_results_state().select(Some(next));
            }
        }
        return false;
    }

    if key == KeyCode::Up {
        if is_command_mode {
            let query_after_colon = &ui.search_query()[1..];
            let filtered = filter_commands(query_after_colon);
            let count = filtered.len();
            if count > 0 {
                let current = ui.search_results_state().selected().unwrap_or(0);
                let prev = if current == 0 { count - 1 } else { current - 1 };
                ui.search_results_state().select(Some(prev));
            }
        } else {
            let count = ui.filtered_indices().len();
            if count > 0 {
                let current = ui.search_results_state().selected().unwrap_or(0);
                let prev = if current == 0 { count - 1 } else { current - 1 };
                ui.search_results_state().select(Some(prev));
            }
        }
        return false;
    }

    // Handle character input for search/query
    if let KeyCode::Char(c) = key {
        ui.search_query_mut().push(c);
        ui.set_search_in_command_mode(ui.search_query().starts_with(':'));
        if ui.search_in_command_mode() {
            update_command_filter(ui);
        } else {
            update_search_filter(state, ui);
        }
        return false;
    }

    // Handle backspace
    if key == KeyCode::Backspace {
        ui.search_query_mut().pop();
        ui.set_search_in_command_mode(ui.search_query().starts_with(':'));
        if ui.search_query().is_empty() {
            ui.search.clear();
            ui.set_showing_spellbooks(true);
            ui.set_search_spellbook_index(Some(0));
            ui.set_search_in_command_mode(false);
        } else if ui.search_in_command_mode() {
            update_command_filter(ui);
        } else if ui.showing_spellbooks() {
            update_spellbook_filter(state, ui);
        } else {
            update_search_filter(state, ui);
        }
        return false;
    }

    false
}

/// Filters all spells by the current search query.
/// Searches across spell name, lore, and glyphs (case-insensitive).
fn update_search_filter(state: &State, ui: &mut UiState) {
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

    // Select first result if we have matches
    if !ui.filtered_indices().is_empty() {
        ui.search_results_state().select(Some(0));
    } else {
        ui.search_results_state().select(None);
    }
}

/// Filters spellbooks by the current search query.
/// Searches across spellbook name (case-insensitive).
fn update_spellbook_filter(state: &State, ui: &mut UiState) {
    let query = ui.search_query().to_lowercase();

    if query.is_empty() {
        ui.filtered_spellbook_indices_mut().clear();
        return;
    }

    use crate::ui::search_overlay::{get_spellbook_item, total_spellbook_count};
    let total = total_spellbook_count(state);

    let indices: Vec<usize> = (0..total)
        .filter(|&idx| {
            if let Some(item) = get_spellbook_item(state, idx) {
                let name: String = item.name();
                name.to_lowercase().contains(&query)
            } else {
                false
            }
        })
        .collect();

    *ui.filtered_spellbook_indices_mut() = indices;

    if !ui.filtered_spellbook_indices().is_empty() {
        ui.set_search_spellbook_index(Some(0));
    }
}

/// Copies the selected search result to the clipboard.
fn copy_search_result(state: &mut State, ui: &mut UiState) {
    let selected_idx = match ui.search_results_state().selected() {
        Some(i) => i,
        None => return,
    };

    let spell_idx = match ui.filtered_indices().get(selected_idx) {
        Some(&i) => i,
        None => return,
    };

    let spell = match state.codex.spells.get(spell_idx) {
        Some(s) => s,
        None => return,
    };

    if clipboard::copy_to_clipboard(&spell.incantation) {
        ui.copy_feedback = Some("Copied! Paste into terminal to run.".to_string());
        state.add_recent(
            spell.id.clone(),
            spell.name.clone(),
            crate::models::RecentAction::Copy,
        );
    } else {
        ui.copy_feedback = Some("Copy failed".to_string());
    }
}

/// Executes a spell at a specific index in a spellbook.
fn execute_spell_at_index(
    state: &mut State,
    ui: &mut UiState,
    spellbook_index: usize,
    spell_index: usize,
) {
    log_info!(
        "execute_spell_at_index called - book_idx: {}, spell_idx: {}",
        spellbook_index,
        spell_index
    );

    let spell = get_spell_by_index(state, spellbook_index, spell_index);
    if let Some(spell) = spell {
        start_spell_execution(&spell, state, ui, false);
    }
}

/// Gets the spell count for a specific spellbook (handles virtual spellbooks).
fn get_spell_count_for_spellbook(state: &State, spellbook_index: usize) -> usize {
    let favorites_count = state.codex.spells.iter().filter(|s| s.favorite).count();
    let has_favorites = favorites_count > 0;
    let has_recent = !state.recents.is_empty();

    if has_favorites && spellbook_index == 0 {
        return favorites_count;
    }

    if has_recent {
        let recent_idx = if has_favorites { 1 } else { 0 };
        if spellbook_index == recent_idx {
            return state.recents.len();
        }
    }

    let real_idx = if has_favorites && spellbook_index > 1 {
        spellbook_index - 2
    } else if has_recent && !has_favorites && spellbook_index > 0 {
        spellbook_index - 1
    } else {
        spellbook_index
    };

    state
        .codex
        .spellbooks
        .get(real_idx)
        .map(|sb| sb.spell_ids.len())
        .unwrap_or(0)
}

/// Gets the full Spell object at a specific index in a spellbook (handles virtual spellbooks).
fn get_spell_by_index(
    state: &State,
    spellbook_index: usize,
    spell_index: usize,
) -> Option<crate::models::Spell> {
    let favorites_count = state.codex.spells.iter().filter(|s| s.favorite).count();
    let has_favorites = favorites_count > 0;
    let has_recent = !state.recents.is_empty();

    if has_favorites && spellbook_index == 0 {
        let fav_spells: Vec<_> = state.codex.spells.iter().filter(|s| s.favorite).collect();
        return fav_spells.get(spell_index).map(|spell| {
            let s: &crate::models::Spell = spell;
            s.clone()
        });
    }

    if has_recent {
        let recent_idx = if has_favorites { 1 } else { 0 };
        if spellbook_index == recent_idx {
            if let Some(recent) = state.recents.get(spell_index) {
                return state.codex.spells.iter().find(|s| s.id == recent.spell_id).map(|s| s.clone());
            }
            return None;
        }
    }

    let real_idx = if has_favorites && spellbook_index > 1 {
        spellbook_index - 2
    } else if has_recent && !has_favorites && spellbook_index > 0 {
        spellbook_index - 1
    } else {
        spellbook_index
    };

    let spellbook = state.codex.spellbooks.get(real_idx)?;
    let spell_id = spellbook.spell_ids.get(spell_index)?;
    state.codex.spells.iter().find(|s| s.id == *spell_id).map(|s| s.clone())
}

/// Gets spell info at a specific index in a spellbook. Returns (spell_id, spell_name).
fn get_spell_at_index(
    state: &State,
    spellbook_index: usize,
    spell_index: usize,
) -> Option<(String, String)> {
    let favorites_count = state.codex.spells.iter().filter(|s| s.favorite).count();
    let has_favorites = favorites_count > 0;
    let has_recent = !state.recents.is_empty();

    if has_favorites && spellbook_index == 0 {
        let fav_spells: Vec<_> = state.codex.spells.iter().filter(|s| s.favorite).collect();
        if let Some(spell) = fav_spells.get(spell_index) {
            return Some((spell.id.clone(), spell.name.clone()));
        }
        return None;
    }

    if has_recent {
        let recent_idx = if has_favorites { 1 } else { 0 };
        if spellbook_index == recent_idx {
            if let Some(recent) = state.recents.get(spell_index) {
                if let Some(spell) = state.codex.spells.iter().find(|s| s.id == recent.spell_id) {
                    return Some((spell.id.clone(), spell.name.clone()));
                }
            }
            return None;
        }
    }

    let real_idx = if has_favorites && spellbook_index > 1 {
        spellbook_index - 2
    } else if has_recent && !has_favorites && spellbook_index > 0 {
        spellbook_index - 1
    } else {
        spellbook_index
    };

    let spellbook = state.codex.spellbooks.get(real_idx)?;

    let spell_id = spellbook.spell_ids.get(spell_index)?;

    let spell = state.codex.spells.iter().find(|s| s.id == *spell_id)?;

    Some((spell.id.clone(), spell.name.clone()))
}

/// Copies a spell at a specific index in a spellbook.
fn copy_spell_at_index(
    state: &mut State,
    ui: &mut UiState,
    spellbook_index: usize,
    spell_index: usize,
) {
    let (spell_id, spell_name) = match get_spell_at_index(state, spellbook_index, spell_index) {
        Some(info) => info,
        None => return,
    };

    let spell = match state.codex.spells.iter().find(|s| s.id == spell_id) {
        Some(s) => s,
        None => return,
    };

    if clipboard::copy_to_clipboard(&spell.incantation) {
        ui.copy_feedback = Some("Copied! Paste into terminal to run.".to_string());
        state.add_recent(
            spell.id.clone(),
            spell.name.clone(),
            crate::models::RecentAction::Copy,
        );
    } else {
        ui.copy_feedback = Some("Copy failed".to_string());
    }
}

/// Executes the selected search result.
fn execute_search_result(state: &mut State, ui: &mut UiState, force_background: bool) {
    log_info!(
        "execute_search_result called, force_background={}",
        force_background
    );
    log_info!(
        "search_results_state.selected: {:?}",
        ui.search_results_state().selected()
    );
    log_info!("filtered_indices count: {}", ui.filtered_indices().len());
    let selected_idx = match ui.search_results_state().selected() {
        Some(i) => i,
        None => return,
    };

    let spell_idx = match ui.filtered_indices().get(selected_idx) {
        Some(&i) => i,
        None => return,
    };

    let spell = match state.codex.spells.get(spell_idx) {
        Some(s) => s.clone(),
        None => return,
    };

    start_spell_execution(&spell, state, ui, force_background);
}

/// Starts spell execution, showing confirmation for confirm=true spells.
fn start_spell_execution(
    spell: &crate::models::Spell,
    state: &mut State,
    ui: &mut UiState,
    force_background: bool,
) {
    use crate::models::RunMode;

    let run_mode = if force_background {
        RunMode::Background
    } else {
        spell.run_mode
    };

    if spell.confirm {
        ui.confirm_dialog = Some(crate::ui::confirm::ConfirmDialogState::execute_spell(spell.clone()));
        ui.push_overlay(Overlay::ConfirmDialog);
        return;
    }

    match run_mode {
        RunMode::Simple => {
            // Execute in simple mode (writes recents then exec())
            execute_simple_mode(spell, state, ui);
            // NOTE: On Unix, execute_simple_mode never returns (exec replaces process)
            // On non-Unix, it returns after command completion
        }
        RunMode::Tui => {
            // Start TUI streaming execution
            let working_dir = if spell.working_dir.is_empty() {
                None
            } else {
                Some(spell.working_dir.clone())
            };
            
            match streaming_modal::start_tui_execution(
                ui,
                spell.incantation.clone(),
                Some(spell.name.clone()),
                working_dir,
            ) {
                Ok(_pid) => {
                    state.add_recent(
                        spell.id.clone(),
                        spell.name.clone(),
                        crate::models::RecentAction::Run,
                    );
                }
                Err(e) => {
                    ui.copy_feedback = Some(format!("Failed to start TUI mode: {}", e));
                }
            }
        }
        RunMode::Background => {
            match crate::invoker::start_spell(
                spell.name.clone(),
                spell.incantation.clone(),
                if spell.working_dir.is_empty() {
                    None
                } else {
                    Some(spell.working_dir.clone())
                },
            ) {
                Ok(job_id) => {
                    ui.copy_feedback = Some(format!("Job {} started: {}", job_id, spell.name));
                    state.add_recent(
                        spell.id.clone(),
                        spell.name.clone(),
                        crate::models::RecentAction::Run,
                    );
                }
                Err(e) => {
                    ui.copy_feedback = Some(format!("Failed to start: {}", e));
                }
            }
        }
    }
}

/// Handles key events in the output popup.
fn handle_output_popup(key: KeyCode, state: &mut State, ui: &mut UiState) {
    match key {
        KeyCode::Esc => {
            ui.hide_output_popup();
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if let Some(ref result) = ui.output_popup {
                let filename = format!(
                    "spellbook_output_{}.txt",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                );
                match result.save_to_file(&filename) {
                    Ok(_) => {
                        ui.copy_feedback = Some(format!("Saved to {}", filename));
                    }
                    Err(e) => {
                        ui.copy_feedback = Some(format!("Save failed: {}", e));
                    }
                }
            }
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            if let Some(ref result) = ui.output_popup {
                // Look up spell to get working_dir
                let working_dir = if let Some(ref spell_name) = result.spell_name {
                    state
                        .codex
                        .spells
                        .iter()
                        .find(|s| s.name == *spell_name)
                        .map(|s| s.working_dir.clone())
                        .unwrap_or_default()
                } else {
                    String::new()
                };

                // Kill the running process
                if let Some(pid) = result.pid {
                    if let Err(e) = crate::invoker::kill_process(pid) {
                        log_warn!("Failed to kill process {}: {}", pid, e);
                    }
                }

                // Restart as background job
                let working_dir_opt = if working_dir.is_empty() {
                    None
                } else {
                    Some(working_dir)
                };

                match crate::invoker::start_spell(
                    result
                        .spell_name
                        .clone()
                        .unwrap_or_else(|| "Command".to_string()),
                    result.command.clone(),
                    working_dir_opt,
                ) {
                    Ok(_) => {
                        ui.copy_feedback = Some("Moved to background. Check with :j".to_string());
                    }
                    Err(e) => {
                        ui.copy_feedback = Some(format!("Failed to start background: {}", e));
                    }
                }
                ui.hide_output_popup();
            }
        }
        _ => {
            ui.hide_output_popup();
        }
    }
}

fn handle_input_popup(key: KeyCode, _state: &mut State, ui: &mut UiState) -> bool {
    let input_state = match ui.input_popup.as_mut() {
        Some(s) => s,
        None => return false,
    };

    match key {
        KeyCode::Esc => {
            ui.input_popup = None;
            true // Close on Esc
        }
        KeyCode::Char(c) => {
            if let Some(placeholder) = input_state
                .placeholders
                .get_mut(input_state.placeholder_index)
            {
                placeholder.value.push(c);
            }
            false
        }
        KeyCode::Backspace => {
            if let Some(placeholder) = input_state
                .placeholders
                .get_mut(input_state.placeholder_index)
            {
                placeholder.value.pop();
            }
            false
        }
        KeyCode::Enter => {
            let resolved = input_state.substitute();
            let spell = input_state.spell.clone();
            let run_background = input_state.run_background;

            ui.input_popup = None;

            if let Some(spell) = spell {
                if run_background {
                    match crate::invoker::start_spell(
                        spell.name.clone(),
                        resolved.clone(),
                        if spell.working_dir.is_empty() {
                            None
                        } else {
                            Some(spell.working_dir.clone())
                        },
                    ) {
                        Ok(job_id) => {
                            ui.copy_feedback =
                                Some(format!("Job {} started: {}", job_id, spell.name));
                        }
                        Err(e) => {
                            ui.copy_feedback = Some(format!("Failed to start: {}", e));
                        }
                    }
                } else {
                    let result = crate::invoker::execute_sync(&resolved);

                    let exec_result = crate::clipboard::ExecutionResult {
                        command: resolved.clone(),
                        stdout: result.stdout.clone(),
                        stderr: result.stderr.clone(),
                        exit_code: result.exit_code,
                        full_stdout: result.stdout,
                        full_stderr: result.stderr,
                        pid: Some(result.pid),
                        spell_name: Some(spell.name.clone()),
                    };
                    ui.show_output_popup(exec_result);
                }
            }
            true // Close after Enter
        }
        _ => false,
    }
}

fn handle_confirm_dialog(key: KeyCode, state: &mut State, ui: &mut UiState) -> bool {
    let dialog = match ui.confirm_dialog.clone() {
        Some(d) => d,
        None => return false,
    };

    match key {
        KeyCode::Esc => {
            ui.confirm_dialog = None;
            ui.pop_overlay();
            false
        }
        KeyCode::Enter => {
            let action = dialog.action.clone();
            let typed_ok = dialog.typed_confirmation.to_uppercase() == dialog.confirmation_word().to_uppercase();
            
            if dialog.requires_typed_confirmation() && !typed_ok {
                return false;
            }
            
            ui.confirm_dialog = None;
            ui.pop_overlay();

            match action {
                crate::ui::confirm::ConfirmAction::DeleteSpell(spell) => {
                    match state.delete_spell(&spell.id) {
                        Ok(_) => {
                            ui.copy_feedback = Some(format!("Deleted: {}", spell.name));
                            ui.spell_list_state.select(Some(0));
                        }
                        Err(e) => {
                            ui.copy_feedback = Some(format!("Delete failed: {}", e));
                        }
                    }
                }
                crate::ui::confirm::ConfirmAction::DeleteSpellbook(name) => {
                    match state.delete_spellbook(&name) {
                        Ok(_) => {
                            ui.copy_feedback = Some(format!("Deleted spellbook: {}", name));
                        }
                        Err(e) => {
                            ui.copy_feedback = Some(format!("Delete failed: {}", e));
                        }
                    }
                }
                crate::ui::confirm::ConfirmAction::ExecuteSpell(spell) => {
                    match spell.run_mode {
                        crate::models::RunMode::Simple => {
                            // Execute in simple mode (writes recents then exec())
                            execute_simple_mode(&spell, state, ui);
                            // NOTE: On Unix, execute_simple_mode never returns (exec replaces process)
                            // On non-Unix, it returns after command completion
                        }
                        crate::models::RunMode::Tui => {
                            // Start TUI streaming execution
                            let working_dir = if spell.working_dir.is_empty() {
                                None
                            } else {
                                Some(spell.working_dir.clone())
                            };
                            
                            if let Err(e) = streaming_modal::start_tui_execution(
                                ui,
                                spell.incantation.clone(),
                                Some(spell.name.clone()),
                                working_dir,
                            ) {
                                ui.copy_feedback = Some(format!("Failed to start TUI mode: {}", e));
                            }
                        }
                        crate::models::RunMode::Background => {
                            match crate::invoker::start_spell(
                                spell.name.clone(),
                                spell.incantation.clone(),
                                if spell.working_dir.is_empty() {
                                    None
                                } else {
                                    Some(spell.working_dir.clone())
                                },
                            ) {
                                Ok(job_id) => {
                                    ui.copy_feedback = Some(format!("Job {} started: {}", job_id, spell.name));
                                }
                                Err(e) => {
                                    ui.copy_feedback = Some(format!("Failed to start: {}", e));
                                }
                            }
                        }
                    }
                }
            }
            false
        }
        KeyCode::Char(c) => {
            if let Some(ref mut d) = ui.confirm_dialog {
                d.typed_confirmation.push(c);
            }
            false
        }
        KeyCode::Backspace => {
            if let Some(ref mut d) = ui.confirm_dialog {
                d.typed_confirmation.pop();
            }
            false
        }
        _ => false,
    }
}

/// Saves the current spell and returns to the spellbook list.
fn save_spell(state: &mut State, ui: &mut UiState) {
    if ui.add_spell.name.trim().is_empty() {
        ui.add_spell.message = Some(("Name is required".to_string(), true));
        return;
    }
    if ui.add_spell.command.trim().is_empty() {
        ui.add_spell.message = Some(("Command is required".to_string(), true));
        return;
    }

    let tags: Vec<String> = ui
        .add_spell
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let is_editing = ui.add_spell.is_editing();

    let spell = crate::models::Spell {
        id: if is_editing {
            ui.add_spell.editing_spell_id.clone().unwrap_or_default()
        } else {
            uuid::Uuid::new_v4().to_string()
        },
        name: ui.add_spell.name.clone(),
        incantation: ui.add_spell.command.clone(),
        lore: ui.add_spell.lore.clone(),
        school: ui.add_spell.school.clone(),
        glyphs: tags,
        confirm: ui.add_spell.confirm,
        run_mode: ui.add_spell.run_mode,
        working_dir: ui.add_spell.working_dir.clone(),
        favorite: if is_editing {
            state
                .get_spell(ui.add_spell.editing_spell_id.as_deref().unwrap_or(""))
                .map(|s| s.favorite)
                .unwrap_or(false)
        } else {
            false
        },
    };

    let spell_name = spell.name.clone();
    if is_editing {
        ui.start_loading("Updating spell...");
        match state.update_spell(spell) {
            Ok(_) => {
                ui.stop_loading();
                ui.add_spell.message = Some(("Spell updated!".to_string(), false));
                ui.add_spell.has_unsaved = false;
                log_info!("Spell updated: {}", spell_name);
            }
            Err(e) => {
                ui.stop_loading();
                ui.add_spell.message = Some((format!("Update failed: {}", e), true));
                log_error!("Update failed: {}", e);
                return;
            }
        }
    } else {
        let spellbook_name = if ui.add_spell.skip_spellbook {
            None
        } else {
            ui.add_spell
                .spellbook_index
                .and_then(|i| state.codex.spellbooks.get(i))
                .map(|b| b.name.clone())
        };

        ui.start_loading("Saving spell...");
        match Archivist::append_spell("codex.toml", &spell, spellbook_name.as_deref()) {
            Ok(_) => {
                ui.stop_loading();
                ui.add_spell.message = Some(("Spell saved!".to_string(), false));
                ui.add_spell.has_unsaved = false;
                log_info!("Spell saved: {}", spell.name);
            }
            Err(e) => {
                ui.stop_loading();
                ui.add_spell.message = Some((format!("Save failed: {}", e), true));
                log_error!("Save failed: {}", e);
                return;
            }
        }
    }

    ui.clear_add_spell_form();
}

/// Handles key events in the Add Spell screen.
fn handle_add_spell(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    if key == KeyCode::Char('s') && modifiers.contains(KeyModifiers::CONTROL) {
        save_spell(state, ui);
        return false;
    }

    match key {
        KeyCode::Esc => {
            if ui.add_spell.field == crate::ui::AddSpellField::Spellbook
                && ui.add_spell.dropdown_open
            {
                ui.add_spell.dropdown_open = false;
            } else if ui.add_spell.has_unsaved {
                ui.add_spell.message = Some((
                    "Unsaved changes - press Esc again to discard".to_string(),
                    true,
                ));
            } else {
                ui.clear_add_spell_form();
            }
            false
        }
        KeyCode::Tab => {
            ui.add_spell.dropdown_open = false;
            ui.add_spell.field = match ui.add_spell.field {
                crate::ui::AddSpellField::Name => crate::ui::AddSpellField::Command,
                crate::ui::AddSpellField::Command => crate::ui::AddSpellField::Lore,
                crate::ui::AddSpellField::Lore => crate::ui::AddSpellField::School,
                crate::ui::AddSpellField::School => crate::ui::AddSpellField::Tags,
                crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::WorkingDir,
                crate::ui::AddSpellField::WorkingDir => crate::ui::AddSpellField::RunMode,
                crate::ui::AddSpellField::RunMode => crate::ui::AddSpellField::Confirm,
                crate::ui::AddSpellField::Confirm => crate::ui::AddSpellField::Spellbook,
                crate::ui::AddSpellField::Spellbook => crate::ui::AddSpellField::Name,
            };
            ui.update_typing_state();
            false
        }
        KeyCode::Up => {
            if ui.add_spell.field == crate::ui::AddSpellField::Spellbook {
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
                    ui.add_spell.field = crate::ui::AddSpellField::Confirm;
                    ui.update_typing_state();
                }
            } else {
                ui.add_spell.field = match ui.add_spell.field {
                    crate::ui::AddSpellField::Command => crate::ui::AddSpellField::Name,
                    crate::ui::AddSpellField::Lore => crate::ui::AddSpellField::Command,
                    crate::ui::AddSpellField::School => crate::ui::AddSpellField::Lore,
                    crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::School,
                    crate::ui::AddSpellField::WorkingDir => crate::ui::AddSpellField::Tags,
                    crate::ui::AddSpellField::RunMode => crate::ui::AddSpellField::WorkingDir,
                    crate::ui::AddSpellField::Confirm => crate::ui::AddSpellField::RunMode,
                    crate::ui::AddSpellField::Name => crate::ui::AddSpellField::Spellbook,
                    crate::ui::AddSpellField::Spellbook => crate::ui::AddSpellField::Name,
                };
                ui.update_typing_state();
            }
            false
        }
        KeyCode::Down => {
            if ui.add_spell.field == crate::ui::AddSpellField::Spellbook {
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
                ui.add_spell.field = match ui.add_spell.field {
                    crate::ui::AddSpellField::Name => crate::ui::AddSpellField::Command,
                    crate::ui::AddSpellField::Command => crate::ui::AddSpellField::Lore,
                    crate::ui::AddSpellField::Lore => crate::ui::AddSpellField::School,
                    crate::ui::AddSpellField::School => crate::ui::AddSpellField::Tags,
                    crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::WorkingDir,
                    crate::ui::AddSpellField::WorkingDir => crate::ui::AddSpellField::RunMode,
                    crate::ui::AddSpellField::RunMode => crate::ui::AddSpellField::Confirm,
                    crate::ui::AddSpellField::Confirm => crate::ui::AddSpellField::Spellbook,
                    crate::ui::AddSpellField::Spellbook => crate::ui::AddSpellField::Name,
                };
                ui.update_typing_state();
            }
            false
        }
        KeyCode::Left => {
            if ui.add_spell.field == crate::ui::AddSpellField::RunMode {
                ui.add_spell.run_mode = match ui.add_spell.run_mode {
                    crate::models::RunMode::Simple => crate::models::RunMode::Background,
                    crate::models::RunMode::Tui => crate::models::RunMode::Simple,
                    crate::models::RunMode::Background => crate::models::RunMode::Tui,
                };
            } else if ui.add_spell.field == crate::ui::AddSpellField::Confirm {
                ui.add_spell.confirm = !ui.add_spell.confirm;
            }
            false
        }
        KeyCode::Right => {
            if ui.add_spell.field == crate::ui::AddSpellField::RunMode {
                ui.add_spell.run_mode = match ui.add_spell.run_mode {
                    crate::models::RunMode::Simple => crate::models::RunMode::Tui,
                    crate::models::RunMode::Tui => crate::models::RunMode::Background,
                    crate::models::RunMode::Background => crate::models::RunMode::Simple,
                };
            } else if ui.add_spell.field == crate::ui::AddSpellField::Confirm {
                ui.add_spell.confirm = !ui.add_spell.confirm;
            }
            false
        }
        KeyCode::Enter => match ui.add_spell.field {
            crate::ui::AddSpellField::Spellbook => {
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
            _ => {
                ui.add_spell.field = match ui.add_spell.field {
                    crate::ui::AddSpellField::Name => crate::ui::AddSpellField::Command,
                    crate::ui::AddSpellField::Command => crate::ui::AddSpellField::Lore,
                    crate::ui::AddSpellField::Lore => crate::ui::AddSpellField::School,
                    crate::ui::AddSpellField::School => crate::ui::AddSpellField::Tags,
                    crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::WorkingDir,
                    crate::ui::AddSpellField::WorkingDir => crate::ui::AddSpellField::Spellbook,
                    _ => ui.add_spell.field,
                };
                ui.update_typing_state();
                false
            }
        },
        KeyCode::Backspace => {
            match ui.add_spell.field {
                crate::ui::AddSpellField::Name => {
                    ui.add_spell.name.pop();
                }
                crate::ui::AddSpellField::Command => {
                    ui.add_spell.command.pop();
                }
                crate::ui::AddSpellField::Lore => {
                    ui.add_spell.lore.pop();
                }
                crate::ui::AddSpellField::School => {
                    ui.add_spell.school.pop();
                }
                crate::ui::AddSpellField::Tags => {
                    ui.add_spell.tags.pop();
                }
                crate::ui::AddSpellField::WorkingDir => {
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
                crate::ui::AddSpellField::Name => {
                    ui.add_spell.name.push(c);
                }
                crate::ui::AddSpellField::Command => {
                    ui.add_spell.command.push(c);
                }
                crate::ui::AddSpellField::Lore => {
                    ui.add_spell.lore.push(c);
                }
                crate::ui::AddSpellField::School => {
                    ui.add_spell.school.push(c);
                }
                crate::ui::AddSpellField::Tags => {
                    ui.add_spell.tags.push(c);
                }
                crate::ui::AddSpellField::WorkingDir => {
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

/// Update filtered commands based on current query after ":"
fn update_command_filter(ui: &mut UiState) {
    let query_after_colon = ui.search_query().strip_prefix(':').unwrap_or("");
    let filtered = filter_commands(query_after_colon);
    *ui.filtered_indices_mut() = filtered.iter().map(|(idx, _, _)| *idx).collect();

    if !ui.filtered_indices().is_empty() {
        ui.search_results_state().select(Some(0));
    } else {
        ui.search_results_state().select(None);
    }
}

/// Execute a command from the search/command bar (legacy, for exact match)
/// Commands are entered without the leading ":" (e.g., "new spell" not ":new spell")
fn execute_command_legacy(cmd: &str, state: &mut State, ui: &mut UiState) {
    let cmd_lower = cmd.to_lowercase();

    match cmd_lower.as_str() {
        // Add new spell
        "n" | "new" | "new spell" | "add spell" => {
            ui.search_mode = SearchMode::AddSpell;
            ui.add_spell.field = crate::ui::AddSpellField::Name;
            ui.is_typing = true;
            log_info!("Command: new spell");
        }
        // Add new spellbook
        "new book" | "new spellbook" | "add spellbook" | "add book" => {
            ui.search_mode = SearchMode::AddSpellbook;
            ui.is_typing = true;
            log_info!("Command: new spellbook");
        }
        // Switch to browse spellbooks
        "b" | "books" | "browse" | "spellbooks" => {
            ui.search_mode = SearchMode::BrowseSpellbooks;
            ui.set_showing_spellbooks(true);
            ui.selected_spellbook = None;
            log_info!("Command: browse");
        }
        // Switch to spells in selected book
        "s" | "spells" => {
            if let Some(idx) = ui.selected_spellbook {
                ui.search_mode = SearchMode::BrowseSpells;
                ui.spell_list_state.select(Some(0));
                log_info!("Command: spells");
            } else {
                ui.copy_feedback = Some("Select a spellbook first".to_string());
            }
        }
        // View modes
        "c" | "cards" => {
            state.user_settings.view_mode = ViewMode::Cards;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            ui.view_mode = ViewMode::Cards;
            log_info!("Command: cards view");
        }
        "p" | "spines" => {
            state.user_settings.view_mode = ViewMode::Spines;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            ui.view_mode = ViewMode::Spines;
            log_info!("Command: spines view");
        }
        "l" | "list" => {
            state.user_settings.view_mode = ViewMode::List;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            ui.view_mode = ViewMode::List;
            log_info!("Command: list view");
        }
        // Theme commands
        "t" | "theme" | "next theme" => {
            state.cycle_theme();
            ui.view_mode = state.user_settings.view_mode;
            log_info!("Command: cycle theme");
        }
        // Help
        "?" | "help" | "commands" => {
            ui.push_overlay(Overlay::Help);
            log_info!("Command: help");
        }
        // Export commands
        cmd if cmd.starts_with("export") || cmd.starts_with("ex") => {
            let args = cmd.strip_prefix("export").unwrap_or(cmd).strip_prefix("ex").unwrap_or(cmd).trim();
            ui.start_loading("Exporting...");
            if args.is_empty() {
                // Export full codex
                let filename = format!("spellbook_export_{}.toml", 
                    chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                match Archivist::export_codex(&state.codex, &filename) {
                    Ok(_) => {
                        ui.stop_loading();
                        ui.copy_feedback = Some(format!("Exported codex to {}", filename));
                        log_info!("Command: export codex to {}", filename);
                    }
                    Err(e) => {
                        ui.stop_loading();
                        ui.copy_feedback = Some(format!("Export failed: {}", e));
                        log_info!("Export failed: {}", e);
                    }
                }
            } else {
                // Check if argument is a spellbook name
                let spellbook_name = args.trim();
                if let Some(_) = state.codex.spellbooks.iter().find(|sb| sb.name == spellbook_name) {
                    let filename = format!("{}_export.toml", 
                        spellbook_name.replace(' ', "_"));
                    match Archivist::export_spellbook(&state.codex, spellbook_name, &filename) {
                        Ok(_) => {
                            ui.stop_loading();
                            ui.copy_feedback = Some(format!("Exported '{}' to {}", spellbook_name, filename));
                            log_info!("Command: export spellbook '{}' to {}", spellbook_name, filename);
                        }
                        Err(e) => {
                            ui.stop_loading();
                            ui.copy_feedback = Some(format!("Export failed: {}", e));
                            log_info!("Export failed: {}", e);
                        }
                    }
                } else {
                    // Treat as filename for full codex export
                    match Archivist::export_codex(&state.codex, spellbook_name) {
                        Ok(_) => {
                            ui.stop_loading();
                            ui.copy_feedback = Some(format!("Exported codex to {}", spellbook_name));
                            log_info!("Command: export codex to {}", spellbook_name);
                        }
                        Err(e) => {
                            ui.stop_loading();
                            ui.copy_feedback = Some(format!("Export failed: {}", e));
                            log_info!("Export failed: {}", e);
                        }
                    }
                }
            }
        }
        // Import commands
        cmd if cmd.starts_with("import") || cmd.starts_with("im") => {
            let args = cmd.strip_prefix("import").unwrap_or(cmd).strip_prefix("im").unwrap_or(cmd).trim();
            if args.is_empty() {
                ui.copy_feedback = Some("Usage: :import <file>".to_string());
                log_info!("Command: import - no file specified");
            } else {
                let filename = args.trim();
                ui.start_loading("Importing...");
                match Archivist::import_codex(filename) {
                    Ok(imported) => {
                        let imported_spells = imported.spells.len();
                        let imported_spellbooks = imported.spellbooks.len();
                        
                        // Auto-merge with rename strategy for conflicts
                        let result = Archivist::merge_codex(&mut state.codex, imported, MergeStrategy::Rename);
                        
                        // Save the merged codex
                        if let Err(e) = Archivist::save(&state.codex, "codex.toml") {
                            ui.stop_loading();
                            ui.copy_feedback = Some(format!("Import succeeded but save failed: {}", e));
                            log_info!("Import save failed: {}", e);
                        } else {
                            ui.stop_loading();
                            let mut msg = format!("Imported {} spell(s), {} spellbook(s)", 
                                result.added_spells.len(), result.added_spellbooks.len());
                            if !result.conflicts.is_empty() {
                                msg.push_str(&format!(" ({} conflicts renamed)", result.conflicts.len()));
                            }
                            ui.copy_feedback = Some(msg);
                            log_info!("Command: import from {} - {} spells, {} spellbooks", 
                                filename, imported_spells, imported_spellbooks);
                        }
                    }
                    Err(e) => {
                        ui.stop_loading();
                        ui.copy_feedback = Some(format!("Import failed: {}", e));
                        log_info!("Import failed: {}", e);
                    }
                }
            }
        }
        // Unknown command
        _ => {
            ui.copy_feedback = Some(format!("Unknown command: {}", cmd));
            log_info!("Unknown command: {}", cmd);
        }
    }
}

/// Execute a spell in simple mode: write recents, then exec() the command.
/// 
/// This function NEVER RETURNS on Unix (replaces process via exec).
/// On non-Unix platforms, it runs the command and returns the exit code.
/// 
/// # Critical
/// Must write recents.toml BEFORE calling exec() because exec() replaces
/// the current process and we get no chance to write after.
fn execute_simple_mode(spell: &crate::models::Spell, state: &mut State, _ui: &mut UiState) {
    use crate::archivist::Archivist;
    
    // Step 1: Add to recents (in memory)
    state.add_recent(
        spell.id.clone(),
        spell.name.clone(),
        crate::models::RecentAction::Run,
    );
    
    // Step 2: CRITICAL - Persist recents to disk BEFORE exec()
    // After exec(), this process is replaced and we never return
    if let Err(e) = Archivist::save_recents(&state.recents) {
        log_error!("Failed to write recents before exec: {}", e);
        // Continue anyway - better to run the command than fail entirely
    } else {
        log_info!("Recents persisted before simple mode exec");
    }
    
    // Step 3: Determine working directory
    let working_dir = if spell.working_dir.is_empty() {
        None
    } else {
        Some(spell.working_dir.clone())
    };
    
    // Step 4: Restore terminal (clean shutdown)
    // Note: We need to disable raw mode before exec
    let _ = crossterm::terminal::disable_raw_mode();
    
    // Step 5: Execute via exec() - this replaces our process
    log_info!("Executing spell '{}' in simple mode: {}", spell.name, spell.incantation);
    crate::invoker::exec_simple(&spell.incantation, working_dir.as_deref());
}

/// Handles key events in the Add Spellbook screen.
fn handle_add_spellbook(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    use crate::ui::add_spellbook_form::AddSpellbookField;
    
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
                ui.mode = Mode::BrowseSpellbooks;
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
        KeyCode::Up => {
            ui.add_spellbook.prev_field();
            false
        }
        KeyCode::Down | KeyCode::Enter => {
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

/// Saves the current spellbook and returns to the spellbook list.
fn save_spellbook(_state: &mut State, ui: &mut UiState) {
    use crate::archivist::Archivist;
    
    if ui.add_spellbook.name.trim().is_empty() {
        ui.add_spellbook.message = Some(("Name is required".to_string(), true));
        return;
    }

    // Take ownership of values to pass to archivist (clean borrowck-friendly approach)
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
            ui.mode = Mode::BrowseSpellbooks;
        }
        Err(e) => {
            ui.stop_loading();
            ui.add_spellbook.message = Some((format!("Save failed: {}", e), true));
            log_error!("Save failed: {}", e);
        }
    }
}
