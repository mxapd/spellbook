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

    let spellbook = &app.codex.spellbooks[app.selected_spellbook.unwrap()];

    frame.render_widget(
        Paragraph::new(format!("✦ {} ✦", spellbook.name))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
        layout[0],
    );

    let items: Vec<ListItem> = spellbook
        .spell_ids
        .iter()
        .filter_map(|id| app.codex.spells.iter().find(|s| s.id == *id))
        .map(|spell| ListItem::new(Line::from(spell.name.as_str()).centered()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, layout[1], &mut app.spell_list_state);

    frame.render_widget(
        Paragraph::new(" [↑/↓] navigate  [esc] back  [q] quit")
            .style(Style::default().fg(Color::DarkGray)),
        layout[2],
    );
}
