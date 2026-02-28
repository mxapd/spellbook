use crate::app::{App, Screen};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::SpellbookList => super::spellbook_list::render(frame, app),
        Screen::SpellList => super::spell_list::render(frame, app),
    }
}

