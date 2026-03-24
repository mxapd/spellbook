//! Context-aware footer hints
//!
//! This module generates footer hints based on the current application state,
//! including mode, overlays, and UI context.

use crate::ui::{Mode, Overlay, UiState};
use crate::models::{FocusTarget, ViewMode};

/// Generate footer text based on current application state
pub fn get_footer_text(ui: &UiState, has_jobs: bool) -> String {
    // Priority: Loading > Overlay > Mode-specific
    
    if ui.is_loading() {
        return String::new(); // Loading indicator is shown separately
    }
    
    // Check for active overlays
    if let Some(overlay) = ui.top_overlay() {
        return get_overlay_footer_text(overlay, ui);
    }
    
    // Check if jobs sidebar is focused
    if ui.jobs_sidebar_open && ui.focus == FocusTarget::JobsSidebar {
        return "↑↓: navigate | Enter: view | k: kill | Esc: close | Tab: main".to_string();
    }
    
    // Mode-specific hints
    match ui.mode {
        Mode::BrowseSpellbooks => get_browse_spellbooks_footer(ui),
        Mode::BrowseSpells => get_browse_spells_footer(ui, has_jobs),
        Mode::AddSpell | Mode::EditSpell => get_add_spell_footer(ui),
        Mode::AddSpellbook => "Tab: next field | Enter: save | Ctrl+S: save | Esc: cancel".to_string(),
    }
}

fn get_browse_spellbooks_footer(ui: &UiState) -> String {
    if ui.search_active() {
        "Type to filter | ↑↓←→: navigate | Enter: open | Esc: clear".to_string()
    } else {
        let view_hint = match ui.view_mode {
            ViewMode::Cards => "v: spines",
            ViewMode::Spines => "v: auto",
            ViewMode::List => "v: cards",
        };
        format!("↑↓←→: navigate | Enter: open | /: search | :: commands | t: theme | {} | q: quit", view_hint)
    }
}

fn get_browse_spells_footer(ui: &UiState, has_jobs: bool) -> String {
    let base = if ui.search_active() {
        "Type to filter | ↑↓: navigate | Enter: copy | Esc: clear".to_string()
    } else {
        "↑↓: navigate | Enter: copy | r: run | s: simple | ^r: tui | ^b: bg | e: edit | d: delete | f: fav | /: search | ←: back".to_string()
    };
    
    if has_jobs && !ui.search_active() {
        format!("{} | :jobs: sidebar", base)
    } else {
        base
    }
}

fn get_add_spell_footer(ui: &UiState) -> String {
    if ui.is_typing {
        if ui.add_spell.field == crate::ui::AddSpellField::Spellbook && ui.add_spell.dropdown_open {
            "↑↓: navigate | Enter: select | Esc: close".to_string()
        } else {
            "Tab: next field | Ctrl+S: save | Esc: cancel".to_string()
        }
    } else {
        "Tab: next field | Enter: save | Ctrl+S: save | Esc: cancel".to_string()
    }
}

fn get_overlay_footer_text(overlay: Overlay, ui: &UiState) -> String {
    match overlay {
        Overlay::OutputModal => {
            if ui.streaming_modal.is_running() {
                "↑↓: scroll | s: toggle scroll | ^C: kill | ^B: background".to_string()
            } else {
                "↑↓: scroll | s: toggle scroll | Esc: close".to_string()
            }
        }
        Overlay::ConfirmDialog => "←→: select | Enter: confirm | Esc: cancel | y: yes | n: no".to_string(),
        Overlay::CommandPalette => "Type to filter | ↑↓: navigate | Enter: execute | Esc: cancel".to_string(),
        Overlay::Help => "↑↓: scroll | Esc: close".to_string(),
        Overlay::InputPopup => "Type value | Tab: next | Enter: confirm | Esc: cancel".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{SearchMode, UiState};

    #[test]
    fn test_loading_state_returns_empty() {
        let mut ui = UiState::new(false);
        ui.start_loading("Testing...");
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.is_empty());
    }

    #[test]
    fn test_browse_spellbooks_footer() {
        let ui = UiState::new(false);
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("navigate"));
        assert!(footer.contains("Enter: open"));
        assert!(footer.contains("/: search"));
        assert!(footer.contains("t: theme"));
        assert!(footer.contains("q: quit"));
    }

    #[test]
    fn test_browse_spellbooks_search_active() {
        let mut ui = UiState::new(false);
        ui.search.search_active = true;
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("filter"));
        assert!(footer.contains("Esc: clear"));
    }

    #[test]
    fn test_browse_spells_footer() {
        let mut ui = UiState::new(false);
        ui.mode = Mode::BrowseSpells;
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("navigate"));
        assert!(footer.contains("Enter: copy"));
        assert!(footer.contains("r: run"));
        assert!(footer.contains("e: edit"));
        assert!(footer.contains("d: delete"));
        assert!(footer.contains("f: fav"));
    }

    #[test]
    fn test_browse_spells_with_jobs() {
        let mut ui = UiState::new(false);
        ui.mode = Mode::BrowseSpells;
        
        let footer = get_footer_text(&ui, true);
        assert!(footer.contains(":jobs"));
    }

    #[test]
    fn test_add_spell_footer() {
        let mut ui = UiState::new(false);
        ui.mode = Mode::AddSpell;
        ui.is_typing = false;
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("Tab: next field"));
        assert!(footer.contains("Ctrl+S: save"));
        assert!(footer.contains("Esc: cancel"));
    }

    #[test]
    fn test_add_spellbook_footer() {
        let mut ui = UiState::new(false);
        ui.mode = Mode::AddSpellbook;
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("Tab: next field"));
        assert!(footer.contains("Enter: save"));
    }

    #[test]
    fn test_jobs_sidebar_focused_footer() {
        let mut ui = UiState::new(false);
        ui.jobs_sidebar_open = true;
        ui.focus = FocusTarget::JobsSidebar;
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("navigate"));
        assert!(footer.contains("Enter: view"));
        assert!(footer.contains("k: kill"));
        assert!(footer.contains("Tab: main"));
    }

    #[test]
    fn test_output_modal_running_footer() {
        let mut ui = UiState::new(false);
        ui.push_overlay(Overlay::OutputModal);
        ui.streaming_modal.output.is_streaming = true;
        ui.streaming_modal.streaming = Some(crate::ui::streaming_modal::StreamingState::new(
            "test".to_string(),
            None,
            None,
        ));
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("kill"));
        assert!(footer.contains("background"));
    }

    #[test]
    fn test_output_modal_finished_footer() {
        let mut ui = UiState::new(false);
        ui.push_overlay(Overlay::OutputModal);
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("scroll"));
        assert!(footer.contains("Esc: close"));
        assert!(!footer.contains("kill"));
    }

    #[test]
    fn test_confirm_dialog_footer() {
        let mut ui = UiState::new(false);
        ui.push_overlay(Overlay::ConfirmDialog);
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("select"));
        assert!(footer.contains("confirm"));
        assert!(footer.contains("cancel"));
        assert!(footer.contains("y: yes"));
        assert!(footer.contains("n: no"));
    }

    #[test]
    fn test_command_palette_footer() {
        let mut ui = UiState::new(false);
        ui.push_overlay(Overlay::CommandPalette);
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("filter"));
        assert!(footer.contains("navigate"));
        assert!(footer.contains("execute"));
    }

    #[test]
    fn test_help_footer() {
        let mut ui = UiState::new(false);
        ui.push_overlay(Overlay::Help);
        
        let footer = get_footer_text(&ui, false);
        assert!(footer.contains("scroll"));
        assert!(footer.contains("Esc: close"));
    }

    #[test]
    fn test_overlay_priority_over_mode() {
        let mut ui = UiState::new(false);
        ui.mode = Mode::BrowseSpells;
        ui.push_overlay(Overlay::Help);
        
        let footer = get_footer_text(&ui, false);
        // Should show help footer, not browse spells footer
        assert!(footer.contains("scroll"));
        assert!(!footer.contains("Enter: copy"));
    }

    #[test]
    fn test_view_mode_hints() {
        // Default is List, so first hint should be "v: cards"
        let ui_list = UiState::new(false);
        let footer_list = get_footer_text(&ui_list, false);
        assert!(footer_list.contains("v: cards"), "Expected 'v: cards' in: {}", footer_list);
        
        let mut ui_cards = UiState::new(false);
        ui_cards.view_mode = ViewMode::Cards;
        let footer_cards = get_footer_text(&ui_cards, false);
        assert!(footer_cards.contains("v: spines"), "Expected 'v: spines' in: {}", footer_cards);
        
        let mut ui_spines = UiState::new(false);
        ui_spines.view_mode = ViewMode::Spines;
        let footer_spines = get_footer_text(&ui_spines, false);
        assert!(footer_spines.contains("v: auto"), "Expected 'v: auto' in: {}", footer_spines);
    }
}
