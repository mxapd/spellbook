mod models;
mod persistence;
mod state;
mod ui;

use ratatui;

fn main() -> std::io::Result<()> {
    // 1. draw update to terminal
    ratatui::run(codex)
    // 2. read input
    // 3. update state
}

fn codex(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let codex = persistence::Archivist::load("codex.json").expect("failed to load codex");
    let state = state::State::new(codex);
    let mut ui_state = ui::UiState::new();

    loop {
        terminal.draw(|frame| {
            ui::render(frame, &state, &mut ui_state);
        })?;
    }
}
