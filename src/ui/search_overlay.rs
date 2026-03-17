use crate::state::State;
use crate::ui::UiState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let area = frame.area();
    let theme = &state.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    let input_text = format!("/{}", ui.search_query);
    let input_block = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(Style::new().fg(theme.accent))
                .title_style(Style::new().fg(theme.accent)),
        )
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_widget(input_block, chunks[0]);

    if ui.filtered_indices.is_empty() {
        let message = if ui.search_query.is_empty() {
            "Type to search all spells..."
        } else {
            "No spells found"
        };
        let empty = Paragraph::new(message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(theme.border)),
            )
            .style(Style::new().fg(theme.muted).bg(theme.bg));
        frame.render_widget(empty, chunks[1]);
    } else {
        let results: Vec<ListItem> = ui
            .filtered_indices
            .iter()
            .filter_map(|&idx| state.codex.spells.get(idx))
            .map(|spell| {
                let line = format!("{}  [{}]", spell.name, spell.school);
                ListItem::new(line).style(Style::new().fg(theme.fg))
            })
            .collect();

        let list = List::new(results)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Results ")
                    .border_style(Style::new().fg(theme.border))
                    .title_style(Style::new().fg(theme.accent)),
            )
            .highlight_style(
                Style::new()
                    .fg(theme.selection)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .highlight_symbol(">")
            .style(Style::new().bg(theme.bg).fg(theme.fg));

        frame.render_stateful_widget(list, chunks[1], &mut ui.search_list_state);
    }

    let details: Vec<Line> = if ui.filtered_indices.is_empty() {
        vec![Line::from("")]
    } else {
        let selected_idx = ui.search_list_state.selected().unwrap_or(0);

        let spell_opt = ui.filtered_indices.get(selected_idx).copied();

        match spell_opt {
            Some(spell_idx) if spell_idx < state.codex.spells.len() => {
                let spell = &state.codex.spells[spell_idx];
                let glyphs_str = spell.glyphs.join(", ");

                let muted = Style::new().fg(theme.muted);
                let command_style = Style::new().fg(theme.accent);

                vec![
                    Line::from(vec![
                        Span::raw("$ "),
                        Span::styled(&spell.incantation, command_style),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&spell.school, muted),
                        Span::styled(" | ", muted),
                        Span::styled(glyphs_str, muted),
                    ]),
                ]
            }
            _ => vec![Line::from("")],
        }
    };

    let details_block = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details ")
                .border_style(Style::new().fg(theme.border))
                .title_style(Style::new().fg(theme.accent)),
        )
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_widget(details_block, chunks[2]);

    let hint = Paragraph::new(format!(
        "↑↓ navigate  enter copy  t {}  esc close",
        state.theme_names[state.current_theme_index]
    ))
    .style(Style::new().fg(theme.muted).bg(theme.bg));
    frame.render_widget(hint, chunks[3]);
}
