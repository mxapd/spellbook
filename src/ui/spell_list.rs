use crate::state::State;
use crate::ui::UiState;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(35),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let theme = &state.theme;

    let spellbook_index = match ui.selected_spellbook() {
        Some(index) => index,
        None => return,
    };

    let spellbook = &state.codex.spellbooks[spellbook_index];

    let spells: Vec<ListItem> = spellbook
        .spell_ids
        .iter()
        .filter_map(|spell_id| state.codex.spells.iter().find(|s| s.id == *spell_id))
        .map(|spell| ListItem::new(spell.name.clone()).style(Style::new().fg(theme.fg)))
        .collect();

    let list_block = Block::bordered()
        .title(spellbook.name.clone())
        .border_style(Style::new().fg(theme.border))
        .title_style(Style::new().fg(theme.accent));

    if spells.is_empty() {
        let inner = list_block.inner(chunks[0]);
        frame.render_widget(list_block, chunks[0]);
        let empty_message = Paragraph::new("No spells\n\nPress :n to add one")
            .style(Style::new().fg(theme.muted).bg(theme.bg));
        frame.render_widget(empty_message, inner);
    } else {
        let list = List::new(spells)
            .block(list_block)
            .highlight_style(
                Style::new()
                    .fg(theme.selection)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .highlight_symbol(">")
            .style(Style::new().bg(theme.bg).fg(theme.fg));

        frame.render_stateful_widget(list, chunks[0], &mut ui.spell_list_state);
    }

    let selected_spell = ui.spell_list_state.selected().and_then(|idx| {
        spellbook
            .spell_ids
            .get(idx)
            .and_then(|spell_id| state.codex.spells.iter().find(|s| s.id == *spell_id))
    });

    let details = match selected_spell {
        Some(spell) => {
            let glyphs_str = spell.glyphs.join(", ");
            let muted = Style::new().fg(theme.muted);
            let command_style = Style::new().fg(theme.accent);
            let lore_style = Style::new().fg(theme.fg);

            vec![
                Line::from(vec![
                    Span::raw("$ "),
                    Span::styled(spell.incantation.clone(), command_style),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("⌬ ", muted),
                    Span::styled(spell.school.clone(), muted),
                ]),
                Line::from(vec![
                    Span::styled("△ ", muted),
                    Span::styled(glyphs_str.clone(), muted),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled("-", muted)]),
                Line::from(vec![Span::styled(spell.lore.clone(), lore_style)]),
            ]
        }
        None => vec![Line::from("Select a spell to view details")],
    };

    let details_block = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Details")
                .border_style(Style::new().fg(theme.border))
                .title_style(Style::new().fg(theme.accent)),
        )
        .alignment(Alignment::Left)
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_widget(details_block, chunks[1]);

    let footer = if let Some(ref msg) = ui.copy_feedback {
        let single_line = msg.lines().next().unwrap_or(msg).to_string();
        Paragraph::new(single_line)
            .style(Style::new().fg(ratatui::style::Color::Green).bg(theme.bg))
            .alignment(ratatui::layout::Alignment::Center)
    } else {
        Paragraph::new(format!(
            "↑↓ navigate  enter copy  s simple  Ctrl+r tui  Ctrl+b bg  esc back  q quit",
        ))
        .style(Style::new().fg(theme.muted).bg(theme.bg))
    };
    frame.render_widget(footer, chunks[2]);
}

pub fn render_in_area(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(35),
            Constraint::Length(1),
        ])
        .split(area);

    let theme = &state.theme;

    let spellbook_index = match ui.selected_spellbook() {
        Some(index) => index,
        None => return,
    };

    let spellbook = &state.codex.spellbooks[spellbook_index];

    let spells: Vec<ListItem> = spellbook
        .spell_ids
        .iter()
        .filter_map(|spell_id| state.codex.spells.iter().find(|s| s.id == *spell_id))
        .map(|spell| ListItem::new(spell.name.clone()).style(Style::new().fg(theme.fg)))
        .collect();

    let list_block = Block::bordered()
        .title(spellbook.name.clone())
        .border_style(Style::new().fg(theme.border))
        .title_style(Style::new().fg(theme.accent));

    if spells.is_empty() {
        let inner = list_block.inner(chunks[0]);
        frame.render_widget(list_block, chunks[0]);
        let empty_message = Paragraph::new("No spells in this spellbook")
            .style(Style::new().fg(theme.muted).bg(theme.bg));
        frame.render_widget(empty_message, inner);
    } else {
        let list = List::new(spells)
            .block(list_block)
            .highlight_style(
                Style::new()
                    .fg(theme.selection)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .highlight_symbol(">")
            .style(Style::new().bg(theme.bg).fg(theme.fg));

        frame.render_stateful_widget(list, chunks[0], &mut ui.spell_list_state);
    }

    let selected_spell = ui.spell_list_state.selected().and_then(|idx| {
        spellbook
            .spell_ids
            .get(idx)
            .and_then(|spell_id| state.codex.spells.iter().find(|s| s.id == *spell_id))
    });

    let details = match selected_spell {
        Some(spell) => {
            let glyphs_str = spell.glyphs.join(", ");
            let muted = Style::new().fg(theme.muted);
            let command_style = Style::new().fg(theme.accent);
            let lore_style = Style::new().fg(theme.fg);

            vec![
                Line::from(vec![
                    Span::raw("$ "),
                    Span::styled(spell.incantation.clone(), command_style),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("School: ", muted),
                    Span::styled(spell.school.clone(), muted),
                ]),
                Line::from(vec![
                    Span::styled("Glyphs: ", muted),
                    Span::styled(glyphs_str.clone(), muted),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(spell.lore.clone(), lore_style)]),
            ]
        }
        None => vec![Line::from("Select a spell to view details")],
    };

    let details_block = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Details")
                .border_style(Style::new().fg(theme.border))
                .title_style(Style::new().fg(theme.accent)),
        )
        .alignment(Alignment::Left)
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_widget(details_block, chunks[1]);

    let footer = if let Some(ref msg) = ui.copy_feedback {
        let single_line = msg.lines().next().unwrap_or(msg).to_string();
        Paragraph::new(single_line)
            .style(Style::new().fg(ratatui::style::Color::Green).bg(theme.bg))
            .alignment(ratatui::layout::Alignment::Center)
    } else {
        Paragraph::new("↑↓ navigate  enter copy  s simple  Ctrl+r tui  Ctrl+b bg  esc back")
            .style(Style::new().fg(theme.muted).bg(theme.bg))
    };
    frame.render_widget(footer, chunks[2]);
}
