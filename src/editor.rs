use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::log_error;
use crate::log_info;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

/// Open the user's preferred editor with the given initial content and return the saved text.
///
/// The editor is determined by the `$EDITOR` environment variable, falling back to `vi`, then
/// `nano`. If the editor exits unsuccessfully or the file is not modified, `None` is returned.
///
/// The terminal is restored before launching the editor and re-initialized afterwards so that
/// the editor can run in a normal terminal environment.
pub fn edit_command(initial_content: &str) -> io::Result<Option<String>> {
    let editor = resolve_editor()?;
    log_info!("Opening editor: {}", editor);

    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join("spellbook_add_command.toml");

    fs::write(&temp_file, initial_content)?;

    // Restore the terminal so the editor can take over.
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let result = Command::new(&editor).arg(&temp_file).status();

    // Re-enter the alternate screen and raw mode for the TUI.
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let status = match result {
        Ok(status) => status,
        Err(e) => {
            log_error!("Failed to launch editor '{}': {}", editor, e);
            let _ = fs::remove_file(&temp_file);
            return Err(e);
        }
    };

    if !status.success() {
        log_info!("Editor exited with non-zero status; aborting add");
        let _ = fs::remove_file(&temp_file);
        return Ok(None);
    }

    let content = fs::read_to_string(&temp_file)?;
    let _ = fs::remove_file(&temp_file);

    // If the user saved without making any changes, treat it as a cancel.
    if content == initial_content {
        log_info!("Editor content unchanged; aborting add");
        return Ok(None);
    }

    log_info!("Editor returned command ({} bytes)", content.len());
    Ok(Some(content))
}

fn resolve_editor() -> io::Result<String> {
    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            return Ok(editor);
        }
    }

    for fallback in ["vi", "nano"] {
        if command_exists(fallback) {
            return Ok(fallback.to_string());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No editor found. Set $EDITOR or install vi/nano.",
    ))
}

fn command_exists(cmd: &str) -> bool {
    env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths)
                .map(|path| path.join(cmd))
                .find(|full| full.is_file())
        })
        .is_some()
}
