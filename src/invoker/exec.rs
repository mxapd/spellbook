use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::sync::mpsc;

// ============================================================================
// Synchronous execution
// ============================================================================

#[derive(Debug, Clone)]
pub struct SyncExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub pid: u32,
}

pub fn execute_sync(command: &str) -> SyncExecutionResult {
    let output = Command::new("bash").arg("-c").arg(command).output();

    match output {
        Ok(out) => SyncExecutionResult {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code(),
            pid: 0,
        },
        Err(e) => SyncExecutionResult {
            stdout: String::new(),
            stderr: format!("Failed to execute: {}", e),
            exit_code: Some(127),
            pid: 0,
        },
    }
}

pub fn execute_sync_spawn(command: &str) -> std::io::Result<(SyncExecutionResult, u32)> {
    let child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id();

    let output = child.wait_with_output()?;

    let result = SyncExecutionResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
        pid,
    };

    Ok((result, pid))
}

pub fn kill_process(pid: u32) -> Result<(), std::io::Error> {
    Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .output()?;
    Ok(())
}

// ============================================================================
// Shell detection
// ============================================================================

pub fn detect_shell_static() -> String {
    let shells = vec![
        "/bin/bash",
        "/usr/bin/bash",
        "/run/current-system/sw/bin/bash",
        "/bin/sh",
        "/usr/bin/sh",
        "/run/current-system/sw/bin/sh",
    ];

    for shell in shells {
        if std::path::Path::new(shell).exists() {
            return shell.to_string();
        }
    }

    if let Ok(path) = std::process::Command::new("which").arg("bash").output() {
        let output = String::from_utf8_lossy(&path.stdout);
        let shell = output.trim();
        if !shell.is_empty() && std::path::Path::new(shell).exists() {
            return shell.to_string();
        }
    }

    "/bin/sh".to_string()
}

// ============================================================================
// Streaming output
// ============================================================================

pub struct StreamOutput {
    pub line: String,
    pub is_stderr: bool,
}

pub fn stream_command(
    command: &str,
    working_dir: Option<&str>,
    launch_dir: &str,
) -> std::io::Result<(
    u32,
    thread::JoinHandle<()>,
    mpsc::Receiver<StreamOutput>,
)> {
    let shell = detect_shell_static();

    let mut cmd = Command::new(&shell);
    cmd.arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let default_dir = if std::path::Path::new(launch_dir).exists() {
        launch_dir.to_string()
    } else {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    };

    let work_dir = working_dir
        .filter(|d| !d.is_empty())
        .map(|d| d.to_string())
        .unwrap_or(default_dir);

    let dir = PathBuf::from(work_dir);
    if dir.exists() && dir.is_dir() {
        cmd.current_dir(dir);
    }

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let pid = child.id();

    let (tx, rx) = mpsc::channel::<StreamOutput>();
    let tx_for_stream = tx.clone();

    let handle = thread::spawn(move || {
        let mut reader_threads = vec![];

        if let Some(stdout) = stdout {
            let tx_clone = tx_for_stream.clone();
            let t = thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_clone.send(StreamOutput {
                        line,
                        is_stderr: false,
                    });
                }
            });
            reader_threads.push(t);
        }

        if let Some(stderr) = stderr {
            let tx_clone = tx_for_stream.clone();
            let t = thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_clone.send(StreamOutput {
                        line,
                        is_stderr: true,
                    });
                }
            });
            reader_threads.push(t);
        }

        for t in reader_threads {
            let _ = t.join();
        }

        let _ = tx_for_stream.send(StreamOutput {
            line: String::new(),
            is_stderr: false,
        });
    });

    Ok((pid, handle, rx))
}

// ============================================================================
// Simple execution (replaces process on Unix)
// ============================================================================

#[cfg(unix)]
pub fn exec_simple(command: &str, working_dir: Option<&str>, launch_dir: &str) -> ! {
    use std::env;
    use std::io::Write;

    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let default_dir = if std::path::Path::new(launch_dir).exists() {
        launch_dir.to_string()
    } else {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    };

    let work_dir = working_dir
        .filter(|d| !d.is_empty())
        .map(|d| d.to_string())
        .unwrap_or(default_dir);

    let _ = std::env::set_current_dir(work_dir);

    let program = std::path::Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("sh");

    // Leave alternate screen and reset SGR before exec
    let cleanup = "\x1b[?1049l\x1b[0m";
    let _ = std::io::stdout().write_all(cleanup.as_bytes());
    let _ = std::io::stdout().flush();

    let _error = exec::execvp(&shell, &[program, "-c", command]);
    std::process::exit(1)
}

#[cfg(not(unix))]
pub fn exec_simple(command: &str, working_dir: Option<&str>, launch_dir: &str) -> i32 {
    use std::env;

    let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

    let default_dir = if std::path::Path::new(launch_dir).exists() {
        launch_dir
    } else {
        env::var("HOME").unwrap_or_else(|_| "/").as_str()
    };

    let mut cmd = Command::new(&shell);
    cmd.arg("-c").arg(command);

    let work_dir = working_dir
        .filter(|d| !d.is_empty() && std::path::Path::new(d).exists())
        .unwrap_or(default_dir);

    cmd.current_dir(work_dir);
    cmd.env("HOME", env::var("HOME").unwrap_or_else(|_| "/".to_string()));

    match cmd.status() {
        Ok(status) => status.code().unwrap_or(0),
        Err(e) => {
            eprintln!("Failed to execute: {}", e);
            1
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // === execute_sync ===

    #[test]
    fn captures_stdout() {
        let result = execute_sync("echo hello");
        assert_eq!(result.stdout.trim(), "hello");
        assert!(result.stderr.is_empty());
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn captures_stderr() {
        let result = execute_sync("bash -c 'echo error >&2'");
        assert_eq!(result.stderr.trim(), "error");
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn exit_code_success() {
        let result = execute_sync("true");
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn exit_code_failure() {
        let result = execute_sync("false");
        assert_eq!(result.exit_code, Some(1));
    }

    #[test]
    fn handles_invalid_command() {
        let result = execute_sync("nonexistent_command_12345");
        assert!(result.exit_code.is_some());
        assert!(result.exit_code.unwrap() != 0 || !result.stderr.is_empty());
    }

    #[test]
    fn multiline_output() {
        let result = execute_sync("echo line1 && echo line2 && echo line3");
        let lines: Vec<&str> = result.stdout.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn special_characters() {
        let result = execute_sync(r#"echo "hello world" '#'"#);
        assert!(result.stdout.contains("hello world"));
    }
}
