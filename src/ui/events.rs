use crate::clipboard;
use crate::persistence::Archivist;
use crate::state::State;
use crate::ui::{AddSpellField, Screen, SearchContext, SearchMode, UiState, ViewMode};
use crate::{log_debug, log_error, log_info};
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
    Help,
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
                .any(|alias| alias.starts_with(&query_lower))
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
            ui.add_spell_field = AddSpellField::Name;
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
            ui.search_showing_spellbooks = true;
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
                ":n new :b browse :s spells :c cards :p spines :l list :t theme :? help"
                    .to_string(),
            );
            log_info!("Command: help");
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
fn handle_search(key: KeyCode, state: &mut State, ui: &mut UiState) -> bool {
    ui.copy_feedback = None;

    // Close search on Escape - handle different modes
    if key == KeyCode::Esc {
        match ui.search_mode {
            SearchMode::BrowseSpellbooks => {
                // Close search and return to previous screen
                ui.screen = match ui.search_return_to {
                    SearchContext::SpellbookList => Screen::SpellbookList,
                    SearchContext::SpellList => Screen::SpellList,
                };
                ui.exit_typing_mode();
            }
            SearchMode::BrowseSpells => {
                // Return to spellbook browsing mode
                ui.search_mode = SearchMode::BrowseSpellbooks;
                ui.selected_spellbook = None;
            }
            SearchMode::AddSpell => {
                // Cancel add spell and return to browse
                ui.search_mode = SearchMode::BrowseSpellbooks;
                ui.add_spell_name.clear();
                ui.add_spell_command.clear();
                ui.add_spell_lore.clear();
                ui.add_spell_school.clear();
                ui.add_spell_tags.clear();
                ui.add_spell_message = None;
                ui.exit_typing_mode();
            }
            SearchMode::AddSpellbook => {
                // Cancel add spellbook and return to browse
                ui.search_mode = SearchMode::BrowseSpellbooks;
                ui.add_spellbook_name.clear();
                ui.add_spellbook_cover.clear();
                ui.add_spellbook_sigil.clear();
                ui.exit_typing_mode();
            }
        }
        return false;
    }

    // Handle spell list navigation in BrowseSpells mode
    if ui.search_mode == SearchMode::BrowseSpells {
        let spellbook_index = match ui.selected_spellbook {
            Some(index) => index,
            None => return false,
        };
        let spellbook = &state.codex.spellbooks[spellbook_index];
        let spell_count = spellbook.spell_ids.len();

        // Enter - copy the selected spell
        if key == KeyCode::Enter && spell_count > 0 {
            copy_selected_spell(state, ui);
            return false;
        }

        // Navigate spell list with wrapping
        if key == KeyCode::Down || key == KeyCode::Char('k') {
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

        if key == KeyCode::Up || key == KeyCode::Char('j') {
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

        // Left (h) - return to spellbook browsing mode
        if key == KeyCode::Left || key == KeyCode::Char('h') {
            ui.search_mode = SearchMode::BrowseSpellbooks;
            ui.selected_spellbook = None;
            return false;
        }

        // Right (l) - page down through spell list
        if key == KeyCode::Right || key == KeyCode::Char('l') {
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
    if ui.search_query.is_empty() && ui.search_showing_spellbooks {
        let spellbook_count = state.codex.spellbooks.len();

        // Enter opens the selected spellbook - stay in search overlay
        if key == KeyCode::Enter {
            if let Some(idx) = ui.search_spellbook_index {
                if idx < spellbook_count {
                    ui.selected_spellbook = Some(idx);
                    ui.spell_list_state.select(Some(0));
                    ui.search_mode = SearchMode::BrowseSpells;
                    return false;
                }
            }
        }

        let spines_per_row = ui.search_items_per_row.max(1);
        let scroll = ui.search_spellbook_scroll;

        // Navigate with arrow keys - Left/Right scroll, Up/Down wrap
        if key == KeyCode::Right || key == KeyCode::Char('l') {
            if spellbook_count > 0 {
                let current = ui.search_spellbook_index.unwrap_or(0);
                let spines_per_row = ui.search_items_per_row.max(1);

                // Calculate current position
                let current_row = current / spines_per_row;
                let current_col = current % spines_per_row;
                let total_rows = (spellbook_count + spines_per_row - 1) / spines_per_row;

                // Find max column in current row (handle partial last row)
                let max_col_in_row = if current_row == total_rows - 1 {
                    let items_in_last_row = spellbook_count - (current_row * spines_per_row);
                    items_in_last_row - 1
                } else {
                    spines_per_row - 1
                };

                // Move within row, wrap to first column if at end
                let next = if current_col >= max_col_in_row {
                    current_row * spines_per_row // first column in same row
                } else {
                    current + 1
                };

                ui.search_spellbook_index = Some(next);

                // Auto-scroll to keep selection visible
                let visible_end = scroll + spines_per_row;
                if next >= visible_end {
                    ui.search_spellbook_scroll = (next + 1).saturating_sub(spines_per_row);
                }
            }
            return false;
        }

        if key == KeyCode::Left || key == KeyCode::Char('h') {
            if spellbook_count > 0 {
                let current = ui.search_spellbook_index.unwrap_or(0);
                let spines_per_row = ui.search_items_per_row.max(1);

                // Calculate current position
                let current_row = current / spines_per_row;
                let current_col = current % spines_per_row;

                // Move within row, wrap to last column if at start
                let prev = if current_col == 0 {
                    // At first column - wrap to last column in same row
                    let total_rows = (spellbook_count + spines_per_row - 1) / spines_per_row;
                    let max_col_in_row = if current_row == total_rows - 1 {
                        let items_in_last_row = spellbook_count - (current_row * spines_per_row);
                        items_in_last_row - 1
                    } else {
                        spines_per_row - 1
                    };
                    current_row * spines_per_row + max_col_in_row
                } else {
                    current - 1
                };

                ui.search_spellbook_index = Some(prev);

                // Auto-scroll to keep selection visible
                if prev < scroll {
                    ui.search_spellbook_scroll = prev;
                }
            }
            return false;
        }

        if key == KeyCode::Down || key == KeyCode::Char('j') {
            if spellbook_count > 0 {
                let current = ui.search_spellbook_index.unwrap_or(0);
                let spines_per_row = ui.search_items_per_row.max(1);

                // Calculate current position
                let current_row = current / spines_per_row;
                let current_col = current % spines_per_row;
                let total_rows = (spellbook_count + spines_per_row - 1) / spines_per_row;

                // Move to next row, wrap to first row if at end
                let next_row = if current_row + 1 >= total_rows {
                    0 // Wrap to first row
                } else {
                    current_row + 1
                };

                // Handle partial last row
                let next_index = if next_row == total_rows - 1 {
                    let items_in_last_row = spellbook_count - (next_row * spines_per_row);
                    let col = current_col.min(items_in_last_row.saturating_sub(1));
                    next_row * spines_per_row + col
                } else {
                    next_row * spines_per_row + current_col
                };

                ui.search_spellbook_index = Some(next_index);

                // Auto-scroll
                let scroll = ui.search_spellbook_scroll;
                let visible_rows =
                    (spellbook_count.saturating_sub(scroll) + spines_per_row - 1) / spines_per_row;
                if next_index >= scroll + visible_rows * spines_per_row {
                    ui.search_spellbook_scroll = scroll + spines_per_row;
                }
            }
            return false;
        }

        if key == KeyCode::Up || key == KeyCode::Char('k') {
            if spellbook_count > 0 {
                let current = ui.search_spellbook_index.unwrap_or(0);
                let spines_per_row = ui.search_items_per_row.max(1);

                // Calculate current position
                let current_row = current / spines_per_row;
                let current_col = current % spines_per_row;
                let total_rows = (spellbook_count + spines_per_row - 1) / spines_per_row;

                // Move to previous row, wrap to last row if at start
                let prev_row = if current_row == 0 {
                    total_rows - 1 // Wrap to last row
                } else {
                    current_row - 1
                };

                // Handle partial last row
                let prev_index = if prev_row == total_rows - 1 {
                    let items_in_last_row = spellbook_count - (prev_row * spines_per_row);
                    let col = current_col.min(items_in_last_row.saturating_sub(1));
                    prev_row * spines_per_row + col
                } else {
                    prev_row * spines_per_row + current_col
                };

                ui.search_spellbook_index = Some(prev_index);

                // Auto-scroll
                let scroll = ui.search_spellbook_scroll;
                if prev_index < scroll {
                    ui.search_spellbook_scroll = scroll.saturating_sub(spines_per_row);
                }
            }
            return false;
        }

        // Any character input switches to search mode
        if let KeyCode::Char(c) = key {
            ui.search_showing_spellbooks = false;
            ui.search_query.push(c);
            update_search_filter(state, ui);
            return false;
        }

        return false;
    }

    // Search mode (when there's a query or we've switched to it)

    // Check if we're in command mode (query starts with :)
    let is_command_mode = ui.search_query.starts_with(':');

    // Enter - execute command if in command mode, otherwise copy result
    if key == KeyCode::Enter {
        if is_command_mode && ui.search_query.len() > 1 {
            // Execute the selected command from filtered commands
            let query_after_colon = &ui.search_query[1..];
            let filtered = filter_commands(query_after_colon);
            if let Some(selected) = ui.search_list_state.selected() {
                if let Some((cmd_idx, _, _)) = filtered.get(selected) {
                    execute_command_by_index(*cmd_idx, state, ui);
                    ui.search_query.clear();
                    ui.search_in_command_mode = false;
                    return false;
                }
            }
            // If no selection, try to execute by exact match
            let cmd = query_after_colon.trim().to_string();
            if !cmd.is_empty() {
                execute_command_legacy(&cmd, state, ui);
                ui.search_query.clear();
                ui.search_in_command_mode = false;
            }
            return false;
        }
        // Not a command - copy result as before
        copy_search_result(state, ui);
        return false;
    }

    // Navigate command list or search results
    if key == KeyCode::Down || key == KeyCode::Char('j') {
        if is_command_mode {
            let query_after_colon = &ui.search_query[1..];
            let filtered = filter_commands(query_after_colon);
            let count = filtered.len();
            if count > 0 {
                let current = ui.search_list_state.selected().unwrap_or(0);
                let next = if current >= count - 1 { 0 } else { current + 1 };
                ui.search_list_state.select(Some(next));
            }
        } else {
            let count = ui.filtered_indices.len();
            if count > 0 {
                let current = ui.search_list_state.selected().unwrap_or(0);
                let next = if current >= count - 1 { 0 } else { current + 1 };
                ui.search_list_state.select(Some(next));
            }
        }
        return false;
    }

    if key == KeyCode::Up || key == KeyCode::Char('k') {
        if is_command_mode {
            let query_after_colon = &ui.search_query[1..];
            let filtered = filter_commands(query_after_colon);
            let count = filtered.len();
            if count > 0 {
                let current = ui.search_list_state.selected().unwrap_or(0);
                let prev = if current == 0 { count - 1 } else { current - 1 };
                ui.search_list_state.select(Some(prev));
            }
        } else {
            let count = ui.filtered_indices.len();
            if count > 0 {
                let current = ui.search_list_state.selected().unwrap_or(0);
                let prev = if current == 0 { count - 1 } else { current - 1 };
                ui.search_list_state.select(Some(prev));
            }
        }
        return false;
    }

    // Handle character input for search/query
    if let KeyCode::Char(c) = key {
        ui.search_query.push(c);
        ui.search_in_command_mode = ui.search_query.starts_with(':');
        if ui.search_in_command_mode {
            update_command_filter(ui);
        } else {
            update_search_filter(state, ui);
        }
        return false;
    }

    // Handle backspace
    if key == KeyCode::Backspace {
        ui.search_query.pop();
        ui.search_in_command_mode = ui.search_query.starts_with(':');
        if ui.search_query.is_empty() {
            ui.filtered_indices.clear();
            ui.search_list_state.select(None);
            ui.search_showing_spellbooks = true;
            ui.search_spellbook_index = Some(0);
            ui.search_in_command_mode = false;
        } else if ui.search_in_command_mode {
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

    if clipboard::copy_to_clipboard(&spell.incantation) {
        ui.copy_feedback = Some("Copied!".to_string());
    } else {
        ui.copy_feedback = Some("Copy failed".to_string());
    }
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

    if clipboard::copy_to_clipboard(&spell.incantation) {
        ui.copy_feedback = Some("Copied!".to_string());
    } else {
        ui.copy_feedback = Some("Copy failed".to_string());
    }
}

/// Saves the current spell and returns to the spellbook list.
fn save_spell(state: &State, ui: &mut UiState) {
    if ui.add_spell_name.trim().is_empty() {
        ui.add_spell_message = Some(("Name is required".to_string(), true));
        return;
    }
    if ui.add_spell_command.trim().is_empty() {
        ui.add_spell_message = Some(("Command is required".to_string(), true));
        return;
    }

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

    match Archivist::append_spell("codex.toml", &spell, spellbook_name.as_deref()) {
        Ok(_) => {
            ui.add_spell_message = Some(("Spell saved!".to_string(), false));
            ui.add_spell_has_unsaved = false;
            log_info!("Spell saved: {}", spell.name);
        }
        Err(e) => {
            ui.add_spell_message = Some((format!("Save failed: {}", e), true));
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
            if ui.add_spell_field == AddSpellField::Spellbook && ui.add_spell_dropdown_open {
                ui.add_spell_dropdown_open = false;
            } else if ui.add_spell_has_unsaved {
                ui.add_spell_message = Some((
                    "Unsaved changes - press Esc again to discard".to_string(),
                    true,
                ));
            } else {
                ui.clear_add_spell_form();
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
            ui.add_spell_message = None;
            ui.add_spell_has_unsaved = true;
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
            ui.add_spell_message = None;
            ui.add_spell_has_unsaved = true;
            false
        }
        _ => false,
    }
}

/// Update filtered commands based on current query after ":"
fn update_command_filter(ui: &mut UiState) {
    let query_after_colon = ui.search_query.strip_prefix(':').unwrap_or("");
    let filtered = filter_commands(query_after_colon);
    ui.filtered_indices = filtered.iter().map(|(idx, _, _)| *idx).collect();

    if !ui.filtered_indices.is_empty() {
        ui.search_list_state.select(Some(0));
    } else {
        ui.search_list_state.select(None);
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
            ui.add_spell_field = AddSpellField::Name;
            ui.is_typing = true;
            log_info!("Command: new spell");
        }
        // Add new spellbook
        "N" | "new book" | "new spellbook" | "add spellbook" | "add book" => {
            ui.search_mode = SearchMode::AddSpellbook;
            ui.is_typing = true;
            log_info!("Command: new spellbook");
        }
        // Switch to browse spellbooks
        "b" | "books" | "browse" | "spellbooks" => {
            ui.search_mode = SearchMode::BrowseSpellbooks;
            ui.search_showing_spellbooks = true;
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
        // Theme commands
        "t" | "theme" | "next theme" => {
            state.cycle_theme();
            ui.view_mode = state.user_settings.view_mode;
            log_info!("Command: cycle theme");
        }
        // Help
        "?" | "help" | "commands" => {
            ui.copy_feedback = Some("Commands: :n/new spell, :N/new book, :b/browse, :s/spells, :c/cards, :p/spines, :a/auto, :t/theme".to_string());
            log_info!("Command: help");
        }
        // Unknown command
        _ => {
            ui.copy_feedback = Some(format!("Unknown command: {}", cmd));
            log_info!("Unknown command: {}", cmd);
        }
    }
}
