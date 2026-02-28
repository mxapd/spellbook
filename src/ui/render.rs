use crate::state::State;
use crate::ui::Screen;
use crate::ui::UiState;
use crate::ui::spellbook_list;
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    match ui.screen {
        Screen::SpellbookList => spellbook_list::render(frame, state, ui),
        Screen::SpellList => {}
    }
}
