use crate::archivist::Archivist;
use crate::models::{RunMode, Spell};
use crate::state::State;
use crate::ui::UiState;
use crate::{log_error, log_info, log_warn};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuickAddField {
    Name,
    Tags,
    RunMode,
    Spellbook,
}

impl Default for QuickAddField {
    fn default() -> Self {
        Self::Name
    }
}

#[derive(Debug, Clone)]
pub struct QuickAddSpellState {
    pub command: String,
    pub name: String,
    pub tags: String,
    pub run_mode: RunMode,
    pub spellbook_index: Option<usize>,
    pub field: QuickAddField,
    pub dropdown_open: bool,
    pub dropdown_index: usize,
    pub is_editing: bool,
    pub message: Option<(String, bool)>,
}

impl QuickAddSpellState {
    pub fn new(command: String, spellbook_index: Option<usize>) -> Self {
        // Dropdown index 0 is "None (unassigned)"; real spellbooks start at 1.
        let dropdown_index = match spellbook_index {
            Some(idx) => idx + 1,
            None => 0,
        };

        Self {
            command,
            name: String::new(),
            tags: String::new(),
            run_mode: RunMode::Simple,
            spellbook_index,
            field: QuickAddField::Name,
            dropdown_open: false,
            dropdown_index,
            is_editing: false,
            message: None,
        }
    }
}

pub fn render(frame: &mut Frame, state: &crate::state::State, ui: &mut UiState) {
    let area = frame.area();

    let width = 70.min(area.width.saturating_sub(4)).max(40);
    let height = 18.min(area.height.saturating_sub(4)).max(12);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let theme = &state.theme;
    let form = ui
        .quick_add_spell
        .as_ref()
        .expect("quick_add_spell overlay without state");

    let block = Block::default()
        .title(" Add Spell ")
        .title_style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent));

    frame.render_widget(&block, popup_area);

    let inner = block.inner(popup_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    // Command preview (read-only)
    let command_preview = if form.command.contains('\n') {
        format!("{}...", form.command.lines().next().unwrap_or(""))
    } else {
        form.command.clone()
    };
    let command_text = if command_preview.len() > (width as usize).saturating_sub(12) {
        format!(
            "{}...",
            &command_preview[..(width as usize).saturating_sub(12)]
        )
    } else {
        command_preview
    };
    let command_line = Paragraph::new(Line::from(vec![
        Span::styled("> Command: ", Style::new().fg(theme.muted)),
        Span::styled(command_text, Style::new().fg(theme.fg)),
    ]));
    frame.render_widget(command_line, chunks[0]);

    // Name field
    render_text_field(
        frame,
        chunks[1],
        "* Name",
        &form.name,
        form.field == QuickAddField::Name,
        form.is_editing && form.field == QuickAddField::Name,
        true,
        theme,
    );

    // Tags field
    render_text_field(
        frame,
        chunks[2],
        "# Tags",
        &form.tags,
        form.field == QuickAddField::Tags,
        form.is_editing && form.field == QuickAddField::Tags,
        false,
        theme,
    );

    // Run mode field
    let run_mode_value = match form.run_mode {
        RunMode::Simple => "Simple",
        RunMode::Tui => "TUI",
        RunMode::Background => "Background",
    };
    render_select_field(
        frame,
        chunks[3],
        "@ Run",
        run_mode_value,
        form.field == QuickAddField::RunMode,
        theme,
    );

    // Spellbook field
    let spellbook_label = if let Some(idx) = form.spellbook_index {
        state
            .codex
            .spellbooks
            .get(idx)
            .map(|sb| sb.name.as_str())
            .unwrap_or("None")
    } else {
        "None (unassigned)"
    };
    let show_dropdown = form.dropdown_open && form.field == QuickAddField::Spellbook;
    render_select_field(
        frame,
        chunks[4],
        "> Book",
        spellbook_label,
        form.field == QuickAddField::Spellbook,
        theme,
    );

    // Message
    if let Some((message, is_error)) = &form.message {
        let msg_style = if *is_error {
            Style::new().fg(ratatui::style::Color::Red)
        } else {
            Style::new().fg(theme.accent)
        };
        let msg = Paragraph::new(message.as_str())
            .style(msg_style)
            .alignment(Alignment::Center);
        frame.render_widget(msg, chunks[5]);
    }

    // Footer hints
    let footer_text = if form.is_editing {
        "Enter: done  Esc: cancel"
    } else {
        "Tab: next  Enter: edit/select  Ctrl+S: save  Esc: cancel"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::new().fg(theme.muted))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[6]);

    // Render dropdown last so it draws on top of the footer/hints bar.
    if show_dropdown {
        render_spellbook_dropdown(frame, chunks[4], state, form, theme);
    }
}

fn render_text_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_active: bool,
    is_editing: bool,
    required: bool,
    theme: &crate::models::RatatuiColors,
) {
    let display = if is_editing {
        format!("{}_", value)
    } else if value.is_empty() {
        if required {
            "[required]".to_string()
        } else {
            "[optional]".to_string()
        }
    } else {
        value.to_string()
    };

    let border_style = if is_editing {
        Style::new().fg(theme.selection)
    } else if is_active {
        Style::new().fg(theme.accent)
    } else {
        Style::new().fg(theme.border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(label)
        .title_style(if is_active {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.muted)
        });

    let text_style = if value.is_empty() && !is_editing {
        Style::new().fg(theme.muted)
    } else {
        Style::new().fg(theme.fg)
    };

    let paragraph = Paragraph::new(display).block(block).style(text_style);
    frame.render_widget(paragraph, area);
}

fn render_select_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_active: bool,
    theme: &crate::models::RatatuiColors,
) {
    let border_style = if is_active {
        Style::new().fg(theme.accent)
    } else {
        Style::new().fg(theme.border)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(label)
        .title_style(if is_active {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.muted)
        });

    let text = format!("[{}]", value);
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::new().fg(theme.fg));
    frame.render_widget(paragraph, area);
}

fn render_spellbook_dropdown(
    frame: &mut Frame,
    area: Rect,
    state: &crate::state::State,
    form: &QuickAddSpellState,
    theme: &crate::models::RatatuiColors,
) {
    let mut items: Vec<ListItem> =
        vec![ListItem::new("None (unassigned)").style(Style::new().fg(theme.fg))];
    items.extend(
        state
            .codex
            .spellbooks
            .iter()
            .map(|sb| ListItem::new(sb.name.clone()).style(Style::new().fg(theme.fg))),
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(theme.accent)),
        )
        .highlight_style(
            Style::new()
                .bg(theme.selection)
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(form.dropdown_index));

    // Render dropdown below the field if there's room, otherwise above.
    let dropdown_height = (state.codex.spellbooks.len() as u16 + 3).min(8);
    let y = if area.y + area.height + dropdown_height <= frame.area().height {
        area.y + area.height
    } else {
        area.y.saturating_sub(dropdown_height)
    };

    let dropdown_area = Rect::new(area.x, y, area.width, dropdown_height);
    frame.render_widget(Clear, dropdown_area);
    frame.render_stateful_widget(list, dropdown_area, &mut list_state);
}

pub fn handle_key(
    key: KeyCode,
    modifiers: KeyModifiers,
    state: &mut State,
    ui: &mut UiState,
) -> bool {
    let form = match ui.quick_add_spell.as_mut() {
        Some(f) => f,
        None => return true,
    };

    // Ctrl+S saves
    if key == KeyCode::Char('s') && modifiers.contains(KeyModifiers::CONTROL) {
        save(state, ui);
        return false;
    }

    // When editing a text field, handle text input only
    if form.is_editing {
        return handle_edit_mode(key, ui);
    }

    // When the spellbook dropdown is open, handle navigation inside it
    if form.dropdown_open && form.field == QuickAddField::Spellbook {
        return handle_dropdown_navigation(key, state, ui);
    }

    match key {
        KeyCode::Esc => {
            ui.quick_add_spell = None;
            ui.pop_overlay();
            return false;
        }

        KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
            form.dropdown_open = false;
            form.field = next_field(form.field);
            return false;
        }

        KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
            form.dropdown_open = false;
            form.field = prev_field(form.field);
            return false;
        }

        KeyCode::Left | KeyCode::Char('h') => {
            if form.field == QuickAddField::RunMode {
                form.run_mode = cycle_run_mode_left(form.run_mode);
            }
            return false;
        }

        KeyCode::Right | KeyCode::Char('l') => {
            if form.field == QuickAddField::RunMode {
                form.run_mode = cycle_run_mode_right(form.run_mode);
            }
            return false;
        }

        KeyCode::Enter => {
            match form.field {
                QuickAddField::Name | QuickAddField::Tags => {
                    form.is_editing = true;
                }
                QuickAddField::Spellbook => {
                    form.dropdown_open = !form.dropdown_open;
                    if form.dropdown_open {
                        form.dropdown_index = match form.spellbook_index {
                            Some(idx) => idx + 1,
                            None => 0,
                        };
                    }
                }
                QuickAddField::RunMode => {
                    // Enter on run mode just cycles; left/right also work.
                    form.run_mode = cycle_run_mode_right(form.run_mode);
                }
            }
            return false;
        }

        _ => false,
    }
}

fn handle_edit_mode(key: KeyCode, ui: &mut UiState) -> bool {
    let form = ui
        .quick_add_spell
        .as_mut()
        .expect("quick_add_spell state missing");

    match key {
        KeyCode::Esc => {
            form.is_editing = false;
            false
        }
        KeyCode::Enter => {
            form.is_editing = false;
            false
        }
        KeyCode::Backspace => {
            match form.field {
                QuickAddField::Name => {
                    form.name.pop();
                }
                QuickAddField::Tags => {
                    form.tags.pop();
                }
                _ => {}
            }
            form.message = None;
            false
        }
        KeyCode::Char(c) => {
            match form.field {
                QuickAddField::Name => {
                    form.name.push(c);
                }
                QuickAddField::Tags => {
                    form.tags.push(c);
                }
                _ => {}
            }
            form.message = None;
            false
        }
        _ => false,
    }
}

fn handle_dropdown_navigation(key: KeyCode, state: &State, ui: &mut UiState) -> bool {
    let form = ui
        .quick_add_spell
        .as_mut()
        .expect("quick_add_spell state missing");
    let option_count = state.codex.spellbooks.len() + 1; // +1 for "None"

    match key {
        KeyCode::Esc => {
            form.dropdown_open = false;
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if form.dropdown_index > 0 {
                form.dropdown_index -= 1;
            }
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if form.dropdown_index + 1 < option_count {
                form.dropdown_index += 1;
            }
            false
        }
        KeyCode::Enter => {
            form.spellbook_index = if form.dropdown_index == 0 {
                None
            } else {
                Some(form.dropdown_index - 1)
            };
            form.dropdown_open = false;
            false
        }
        _ => false,
    }
}

fn next_field(current: QuickAddField) -> QuickAddField {
    match current {
        QuickAddField::Name => QuickAddField::Tags,
        QuickAddField::Tags => QuickAddField::RunMode,
        QuickAddField::RunMode => QuickAddField::Spellbook,
        QuickAddField::Spellbook => QuickAddField::Name,
    }
}

fn prev_field(current: QuickAddField) -> QuickAddField {
    match current {
        QuickAddField::Name => QuickAddField::Spellbook,
        QuickAddField::Tags => QuickAddField::Name,
        QuickAddField::RunMode => QuickAddField::Tags,
        QuickAddField::Spellbook => QuickAddField::RunMode,
    }
}

fn cycle_run_mode_left(current: RunMode) -> RunMode {
    match current {
        RunMode::Simple => RunMode::Background,
        RunMode::Tui => RunMode::Simple,
        RunMode::Background => RunMode::Tui,
    }
}

fn cycle_run_mode_right(current: RunMode) -> RunMode {
    match current {
        RunMode::Simple => RunMode::Tui,
        RunMode::Tui => RunMode::Background,
        RunMode::Background => RunMode::Simple,
    }
}

fn save(state: &mut State, ui: &mut UiState) {
    let form = match ui.quick_add_spell.as_ref() {
        Some(f) => f,
        None => return,
    };

    if form.name.trim().is_empty() {
        ui.quick_add_spell.as_mut().unwrap().message = Some(("Name is required".to_string(), true));
        return;
    }

    if form.command.trim().is_empty() {
        ui.quick_add_spell.as_mut().unwrap().message =
            Some(("Command is required".to_string(), true));
        return;
    }

    let name_lower = form.name.to_lowercase();
    let exists = state
        .codex
        .spells
        .iter()
        .any(|s| s.name.to_lowercase() == name_lower);
    if exists {
        ui.quick_add_spell.as_mut().unwrap().message =
            Some(("A spell with this name already exists".to_string(), true));
        return;
    }

    let glyphs: Vec<String> = form
        .tags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let spell_id = uuid::Uuid::new_v4().to_string();
    let spell = Spell {
        id: spell_id,
        name: form.name.trim().to_string(),
        incantation: form.command.trim().to_string(),
        lore: String::new(),
        school: String::new(),
        glyphs,
        confirm: false,
        run_mode: form.run_mode,
        working_dir: String::new(),
        favorite: false,
    };

    let spell_name = spell.name.clone();
    let spell_id_for_book = spell.id.clone();

    let saved_to_book = if let Some(spellbook_index) = form.spellbook_index {
        if spellbook_index < state.codex.spellbooks.len() {
            state.codex.spellbooks[spellbook_index]
                .spell_ids
                .push(spell_id_for_book);
            true
        } else {
            log_warn!(
                "Spellbook index {} out of bounds; saving spell '{}' as unassigned",
                spellbook_index,
                spell_name
            );
            false
        }
    } else {
        false
    };

    state.codex.spells.push(spell);

    match Archivist::save(&state.codex, "codex.toml") {
        Ok(_) => {
            state.reload_codex();
            log_info!("Spell saved: {}", spell_name);
            let feedback = if saved_to_book {
                format!("Spell '{}' saved", spell_name)
            } else {
                format!("Spell '{}' saved as unassigned", spell_name)
            };
            ui.show_success(feedback);
            ui.quick_add_spell = None;
            ui.pop_overlay();
        }
        Err(e) => {
            log_error!("Failed to save spell: {}", e);
            ui.quick_add_spell.as_mut().unwrap().message =
                Some((format!("Save failed: {}", e), true));
        }
    }
}
