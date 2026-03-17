use crate::state::State;
use crate::ui::{AddSpellField, UiState};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListState;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let theme = &state.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let label_style = Style::new().fg(theme.muted);
    let active_style = Style::new().bg(theme.selection).fg(theme.fg);
    let normal_style = Style::new().bg(theme.bg).fg(theme.fg);
    let accent_style = Style::new().fg(theme.accent).bold();

    let block = Block::bordered()
        .border_style(Style::new().fg(theme.border))
        .title_style(accent_style)
        .title("  Add New Spell  ");

    let form_area = chunks[0];
    let inner_area = block.inner(form_area);
    frame.render_widget(block, form_area);

    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner_area);

    let name_value = if ui.add_spell_name.is_empty() {
        "[...]".to_string()
    } else {
        format!("[{}]", ui.add_spell_name)
    };
    let name_line = Paragraph::new(Line::from(vec![
        Span::styled("* Name:    ", label_style),
        Span::styled(
            name_value,
            if ui.add_spell_field == AddSpellField::Name {
                active_style
            } else {
                normal_style
            },
        ),
    ]));
    frame.render_widget(name_line, form_chunks[0]);

    let cmd_value = if ui.add_spell_command.is_empty() {
        "[...]".to_string()
    } else {
        format!("[{}]", ui.add_spell_command)
    };
    let cmd_line = Paragraph::new(Line::from(vec![
        Span::styled("> Command: ", label_style),
        Span::styled(
            cmd_value,
            if ui.add_spell_field == AddSpellField::Command {
                active_style
            } else {
                normal_style
            },
        ),
    ]));
    frame.render_widget(cmd_line, form_chunks[1]);

    let lore_value = if ui.add_spell_lore.is_empty() {
        "[...]".to_string()
    } else {
        format!("[{}]", ui.add_spell_lore)
    };
    let lore_line = Paragraph::new(Line::from(vec![
        Span::styled(":: Lore:   ", label_style),
        Span::styled(
            lore_value,
            if ui.add_spell_field == AddSpellField::Lore {
                active_style
            } else {
                normal_style
            },
        ),
    ]));
    frame.render_widget(lore_line, form_chunks[2]);

    let school_value = if ui.add_spell_school.is_empty() {
        "[...]".to_string()
    } else {
        format!("[{}]", ui.add_spell_school)
    };
    let school_line = Paragraph::new(Line::from(vec![
        Span::styled("^ School:  ", label_style),
        Span::styled(
            school_value,
            if ui.add_spell_field == AddSpellField::School {
                active_style
            } else {
                normal_style
            },
        ),
    ]));
    frame.render_widget(school_line, form_chunks[3]);

    let tags_hint = "(comma separated)";
    let tags_value = if ui.add_spell_tags.is_empty() {
        "[...]".to_string()
    } else {
        format!("[{}]", ui.add_spell_tags)
    };
    let tags_line = Paragraph::new(Line::from(vec![
        Span::styled("# Tags:    ", label_style),
        Span::styled(
            tags_value,
            if ui.add_spell_field == AddSpellField::Tags {
                active_style
            } else {
                normal_style
            },
        ),
        Span::raw(" "),
        Span::styled(tags_hint, Style::new().fg(theme.muted)),
    ]));
    frame.render_widget(tags_line, form_chunks[4]);

    let divider = Paragraph::new("-".repeat(30))
        .alignment(Alignment::Center)
        .style(Style::new().fg(theme.border));
    frame.render_widget(divider, form_chunks[5]);

    let spellbook_is_active = ui.add_spell_field == AddSpellField::Spellbook;
    let current_selection = if ui.add_spell_skip_spellbook {
        "Skip - just create spell".to_string()
    } else {
        let selected = ui.add_spell_spellbook.unwrap_or(0);
        state
            .codex
            .spellbooks
            .get(selected)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "None".to_string())
    };

    let spellbook_line = Paragraph::new(Line::from(vec![
        Span::styled("> Spellbook:", label_style),
        Span::styled(
            if spellbook_is_active {
                format!("[▼] {}", current_selection)
            } else {
                format!("[ ] {}", current_selection)
            },
            if spellbook_is_active {
                active_style
            } else {
                normal_style
            },
        ),
    ]));
    frame.render_widget(spellbook_line, form_chunks[6]);

    // Show dropdown when Spellbook field is active
    if spellbook_is_active {
        let dropdown_items: Vec<ListItem> = state
            .codex
            .spellbooks
            .iter()
            .map(|sb| ListItem::new(sb.name.clone()).style(Style::new().fg(theme.fg)))
            .chain(std::iter::once(
                ListItem::new("Skip - just create spell").style(Style::new().fg(theme.muted)),
            ))
            .collect();

        let dropdown_list = List::new(dropdown_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(theme.border)),
            )
            .highlight_style(
                Style::new()
                    .bg(theme.selection)
                    .fg(theme.fg)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let mut dropdown_state = ListState::default();
        dropdown_state.select(Some(ui.add_spell_dropdown_index));

        // Calculate dropdown position - show below the spellbook field
        if form_chunks.len() > 7 {
            frame.render_stateful_widget(dropdown_list, form_chunks[7], &mut dropdown_state);
        }
    }

    let footer = Paragraph::new(format!(
        "tab/↑↓ navigate  Enter save  Esc cancel  t {}",
        state.theme_names[state.current_theme_index]
    ))
    .style(Style::new().fg(theme.muted).bg(theme.bg));

    frame.render_widget(footer, chunks[1]);
}
