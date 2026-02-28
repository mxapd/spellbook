use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("✦ spellbooks ✦")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
        layout[0],
    );

    let items: Vec<ListItem> = app
        .codex
        .spellbooks
        .iter()
        .map(|sb| ListItem::new(Line::from(sb.name.as_str()).centered()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, layout[1], &mut app.spellbook_list_state);

    frame.render_widget(
        Paragraph::new(" [↑/↓] navigate  [enter] open  [q] quit")
            .style(Style::default().fg(Color::DarkGray)),
        layout[2],
    );
}
