mod archivist;
mod clipboard;
mod cli;
mod invoker;
mod logging;
mod models;
mod state;
mod ui;
mod validation;

use crate::cli::{AppMode, CliArgs};
use crossterm::{
    event::{self, Event, KeyEventKind, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use ratatui::DefaultTerminal;
use std::io;

fn main() -> io::Result<()> {
    ratatui::run(run)
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let args = CliArgs::parse();
    let mode = args.mode;

    let codex = match archivist::Archivist::load("codex.toml") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading codex.toml: {}", e);
            eprintln!("Please ensure the file exists and contains valid TOML.");
            return Ok(());
        }
    };

    let mut state = state::State::new(codex);
    let mut ui_state = ui::UiState::new(mode == AppMode::AddSpell);

    // Start on SearchOverlay by default (unless --add is passed for AddSpell screen)
    if mode == AppMode::Browse {
        ui_state.open_search();
    }

    logging::init_logging();
    log_info!("Spellbook started (mode: {:?})", mode);

    // Initialize job manager (starts background polling thread)
    let _ = invoker::get_job_manager();
    log_info!("Job manager initialized");

    let _ = execute!(
        io::stdout(),
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
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

            // Force redraw if requested (e.g., after Alt+R)
            if ui_state.needs_redraw {
                terminal.draw(|frame| {
                    ui::render(frame, &state, &mut ui_state);
                })?;
                let _ = ui_state.clear_redraw_flag();
            }
        }
    }
}
