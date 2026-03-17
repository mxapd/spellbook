use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Attempts to copy text to clipboard using multiple backends.
///
/// Tries in order:
/// 1. wl-copy (Wayland) - preferred on modern Linux
/// 2. xclip (X11) - classic X11 choice
/// 3. xsel (X11) - lightweight X11 alternative
///
/// Returns true if any method succeeded, false if all failed.
pub fn copy_to_clipboard(text: &str) -> bool {
    let mut success = false;

    // Try wl-copy (Wayland)
    if copy_via_stdin("wl-copy", &[], text) {
        success = true;
    }
    // Try xclip (X11)
    else if copy_via_stdin("xclip", &["-selection", "clipboard"], text) {
        success = true;
    }
    // Try xsel (X11)
    else if copy_via_stdin("xsel", &["--clipboard", "--input"], text) {
        success = true;
    }

    if success {
        send_notification("Copied!", text);
    } else {
        send_notification("Failed to copy", text);
    }

    success
}

/// Sends a system notification via D-Bus using notify-send.
fn send_notification(title: &str, body: &str) {
    let _ = Command::new("notify-send").args([title, body]).spawn();
}

/// Copies text to clipboard via stdin pipe with timeout.
fn copy_via_stdin(program: &str, args: &[&str], text: &str) -> bool {
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: failed to spawn: {}", program, e);
            return false;
        }
    };

    // Write to stdin
    if let Some(ref mut stdin) = child.stdin {
        if let Err(e) = stdin.write_all(text.as_bytes()) {
            eprintln!("{}: failed to write: {}", program, e);
            return false;
        }
    }

    // Wait for completion with a timeout using a separate thread
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let status = child.wait();
        let _ = tx.send(status);
    });

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(status)) => {
            if status.success() {
                true
            } else {
                eprintln!("{}: exited with status: {:?}", program, status);
                false
            }
        }
        Ok(Err(e)) => {
            eprintln!("{}: wait failed: {}", program, e);
            false
        }
        Err(_) => {
            eprintln!("{}: timed out", program);
            false
        }
    }
}
