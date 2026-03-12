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

    // create a list of spellbooks
    let spellbooks: Vec<ListItem> = state
        .codex
        .spellbooks
        .iter()
        .map(|sb| ListItem::new(format!("{}", sb.name)))
        .collect();

    let list = List::new(spellbooks)
        .block(Block::bordered().title("spellbook"))
        .highlight_style(Style::new().italic())
        .highlight_symbol(">");

    frame.render_stateful_widget(list, chunks[0], &mut ui.spellbook_list_state);

    // footer
    let footer = Paragraph::new(" / search   q quit");

    frame.render_widget(footer, chunks[1]);
}
