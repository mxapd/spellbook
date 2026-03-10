use crate::state::State;
use crate::ui::UiState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // list area
            Constraint::Length(1), // footer
        ])
        .split(frame.area());

    // Spellbook list
    let items: Vec<ListItem> = state
        .codex
        .spellbooks
        .iter()
        .enumerate()
        .map(|(i, sb)| {
            let prefix = if Some(i) == ui.spellbook_list_state.selected() {
                "> "
            } else {
                "  "
            };
            ListItem::new(format!("{}{}", prefix, sb.name))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("spellbook").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[0], &mut ui.spellbook_list_state);

    // Footer
    let footer = Paragraph::new(" / search   q quit").style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, chunks[1]);
}

