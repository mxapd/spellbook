use std::process::Command;

const MAX_DISPLAY_LINES: usize = 500;

pub fn is_clipboard_available() -> bool {
    true
}

pub fn copy_to_clipboard(text: &str) -> bool {
    // Escape single quotes for shell
    let escaped = text.replace("'", "'\\''");

    // Try wl-copy first (Wayland native) - non-blocking
    if Command::new("sh")
        .args(["-c", &format!("printf '%s' '{}' | wl-copy", escaped)])
        .spawn()
        .is_ok()
    {
        // Spawn notification, don't wait
        let _ = Command::new("notify-send").args(["Copied!", text]).spawn();
        return true;
    }

    // Fallback to xclip (X11) - non-blocking
    if Command::new("sh")
        .args([
            "-c",
            &format!("printf '%s' '{}' | xclip -selection clipboard", escaped),
        ])
        .spawn()
        .is_ok()
    {
        // Spawn notification, don't wait
        let _ = Command::new("notify-send").args(["Copied!", text]).spawn();
        return true;
    }

    false
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
