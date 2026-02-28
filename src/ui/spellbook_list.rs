use crate::state::State;
use crate::ui::UiState;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    // split the screen into 3 vertical chunks
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(0),    // list
            Constraint::Length(1), // footer
        ])
        .split(frame.area());

    // title
    frame.render_widget(
        Paragraph::new("✦ spellbooks ✦")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
        layout[0],
    );

    // build list items from state
    let items: Vec<ListItem> = state
        .codex
        .spellbooks
        .iter()
        .map(|sb| ListItem::new(Line::from(sb.name.as_str()).centered()))
        .collect();

    // list widget
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    // stateful because we need to track which item is selected
    frame.render_stateful_widget(list, layout[1], &mut ui.spellbook_list_state);

    // footer
    frame.render_widget(
        Paragraph::new(" [↑/↓] navigate  [enter] open  [q] quit")
            .style(Style::default().fg(Color::DarkGray)),
        layout[2],
    );
}
