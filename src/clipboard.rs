use arboard::Clipboard;
use std::process::Command;
use std::sync::{LazyLock, Mutex};

const MAX_DISPLAY_LINES: usize = 500;

static CLIPBOARD: LazyLock<Mutex<Option<Clipboard>>> = LazyLock::new(|| {
    Mutex::new(Clipboard::new().ok())
});

pub fn copy_to_clipboard(text: &str) -> bool {
    let mut guard = match CLIPBOARD.lock() {
        Ok(g) => g,
        Err(_) => return false,
    };

    match guard.as_mut() {
        Some(cb) => {
            if cb.set_text(text).is_ok() {
                let _ = Command::new("notify-send")
                    .args(["Copied!", text])
                    .spawn();
                true
            } else {
                false
            }
        }
        None => {
            eprintln!("Clipboard not available");
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    #[allow(dead_code)]
    pub full_stdout: String,
    #[allow(dead_code)]
    pub full_stderr: String,
    pub pid: Option<u32>,
    pub spell_name: Option<String>,
}

impl ExecutionResult {
    pub fn display_stdout(&self) -> String {
        let lines: Vec<&str> = self.stdout.lines().take(MAX_DISPLAY_LINES).collect();
        let mut display = lines.join("\n");
        if self.stdout.lines().count() > MAX_DISPLAY_LINES {
            display.push_str(&format!(
                "\n... ({} more lines truncated, use 's' to save full output)",
                self.stdout.lines().count() - MAX_DISPLAY_LINES
            ));
        }
        display
    }

    pub fn display_stderr(&self) -> String {
        let lines: Vec<&str> = self.stderr.lines().take(MAX_DISPLAY_LINES).collect();
        let mut display = lines.join("\n");
        if self.stderr.lines().count() > MAX_DISPLAY_LINES {
            display.push_str(&format!(
                "\n... ({} more lines truncated, use 's' to save full output)",
                self.stderr.lines().count() - MAX_DISPLAY_LINES
            ));
        }
        display
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        writeln!(file, "# Command: {}", self.command)?;
        writeln!(file, "# Exit code: {:?}", self.exit_code)?;
        writeln!(file)?;
        writeln!(file, "# STDOUT:")?;
        write!(file, "{}", self.stdout)?;
        if !self.stdout.is_empty() && !self.stdout.ends_with('\n') {
            writeln!(file)?;
        }
        writeln!(file)?;
        writeln!(file, "# STDERR:")?;
        write!(file, "{}", self.stderr)?;

        Ok(())
    }
}
