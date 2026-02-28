mod models;
mod persistence;
mod state;
mod ui;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::io;

fn main() -> io::Result<()> {
    ratatui::run(run)
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let codex = persistence::Archivist::load("codex.json").expect("failed to load codex");
    let state = state::State::new(codex);
    let mut ui_state = ui::UiState::new();

    loop {
        terminal.draw(|frame| {
            ui::render(frame, &state, &mut ui_state);
        })?;

        //this inputhandling can surely be improved
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let should_quit = ui::handle_event(key.code, &state, &mut ui_state);
            if should_quit {
                return Ok(());
            }
        }
    }
}
