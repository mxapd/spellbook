use crate::clipboard;
use crate::persistence::Archivist;
use crate::state::State;
use crate::ui::search_overlay::{find_nearest_card, CardDirection};
use crate::ui::{Screen, SearchMode, UiState, ViewMode};
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
            ui.copy_feedback = Some(
                ":n new :b browse :s spells :c cards :p spines :l list :t theme :j jobs :e experimental :? help"
                    .to_string(),
            );
            log_info!("Command: help");
        }
        CommandAction::Jobs => {
            ui.screen = Screen::JobsPanel;
            ui.jobs_panel_state.selected_index = None;
            log_info!("Command: jobs");
        }
        CommandAction::Experimental => {
            state.user_settings.experimental_mode = !state.user_settings.experimental_mode;
            let _ = Archivist::save_user_settings("theme.toml", &state.user_settings);
            let status = if state.user_settings.experimental_mode { "on" } else { "off" };
            ui.copy_feedback = Some(format!("Experimental mode: {}", status));
            log_info!("Command: experimental mode {}", status);
        }
    }
}

/// Main event handler - routes key events to the appropriate screen handler.
pub fn handle_event(
    key: KeyCode,
    state: &mut State,
    ui: &mut UiState,
    modifiers: KeyModifiers,
) -> bool {
    // Handle Ctrl+C to quit
    if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        log_info!("Quit via Ctrl+C");
        return true;
    }

    // Handle Ctrl+Z - intercept but don't process (let terminal handle job control)
    // Returning false without doing anything prevents it from being added to search
    if key == KeyCode::Char('z') && modifiers.contains(KeyModifiers::CONTROL) {
        log_info!("Ctrl+Z intercepted - terminal should handle suspend");
        return false;
    }

    // Handle Alt+R to refresh/reload the codex
    if key == KeyCode::Char('r') && modifiers.contains(KeyModifiers::ALT) {
        log_info!("Alt+R detected - reloading codex");
        state.reload_codex();
        ui.copy_feedback = Some("Codex refreshed".to_string());
        ui.request_redraw();
        return false;
    }

    // Handle theme cycling with 't' - disabled while typing
    if key == KeyCode::Char('t') && !ui.is_typing {
        state.cycle_theme();
        return false;
    }

    // Handle view mode cycling with 'v' - disabled while typing
    if key == KeyCode::Char('v') && !ui.is_typing {
        state.cycle_view_mode();
        return false;
    }

    // Sync view_mode from state to ui for render functions
    ui.view_mode = state.user_settings.view_mode;

    match &ui.screen {
        Screen::SpellList => {
            log_debug!("Screen: SpellList");
            handle_spell_list(key, state, ui, modifiers)
        }
        Screen::SearchOverlay => {
            log_debug!("Screen: SearchOverlay");
            handle_search(key, state, ui, modifiers)
        }
        Screen::AddSpell => {
            log_debug!("Screen: AddSpell");
            handle_add_spell(key, state, ui, modifiers)
        }
        Screen::OutputPopup => {
            log_debug!("Screen: OutputPopup");
            handle_output_popup(key, state, ui);
            false
        }
        Screen::JobsPanel => {
            log_debug!("Screen: JobsPanel");
            crate::ui::jobs::handle_jobs_key(key, ui)
        }
        Screen::ConfirmDialog => {
            log_debug!("Screen: ConfirmDialog");
            if let Some(handled) = crate::ui::confirm::handle_confirm_key(key, None, ui, state) {
                handled
            } else {
                false
            }
        }
        Screen::InputPopup => {
            log_debug!("Screen: InputPopup");
            handle_input_popup(key, state, ui)
        }
    }
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
            ui.screen = Screen::SearchOverlay;
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
        let spellbook = &state.codex.spellbooks[spellbook_index];
        let spell_count = spellbook.spell_ids.len();
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

        return false;
    }

    // Handle spellbook browser navigation
    if ui.search_query().is_empty() && ui.showing_spellbooks() {
        let spellbook_count = state.codex.spellbooks.len();

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
                    ui.set_search_spellbook_scroll(scroll.saturating_sub(cards_per_row));
                }
            }
            return false;
        }

        // Any character input switches to search mode
        if let KeyCode::Char(c) = key {
            ui.set_showing_spellbooks(false);
            ui.search_query_mut().push(c);
            update_search_filter(state, ui);
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

    // Filter spells that match the query in name, lore, or glyphs
    let indices: Vec<usize> = state
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

    *ui.filtered_indices_mut() = indices;

    // Select first result if we have matches
    if !ui.filtered_indices().is_empty() {
        ui.search_results_state().select(Some(0));
    } else {
        ui.search_results_state().select(None);
    }
}

/// Copies the selected search result to the clipboard.
fn copy_search_result(state: &State, ui: &mut UiState) {
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
    let spellbook = match state.codex.spellbooks.get(spellbook_index) {
        Some(sb) => sb,
        None => return,
    };

    let spell_id = match spellbook.spell_ids.get(spell_index) {
        Some(id) => id,
        None => return,
    };

    let spell = match state.codex.spells.iter().find(|s| s.id == *spell_id) {
        Some(s) => s.clone(),
        None => return,
    };

    start_spell_execution(&spell, state, ui, false);
}

/// Copies a spell at a specific index in a spellbook.
fn copy_spell_at_index(
    state: &State,
    ui: &mut UiState,
    spellbook_index: usize,
    spell_index: usize,
) {
    let spellbook = match state.codex.spellbooks.get(spellbook_index) {
        Some(sb) => sb,
        None => return,
    };

    let spell_id = match spellbook.spell_ids.get(spell_index) {
        Some(id) => id,
        None => return,
    };

    let spell = match state.codex.spells.iter().find(|s| s.id == *spell_id) {
        Some(s) => s,
        None => return,
    };

    if clipboard::copy_to_clipboard(&spell.incantation) {
        ui.copy_feedback = Some("Copied! Paste into terminal to run.".to_string());
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

/// Starts spell execution, showing input popup for elevated commands with placeholders.
fn start_spell_execution(spell: &crate::models::Spell, state: &mut State, ui: &mut UiState, force_background: bool) {
    // Simplified mode (default): just copy to clipboard
    if !state.user_settings.experimental_mode {
        if crate::clipboard::copy_to_clipboard(&spell.incantation) {
            ui.copy_feedback = Some("Copied! Paste into terminal to run.".to_string());
        } else {
            ui.copy_feedback = Some("Copy failed".to_string());
        }
        return;
    }

    // Experimental mode: full elevated/background/job handling
    let should_run_background = force_background || spell.background;
    let placeholders = crate::executor::detect_placeholders(&spell.incantation);

    if spell.elevated {
        ui.previous_screen = Some(ui.screen);
        ui.screen = Screen::InputPopup;
        ui.input_popup = Some(crate::ui::input::InputPopupState::with_password(
            spell.clone(),
            placeholders,
            spell.incantation.clone(),
            should_run_background,
        ));
    } else if !placeholders.is_empty() {
        ui.previous_screen = Some(ui.screen);
        ui.screen = Screen::InputPopup;
        ui.input_popup = Some(crate::ui::input::InputPopupState::new(
            spell.clone(),
            placeholders,
            spell.incantation.clone(),
            should_run_background,
        ));
    } else if should_run_background {
        match crate::executor::start_spell(
            spell.name.clone(),
            spell.incantation.clone(),
            false,
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
    } else {
        let result = crate::executor::execute_sync(&spell.incantation, false);
        let exec_result = crate::clipboard::ExecutionResult {
            command: spell.incantation.clone(),
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
                // Look up spell to get elevated status and working_dir
                let (elevated, working_dir) = if let Some(ref spell_name) = result.spell_name {
                    state
                        .codex
                        .spells
                        .iter()
                        .find(|s| s.name == *spell_name)
                        .map(|s| (s.elevated, s.working_dir.clone()))
                        .unwrap_or((false, String::new()))
                } else {
                    (false, String::new())
                };

                // Save background preference to codex.toml and update in-memory
                if let Some(ref spell_name) = result.spell_name {
                    if let Err(e) = crate::persistence::Archivist::update_spell_background(
                        "codex.toml",
                        spell_name,
                        true,
                    ) {
                        log_error!("Failed to update spell background preference: {}", e);
                    }
                    if let Some(spell) = state
                        .codex
                        .spells
                        .iter_mut()
                        .find(|s| s.name == *spell_name)
                    {
                        spell.background = true;
                    }
                }

                // Kill the running process
                if let Some(pid) = result.pid {
                    if let Err(e) = crate::executor::kill_process(pid) {
                        log_warn!("Failed to kill process {}: {}", pid, e);
                    }
                }

                // Restart as background job
                let working_dir_opt = if working_dir.is_empty() {
                    None
                } else {
                    Some(working_dir)
                };

                if elevated {
                    match crate::executor::start_spell_with_sudo_cached(
                        result.spell_name.clone().unwrap_or_else(|| "Command".to_string()),
                        result.command.clone(),
                        working_dir_opt,
                    ) {
                        Ok(_) => {
                            ui.copy_feedback =
                                Some("Moved to background. Check with :j".to_string());
                        }
                        Err(e) => {
                            ui.copy_feedback = Some(format!(
                                "No cached sudo credentials - run spell again to re-authenticate"
                            ));
                            log_warn!(
                                "Failed to start sudo background job: {}",
                                e
                            );
                        }
                    }
                } else {
                    match crate::executor::start_spell(
                        result.spell_name.clone().unwrap_or_else(|| "Command".to_string()),
                        result.command.clone(),
                        false,
                        working_dir_opt,
                    ) {
                        Ok(_) => {
                            ui.copy_feedback =
                                Some("Moved to background. Check with :j".to_string());
                        }
                        Err(e) => {
                            ui.copy_feedback =
                                Some(format!("Failed to start background: {}", e));
                        }
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
            ui.screen = ui.previous_screen.take().unwrap_or(Screen::SearchOverlay);
            false
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if input_state.phase == crate::ui::input::InputPhase::Password {
                input_state.show_password = !input_state.show_password;
            }
            false
        }
        KeyCode::Char(c) => {
            if input_state.phase == crate::ui::input::InputPhase::Password {
                input_state.password.push(c);
            }
            false
        }
        KeyCode::Backspace => {
            if input_state.phase == crate::ui::input::InputPhase::Password {
                input_state.password.pop();
            }
            false
        }
        KeyCode::Enter => {
            if !input_state.validate() {
                ui.copy_feedback = Some("Please fill in all fields".to_string());
                return false;
            }

            match input_state.phase {
                crate::ui::input::InputPhase::Password => {
                    if input_state.placeholders.is_empty() {
                        let resolved = input_state.substitute();
                        let spell = input_state.spell.clone();
                        let password = input_state.password.clone();
                        let run_background = input_state.run_background;

                        ui.input_popup = None;
                        ui.screen = ui.previous_screen.take().unwrap_or(Screen::SearchOverlay);

                        if let Some(spell) = spell {
                            if run_background {
                                match crate::executor::start_spell_with_sudo(
                                    spell.name.clone(),
                                    resolved.clone(),
                                    password,
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
                                let result =
                                    crate::executor::execute_with_sudo(&resolved, &password);
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
                        false
                    } else {
                        input_state.phase = crate::ui::input::InputPhase::Arguments;
                        false
                    }
                }
                crate::ui::input::InputPhase::Arguments => {
                    let resolved = input_state.substitute();
                    let spell = input_state.spell.clone();
                    let elevated = spell.as_ref().map(|s| s.elevated).unwrap_or(false);
                    let password = input_state.password.clone();
                    let run_background = input_state.run_background;

                    ui.input_popup = None;
                    ui.screen = ui.previous_screen.take().unwrap_or(Screen::SearchOverlay);

                    if let Some(spell) = spell {
                        if run_background {
                            if elevated {
                                match crate::executor::start_spell_with_sudo(
                                    spell.name.clone(),
                                    resolved.clone(),
                                    password,
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
                                match crate::executor::start_spell(
                                    spell.name.clone(),
                                    resolved.clone(),
                                    false,
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
                            }
                        } else {
                            let result = if elevated {
                                crate::executor::execute_with_sudo(&resolved, &password)
                            } else {
                                crate::executor::execute_sync(&resolved, false)
                            };

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
                    false
                }
            }
        }
        _ => false,
    }
}

/// Saves the current spell and returns to the spellbook list.
fn save_spell(state: &State, ui: &mut UiState) {
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

    let spell = crate::models::Spell {
        id: 0,
        name: ui.add_spell.name.clone(),
        incantation: ui.add_spell.command.clone(),
        lore: ui.add_spell.lore.clone(),
        school: ui.add_spell.school.clone(),
        glyphs: tags,
        elevated: false,
        dangerous: false,
        confirm: false,
        working_dir: String::new(),
        background: false,
    };

    let spellbook_name = if ui.add_spell.skip_spellbook {
        None
    } else {
        ui.add_spell
            .spellbook_index
            .and_then(|i| state.codex.spellbooks.get(i))
            .map(|b| b.name.clone())
    };

    match Archivist::append_spell("codex.toml", &spell, spellbook_name.as_deref()) {
        Ok(_) => {
            ui.add_spell.message = Some(("Spell saved!".to_string(), false));
            ui.add_spell.has_unsaved = false;
            log_info!("Spell saved: {}", spell.name);
        }
        Err(e) => {
            ui.add_spell.message = Some((format!("Save failed: {}", e), true));
            log_error!("Save failed: {}", e);
            return;
        }
    }

    ui.clear_add_spell_form();
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
                crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::Spellbook,
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
                    ui.add_spell.field = crate::ui::AddSpellField::Tags;
                    ui.update_typing_state();
                }
            } else {
                ui.add_spell.field = match ui.add_spell.field {
                    crate::ui::AddSpellField::Command => crate::ui::AddSpellField::Name,
                    crate::ui::AddSpellField::Lore => crate::ui::AddSpellField::Command,
                    crate::ui::AddSpellField::School => crate::ui::AddSpellField::Lore,
                    crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::School,
                    crate::ui::AddSpellField::Spellbook => crate::ui::AddSpellField::Tags,
                    crate::ui::AddSpellField::Name => crate::ui::AddSpellField::Spellbook,
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
                    crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::Spellbook,
                    _ => ui.add_spell.field,
                };
                ui.update_typing_state();
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
                    crate::ui::AddSpellField::Tags => crate::ui::AddSpellField::Spellbook,
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
            ui.copy_feedback = Some("Commands: :n/new spell, :N/new book, :b/browse, :s/spells, :c/cards, :p/spines, :l/list, :t/theme".to_string());
            log_info!("Command: help");
        }
        // Unknown command
        _ => {
            ui.copy_feedback = Some(format!("Unknown command: {}", cmd));
            log_info!("Unknown command: {}", cmd);
        }
    }
}
