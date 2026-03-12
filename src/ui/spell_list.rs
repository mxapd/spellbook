use crate::models::Spell;
use crate::state::State;
use crate::ui::UiState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let bottom_percentage = 40;
    let top_percentage = 60;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(top_percentage),
            Constraint::Percentage(bottom_percentage),
        ])
        .split(frame.area());

    let spellbook_index = match ui.selected_spellbook {
        Some(index) => index,
        None => return,
    };

    let spellbook = &state.codex.spellbooks[spellbook_index];

    let spells: Vec<ListItem> = spellbook
        .spell_ids
        .iter()
        .filter_map(|spell_id| state.codex.spells.iter().find(|s| s.id == *spell_id))
        .map(|spell| ListItem::new(spell.name.clone()))
        .collect();

    let list = List::new(spells);

    frame.render_stateful_widget(list, chunks[0], &mut ui.spell_list_state);

    let bottom = Paragraph::new("Details go here")
        .block(Block::default().borders(Borders::ALL).title("Details"));
    frame.render_widget(bottom, chunks[1]);
}
