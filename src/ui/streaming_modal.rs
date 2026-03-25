//! Streaming Output Modal
//!
//! This module provides real-time command output streaming for the TUI execution mode.
//! It manages a child process with captured stdout/stderr, displays output in real-time,
//! and provides controls to interact with running processes.
//!
//! # Architecture
//!
//! The streaming system uses a multi-threaded approach:
//! 1. **Main Thread**: UI rendering and event handling
//! 2. **Process Thread**: Manages the child process
//! 3. **Reader Threads**: Separate threads for stdout and stderr (spawned by invoker)
//! 4. **mpsc Channel**: Communicates output lines from reader threads to main thread
//!
//! # Flow
//!
//! 1. User triggers TUI mode execution (e.g., `Ctrl+r` or spell with `run_mode = Tui`)
//! 2. `start_tui_execution()` resets state and spawns process via `invoker::stream_command()`
//! 3. Background threads read output lines from pipes
//! 4. Lines sent via mpsc channel to main event loop
//! 5. Event loop polls channel every 100ms (on timeout) via `poll_stream_output()`
//! 6. UI updates in real-time with auto-scroll to bottom
//! 7. Process completion detected by empty line signal or error
//!
//! # Controls
//!
//! - `Ctrl+C`: Kill the running process
//! - `Ctrl+B`: Promote to background (restarts as detached job)
//! - `Esc`: Close modal (only when process finished)
//! - `s`: Toggle auto-scroll
//! - `↑/↓`: Manual scroll (disables auto-scroll)
//!
//! # Output Buffer
//!
//! Output is stored in `OutputModalState::content` with a 10,000 line cap.
//! When the limit is reached, oldest lines are removed (FIFO eviction).
//! A truncation warning appears in the footer when this happens.

use crate::invoker::StreamOutput;
use crate::state::OutputModalState;
use crate::ui::UiState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Streaming state for real-time output
#[derive(Debug, Clone)]
pub struct StreamingState {
    pub pid: Option<u32>,
    pub is_running: bool,
    pub command: String,
    pub spell_name: Option<String>,
    pub working_dir: Option<String>,
}

impl StreamingState {
    pub fn new(command: String, spell_name: Option<String>, working_dir: Option<String>) -> Self {
        Self {
            pid: None,
            is_running: true,
            command,
            spell_name,
            working_dir,
        }
    }
}

/// Extended output modal state with streaming support
pub struct StreamingModalState {
    pub output: OutputModalState,
    pub streaming: Option<StreamingState>,
    pub auto_scroll: bool,
    pub receiver: Option<std::sync::mpsc::Receiver<StreamOutput>>,
}

impl Default for StreamingModalState {
    fn default() -> Self {
        Self {
            output: OutputModalState::default(),
            streaming: None,
            auto_scroll: true,
            receiver: None,
        }
    }
}

impl StreamingModalState {
    pub fn start_streaming(
        &mut self,
        command: String,
        spell_name: Option<String>,
        working_dir: Option<String>,
    ) -> std::io::Result<u32> {
        use crate::invoker::{stream_command, StreamOutput};

        self.output.is_streaming = true;
        self.output.exit_code = None;
        self.auto_scroll = true;

        let (pid, _handle, receiver) = stream_command(&command, working_dir.as_deref())?;
        self.receiver = Some(receiver);

        self.streaming = Some(StreamingState::new(command, spell_name, working_dir));
        self.streaming.as_mut().unwrap().pid = Some(pid);

        Ok(pid)
    }

    pub fn stop_streaming(&mut self, exit_code: Option<i32>) {
        if let Some(ref mut stream) = self.streaming {
            stream.is_running = false;
        }
        self.output.is_streaming = false;
        self.output.exit_code = exit_code;
    }

    pub fn kill(&mut self) -> Result<(), std::io::Error> {
        if let Some(pid) = self.streaming.as_ref().and_then(|s| s.pid) {
            crate::invoker::kill_process(pid)?;
            self.stop_streaming(Some(-1));
            self.output.add_line("[Process killed]".to_string());
        }
        Ok(())
    }

    pub fn promote_to_background(&mut self) -> Result<u64, crate::invoker::ExecutorError> {
        if let Some(ref stream) = self.streaming {
            if stream.is_running {
                // Kill the current process
                if let Some(pid) = stream.pid {
                    let _ = crate::invoker::kill_process(pid);
                }

                // Start as background job
                let job_id = crate::invoker::start_spell(
                    stream
                        .spell_name
                        .clone()
                        .unwrap_or_else(|| "Command".to_string()),
                    stream.command.clone(),
                    stream.working_dir.clone(),
                )?;

                self.stop_streaming(None);
                self.output
                    .add_line(format!("[Moved to background job {}]", job_id));

                return Ok(job_id);
            }
        }
        Err(crate::invoker::ExecutorError::JobNotFound(0))
    }

    pub fn is_running(&self) -> bool {
        self.streaming
            .as_ref()
            .map(|s| s.is_running)
            .unwrap_or(false)
    }

    pub fn get_pid(&self) -> Option<u32> {
        self.streaming.as_ref().and_then(|s| s.pid)
    }
}

/// Start TUI execution with streaming
pub fn start_tui_execution(
    ui: &mut UiState,
    command: String,
    spell_name: Option<String>,
    working_dir: Option<String>,
) -> Result<u32, std::io::Error> {
    use crate::ui::Overlay;

    // Reset the modal state
    ui.streaming_modal = StreamingModalState::default();

    // Start streaming
    let pid = ui
        .streaming_modal
        .start_streaming(command, spell_name, working_dir)?;

    // Push the output modal overlay
    ui.push_overlay(Overlay::OutputModal);

    Ok(pid)
}

/// Poll the stream receiver and update the modal state
pub fn poll_stream_output(ui: &mut UiState) {
    let receiver = ui.streaming_modal.receiver.take();
    if let Some(mut receiver) = receiver {
        let mut stop = false;
        let mut lines = Vec::new();

        while let Ok(output) = receiver.try_recv() {
            if output.line.is_empty() {
                stop = true;
            } else {
                let prefix = if output.is_stderr { "[stderr] " } else { "" };
                lines.push(format!("{}{}", prefix, output.line));
            }
        }

        ui.streaming_modal.receiver = Some(receiver);

        if stop {
            ui.streaming_modal.stop_streaming(Some(0));
        }
        for line in lines {
            ui.streaming_modal.output.add_line(line);
        }
    }
}

/// Render the streaming output modal
pub fn render_streaming_modal(
    frame: &mut Frame,
    ui: &UiState,
    theme: &crate::models::RatatuiColors,
) {
    let area = frame.area();
    render_streaming_modal_in_area(frame, ui, theme, area);
}

/// Render the streaming output modal in a specific area
pub fn render_streaming_modal_in_area(
    frame: &mut Frame,
    ui: &UiState,
    theme: &crate::models::RatatuiColors,
    area: Rect,
) {
    // Full-screen overlay
    let overlay = Paragraph::new("").style(Style::new().bg(theme.bg));
    frame.render_widget(overlay, area);

    // Calculate popup dimensions
    let max_width = area.width.saturating_sub(4).max(20);
    let max_height = area.height.saturating_sub(4).max(6);
    let popup_width = max_width.min(80);
    let popup_height = max_height.min(area.height.saturating_sub(2)).max(6);

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    let state = &ui.streaming_modal;
    let is_running = state.is_running();
    let is_truncated = state.output.is_truncated();
    let exit_code = state.output.exit_code;

    // Status indicator for title
    let status_indicator = if is_running {
        " ⟳ ".to_string()
    } else {
        match exit_code {
            Some(0) => " ✓ ".to_string(),
            Some(_) => " ✗ ".to_string(),
            None => " ? ".to_string(),
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent))
        .title(format!("{}Output", status_indicator))
        .title_style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD));

    frame.render_widget(&block, popup_area);

    let inner = block.inner(popup_area);
    let inner_width = inner.width as usize;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Command line
            Constraint::Min(1),    // Output area
            Constraint::Length(1), // Footer
        ])
        .split(inner);

    // Command line
    let cmd_text = Line::from(vec![
        Span::raw("$ "),
        Span::styled(
            truncate_string(
                &state
                    .streaming
                    .as_ref()
                    .map(|s| s.command.clone())
                    .unwrap_or_default(),
                inner_width.saturating_sub(3),
            ),
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
        ),
    ]);
    let cmd_para = Paragraph::new(cmd_text).style(Style::new().bg(theme.bg).fg(theme.fg));
    frame.render_widget(cmd_para, layout[0]);

    // Output area with scrolling
    let output_area_height = layout[1].height as usize;
    let scroll_offset = if state.auto_scroll && state.output.content.len() > output_area_height {
        state
            .output
            .content
            .len()
            .saturating_sub(output_area_height)
    } else {
        state.output.scroll_offset
    };

    let visible_lines: Vec<Line> = state
        .output
        .content
        .iter()
        .skip(scroll_offset)
        .take(output_area_height)
        .map(|line| {
            let style = if line.starts_with("[stderr] ") {
                Style::new().fg(Color::Indexed(196)) // Red for stderr
            } else if line.starts_with('[') && line.ends_with(']') {
                Style::new().fg(theme.muted) // Muted for system messages
            } else {
                Style::new().fg(theme.fg)
            };
            Line::from(vec![Span::styled(
                truncate_string(line, inner_width),
                style,
            )])
        })
        .collect();

    let output_text = Text::from(visible_lines);
    let output_para = Paragraph::new(output_text)
        .style(Style::new().bg(theme.bg))
        .wrap(Wrap { trim: false });
    frame.render_widget(output_para, layout[1]);

    // Footer with hints
    let mut footer_parts = vec![];

    if is_running {
        footer_parts.push(("Ctrl+C", "Kill"));
        footer_parts.push(("Ctrl+B", "Background"));
    }

    footer_parts.push(("Esc", "Close"));

    if is_truncated {
        footer_parts.push(("[!]", "Truncated"));
    }

    let footer_spans: Vec<Span> = footer_parts
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut spans = vec![];
            if i > 0 {
                spans.push(Span::raw(" | "));
            }
            spans.push(Span::styled(
                *key,
                Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(format!(" {}", desc)));
            spans
        })
        .collect();

    let footer = Line::from(footer_spans);
    let footer_para = Paragraph::new(footer)
        .alignment(Alignment::Center)
        .style(Style::new().fg(theme.muted));
    frame.render_widget(footer_para, layout[2]);
}

/// Handle key events for the streaming modal
/// Returns true if the modal should close
pub fn handle_streaming_modal_key(
    key: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
    ui: &mut UiState,
) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    let state = &mut ui.streaming_modal;

    match key {
        // Kill running process
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            if state.is_running() {
                if let Err(e) = state.kill() {
                    state.output.add_line(format!("[Failed to kill: {}]", e));
                }
            }
            false
        }

        // Promote to background
        KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
            if state.is_running() {
                match state.promote_to_background() {
                    Ok(job_id) => {
                        state
                            .output
                            .add_line(format!("[Moved to background job {}]", job_id));
                        return true; // Close modal
                    }
                    Err(e) => {
                        state
                            .output
                            .add_line(format!("[Failed to move to background: {}]", e));
                    }
                }
            }
            false
        }

        // Close modal
        KeyCode::Esc => {
            if !state.is_running() {
                ui.pop_overlay();
                return true;
            }
            false
        }

        // Toggle auto-scroll
        KeyCode::Char('s') => {
            state.auto_scroll = !state.auto_scroll;
            false
        }

        // Scroll up
        KeyCode::Up => {
            if state.output.scroll_offset > 0 {
                state.output.scroll_offset -= 1;
                state.auto_scroll = false;
            }
            false
        }

        // Scroll down
        KeyCode::Down => {
            let output_height = 20; // Approximate
            if state.output.scroll_offset + output_height < state.output.content.len() {
                state.output.scroll_offset += 1;
            }
            false
        }

        _ => false,
    }
}

fn truncate_string(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else {
        s.chars()
            .take(max_width.saturating_sub(3))
            .collect::<String>()
            + "..."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_state_new() {
        let state = StreamingState::new(
            "echo hello".to_string(),
            Some("Test Spell".to_string()),
            Some("/tmp".to_string()),
        );

        assert_eq!(state.command, "echo hello");
        assert_eq!(state.spell_name, Some("Test Spell".to_string()));
        assert_eq!(state.working_dir, Some("/tmp".to_string()));
        assert!(state.is_running);
        assert!(state.pid.is_none());
    }

    #[test]
    fn test_streaming_modal_state_default() {
        let state = StreamingModalState::default();

        assert!(state.streaming.is_none());
        assert!(state.auto_scroll);
        assert!(!state.output.is_streaming);
        assert!(state.output.exit_code.is_none());
        assert!(state.output.content.is_empty());
    }

    #[test]
    fn test_stop_streaming() {
        let mut state = StreamingModalState::default();
        state.streaming = Some(StreamingState::new("echo test".to_string(), None, None));
        state.output.is_streaming = true;

        state.stop_streaming(Some(0));

        assert!(!state.output.is_streaming);
        assert_eq!(state.output.exit_code, Some(0));
        assert!(state
            .streaming
            .as_ref()
            .map(|s| !s.is_running)
            .unwrap_or(false));
    }

    #[test]
    fn test_is_running() {
        let mut state = StreamingModalState::default();

        // Not running when no streaming state
        assert!(!state.is_running());

        // Running when streaming is active
        state.streaming = Some(StreamingState::new("sleep 10".to_string(), None, None));
        assert!(state.is_running());

        // Not running after stopped
        state.stop_streaming(Some(0));
        assert!(!state.is_running());
    }

    #[test]
    fn test_get_pid() {
        let mut state = StreamingModalState::default();

        // No PID when not streaming
        assert!(state.get_pid().is_none());

        // Has PID when streaming with PID set
        let mut streaming = StreamingState::new("echo test".to_string(), None, None);
        streaming.pid = Some(12345);
        state.streaming = Some(streaming);

        assert_eq!(state.get_pid(), Some(12345));
    }

    #[test]
    fn test_kill_sets_exit_code() {
        let mut state = StreamingModalState::default();
        state.streaming = Some(StreamingState::new("sleep 10".to_string(), None, None));
        state.output.is_streaming = true;

        // Since we can't actually test process killing in unit tests,
        // we verify the state changes correctly
        state.stop_streaming(Some(-1));

        assert!(!state.is_running());
        assert_eq!(state.output.exit_code, Some(-1));
    }

    #[test]
    fn test_auto_scroll_toggle() {
        let mut state = StreamingModalState::default();

        assert!(state.auto_scroll);

        state.auto_scroll = !state.auto_scroll;
        assert!(!state.auto_scroll);

        state.auto_scroll = !state.auto_scroll;
        assert!(state.auto_scroll);
    }

    #[test]
    fn test_truncate_string_short() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("test", 4), "test");
    }

    #[test]
    fn test_truncate_string_long() {
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
    }

    #[test]
    fn test_truncate_string_exact() {
        assert_eq!(truncate_string("exact", 5), "exact");
        assert_eq!(truncate_string("sixchr", 6), "sixchr");
    }

    #[test]
    fn test_truncate_string_unicode() {
        // Test with unicode characters
        assert_eq!(truncate_string("hello 世界", 10), "hello 世界");
        assert_eq!(truncate_string("hello 世界wide", 10), "hello 世...");
    }

    #[test]
    fn test_streaming_state_fields() {
        let state = StreamingState {
            pid: Some(42),
            is_running: true,
            command: "ls -la".to_string(),
            spell_name: Some("List Files".to_string()),
            working_dir: Some("/home".to_string()),
        };

        assert_eq!(state.pid, Some(42));
        assert!(state.is_running);
        assert_eq!(state.command, "ls -la");
        assert_eq!(state.spell_name, Some("List Files".to_string()));
        assert_eq!(state.working_dir, Some("/home".to_string()));
    }

    #[test]
    fn test_streaming_with_none_working_dir() {
        let state = StreamingState::new(
            "pwd".to_string(),
            Some("Current Directory".to_string()),
            None,
        );

        assert!(state.working_dir.is_none());
        assert_eq!(state.command, "pwd");
    }

    #[test]
    fn test_streaming_with_none_spell_name() {
        let state = StreamingState::new("echo anonymous".to_string(), None, None);

        assert!(state.spell_name.is_none());
        assert_eq!(state.command, "echo anonymous");
    }
}
