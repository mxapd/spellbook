mod archivist;
mod cli;
mod clipboard;
mod editor;
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
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
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
        Ok(c) => {
            // Run validation and log warnings
            let warnings = validation::validate_codex_warnings(&c);
            for warning in &warnings {
                match warning.severity {
                    validation::WarningSeverity::Error => {
                        log_warn!("Validation error: {}", warning.message);
                    }
                    validation::WarningSeverity::Warning => {
                        log_warn!("Warning: {}", warning.message);
                    }
                    validation::WarningSeverity::Info => {
                        log_info!("Info: {}", warning.message);
                    }
                }
            }
            if !warnings.is_empty() {
                log_info!("Found {} validation issue(s)", warnings.len());
            }
            c
        }
        Err(e) => {
            eprintln!("Error loading codex.toml: {}", e);
            eprintln!("Creating empty codex...");
            let empty_codex = models::Codex {
                spells: vec![],
                spellbooks: vec![],
            };
            // Try to save the empty codex
            if let Err(save_err) = archivist::Archivist::save(&empty_codex, "codex.toml") {
                eprintln!("Warning: Could not save empty codex: {}", save_err);
            }
            empty_codex
        }
    };

    let mut state = state::State::new(codex);
    let mut ui_state = ui::UiState::new(mode == AppMode::AddSpell);

    // Start on BrowseSpellbooks mode by default (unless --add is passed for AddSpell mode)
    if mode != AppMode::Browse {
        ui_state.set_mode(ui::Mode::AddSpell(ui::FormState::default()));
    }

    logging::init_logging();
    log_info!("Spellbook started (mode: {:?})", mode);

    // Enable alternate screen for TUI
    let _ = execute!(io::stdout(), EnterAlternateScreen);

    // Initialize job manager (starts background polling thread)
    let _ = invoker::init_job_manager(state.launch_dir.clone());
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

        let poll_result = event::poll(std::time::Duration::from_millis(100));
        match poll_result {
            Ok(true) => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    log_debug!(
                        "KeyEvent: code={:?}, modifiers={:?}",
                        key.code,
                        key.modifiers
                    );
                    let should_quit =
                        ui::handle_event(key.code, &mut state, &mut ui_state, key.modifiers);
                    if should_quit || ui_state.should_quit {
                        log_info!("Spellbook exiting");
                        let _ = execute!(io::stdout(), LeaveAlternateScreen);
                        return Ok(());
                    }

                    if ui_state.needs_redraw {
                        terminal.draw(|frame| {
                            ui::render(frame, &state, &mut ui_state);
                        })?;
                        let _ = ui_state.clear_redraw_flag();
                    }
                }
            }
            Ok(false) => {
                // Timeout elapsed - poll for streaming output, update spinner and cursor, and redraw
                ui::streaming_modal::poll_stream_output(&mut ui_state);
                ui_state.tick_spinner();
                ui_state.tick_search_cursor();
                if ui_state.is_loading() {
                    terminal.draw(|frame| {
                        ui::render(frame, &state, &mut ui_state);
                    })?;
                }
            }
            Err(e) => {
                log_debug!("Event poll error: {}", e);
            }
        }
    }
}
