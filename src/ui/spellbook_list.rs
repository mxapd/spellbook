use crate::state::State;
use crate::ui::UiState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let theme = &state.theme;

    let spellbooks: Vec<ListItem> = state
        .codex
        .spellbooks
        .iter()
        .map(|sb| ListItem::new(format!("{}", sb.name)).style(Style::new().fg(theme.fg)))
        .collect();

    let list = List::new(spellbooks)
        .block(
            Block::bordered()
                .title("spellbook")
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

    frame.render_stateful_widget(list, chunks[0], &mut ui.spellbook_list_state);

    let footer = Paragraph::new(format!(
        " / search  ↑↓ navigate  enter select  t {}  q quit",
        state.theme_names[state.current_theme_index]
    ))
    .style(Style::new().fg(theme.muted).bg(theme.bg));

    frame.render_widget(footer, chunks[1]);
}
