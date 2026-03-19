mod clipboard;
mod logging;
mod models;
mod persistence;
mod state;
mod ui;

use crossterm::{
    event::{self, Event, KeyEventKind, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use ratatui::DefaultTerminal;
use std::env;
use std::io;

fn main() -> io::Result<()> {
    ratatui::run(run)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Browse,
    AddSpell,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Browse
    }
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mode = if args.iter().any(|a| a == "--add") {
        AppMode::AddSpell
    } else {
        AppMode::Browse
    };

    let codex = match persistence::Archivist::load("codex.toml") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading codex.toml: {}", e);
            eprintln!("Please ensure the file exists and contains valid TOML.");
            return Ok(());
        }
    };

    let mut state = state::State::new(codex);
    let mut ui_state = ui::UiState::new(mode == AppMode::AddSpell);

    logging::init_logging();
    log_info!("Spellbook started (mode: {:?})", mode);

    let _ = execute!(
        io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    );

    loop {
        terminal.draw(|frame| {
            ui::render(frame, &state, &mut ui_state);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            log_debug!(
                "KeyEvent: code={:?}, modifiers={:?}",
                key.code,
                key.modifiers
            );
            let should_quit = ui::handle_event(key.code, &mut state, &mut ui_state, key.modifiers);
            if should_quit {
                log_info!("Spellbook exiting");
                return Ok(());
            }
        }
    }
}
