use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
};

use crate::invoker::{self, Job, JobStatus};
use crate::state::State;
use crate::ui::events;

pub struct JobsPanelState {
    pub selected_index: Option<usize>,
    pub scroll_offset: usize,
    pub job_ids: Vec<u64>,
}

impl Default for JobsPanelState {
    fn default() -> Self {
        Self {
            selected_index: None,
            scroll_offset: 0,
            job_ids: Vec::new(),
        }
    }
}

pub fn render_jobs_panel(f: &mut Frame, state: &State, ui: &mut crate::ui::UiState, area: Rect) {
    let theme = &state.theme;
    let jobs = invoker::list_jobs();

    if jobs.is_empty() {
        let empty_msg = Paragraph::new("No jobs yet")
            .style(Style::new().fg(theme.muted));
        f.render_widget(empty_msg, area);
        return;
    }

    let visible_height = area.height.saturating_sub(1) as usize;
    let total_height = jobs.len();

    if ui.jobs_panel_state.selected_index.is_none() && !jobs.is_empty() {
        ui.jobs_panel_state.selected_index = Some(0);
    }

    let selected = ui.jobs_panel_state.selected_index.unwrap_or(0);

    let start = selected.saturating_sub(visible_height.saturating_sub(1));
    let end = (start + visible_height).min(total_height);
    let visible_jobs: Vec<&Job> = jobs[start..end].iter().collect();

    let items: Vec<ListItem> = visible_jobs
        .iter()
        .enumerate()
        .map(|(i, job)| {
            let global_idx = start + i;
            let is_selected = global_idx == selected;

            let status_icon = match job.status {
                JobStatus::Queued => "○",
                JobStatus::Running => "●",
                JobStatus::Completed => "✓",
                JobStatus::Failed => "✗",
                JobStatus::Cancelled => "⊘",
            };

            let status_style = match job.status {
                JobStatus::Queued => Style::new().fg(theme.muted),
                JobStatus::Running => Style::new().fg(theme.accent),
                JobStatus::Completed => Style::new().fg(theme.accent),
                JobStatus::Failed => Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
                JobStatus::Cancelled => Style::new().fg(theme.muted),
            };

            let duration = if let Some(completed) = job.completed_at {
                let diff = completed.signed_duration_since(job.started_at);
                format_duration(diff.num_seconds())
            } else if job.status == JobStatus::Running {
                let diff = chrono::Utc::now().signed_duration_since(job.started_at);
                format_duration(diff.num_seconds())
            } else {
                String::new()
            };

            let exit_info = match (job.status.clone(), job.exit_code) {
                (JobStatus::Completed, Some(code)) => format!(" [{}]", code),
                (JobStatus::Failed, Some(code)) => format!(" [{}]", code),
                _ => String::new(),
            };

            let name_style = if is_selected {
                Style::new().fg(theme.selection)
            } else {
                Style::new().fg(theme.fg)
            };

            let status_str = job.status.to_string().to_lowercase();
            let duration_str = duration.clone();
            let exit_str = exit_info.clone();

            let line = Line::from(vec![
                Span::raw(status_icon.to_string()),
                Span::raw(" ".to_string()),
                Span::styled(job.name.clone(), name_style),
                Span::raw("  ".to_string()),
                Span::styled(status_str, status_style),
                Span::raw("  ".to_string()),
                Span::raw(duration_str),
                Span::raw(exit_str),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .style(Style::new().bg(theme.bg));

    let list_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(2));
    f.render_widget(list, list_area);

    let help_text = Line::from(vec![
        Span::styled("[k]ill", Style::new().fg(theme.accent)),
        Span::raw("  "),
        Span::styled("[c]ancel", Style::new().fg(theme.accent)),
        Span::raw("  "),
        Span::styled("[d]ismiss", Style::new().fg(theme.accent)),
        Span::raw("  "),
        Span::styled("[v]iew", Style::new().fg(theme.accent)),
    ]);
    let help_para = Paragraph::new(help_text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::new().fg(theme.muted));
    let help_area = Rect::new(area.x, area.y + area.height.saturating_sub(2), area.width, 1);
    f.render_widget(help_para, help_area);
}

pub fn handle_jobs_key(
    key: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
    ui: &mut crate::ui::UiState,
) -> bool {
    // Check Ctrl+C first - should work even in sidebar
    if key == crossterm::event::KeyCode::Char('c')
        && modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
    {
        return true; // Signal to quit
    }

    match key {
        crossterm::event::KeyCode::Esc => {
            // Close jobs sidebar if open, otherwise nothing (Esc handled by mode handler)
            if ui.jobs_sidebar_open {
                ui.jobs_sidebar_open = false;
                ui.focus = crate::models::FocusTarget::Main;
            }
            false
        }
        crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Down => {
            let jobs = invoker::list_jobs();
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                let new_idx = match key {
                    crossterm::event::KeyCode::Up => {
                        if idx > 0 {
                            idx - 1
                        } else {
                            jobs.len().saturating_sub(1)
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        if idx < jobs.len().saturating_sub(1) {
                            idx + 1
                        } else {
                            0
                        }
                    }
                    _ => idx,
                };
                ui.jobs_panel_state.selected_index = Some(new_idx);
            }
            false
        }
        crossterm::event::KeyCode::Char('k')
        | crossterm::event::KeyCode::Char('K')
        | crossterm::event::KeyCode::Char('c')
        | crossterm::event::KeyCode::Char('C')
        | crossterm::event::KeyCode::Char('d')
        | crossterm::event::KeyCode::Char('D')
        | crossterm::event::KeyCode::Char('v')
        | crossterm::event::KeyCode::Char('V')
        | crossterm::event::KeyCode::Enter => {
            let jobs = invoker::list_jobs();
            let job_ids: Vec<u64> = jobs.iter().map(|j| j.id).collect();
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                if idx < job_ids.len() {
                    let job_id = job_ids[idx];
                    match key {
                        crossterm::event::KeyCode::Char('k')
                        | crossterm::event::KeyCode::Char('K') => {
                            if let Err(e) = invoker::kill_job(job_id) {
                                ui.copy_feedback = Some(format!("Kill failed: {}", e));
                            } else {
                                ui.copy_feedback = Some(format!("Job {} killed", job_id));
                            }
                        }
                        crossterm::event::KeyCode::Char('c')
                        | crossterm::event::KeyCode::Char('C') => {
                            if let Err(e) = invoker::cancel_job(job_id) {
                                ui.copy_feedback = Some(format!("Cancel failed: {}", e));
                            } else {
                                ui.copy_feedback = Some(format!("Job {} cancelled", job_id));
                            }
                        }
                        crossterm::event::KeyCode::Char('d')
                        | crossterm::event::KeyCode::Char('D') => {
                            if let Err(e) = invoker::dismiss_job(job_id) {
                                ui.copy_feedback = Some(format!("Dismiss failed: {}", e));
                            } else {
                                ui.copy_feedback = Some(format!("Job {} dismissed", job_id));
                                if jobs.len() > 1 {
                                    ui.jobs_panel_state.selected_index =
                                        Some(idx.min(jobs.len() - 2));
                                } else {
                                    ui.jobs_panel_state.selected_index = None;
                                }
                            }
                        }
                        crossterm::event::KeyCode::Char('v')
                        | crossterm::event::KeyCode::Char('V')
                        | crossterm::event::KeyCode::Enter => {
                            if let Some(job) = invoker::get_job(job_id) {
                                let (stdout, stderr) = invoker::get_job_output(job_id);
                                let result = crate::clipboard::ExecutionResult {
                                    command: job.command.clone(),
                                    stdout: stdout.clone(),
                                    stderr: stderr.clone(),
                                    exit_code: job.exit_code,
                                    full_stdout: stdout,
                                    full_stderr: stderr,
                                    pid: job.pid,
                                    spell_name: Some(job.name.clone()),
                                };
                                ui.show_output_popup(result);
                            }
                        }
                        _ => {}
                    }
                }
            }
            false
        }
        crossterm::event::KeyCode::Char('/') => {
            // Switch focus to main and open search
            ui.focus = crate::models::FocusTarget::Main;
            ui.open_search();
            false
        }
        crossterm::event::KeyCode::Char(':') => {
            // Switch focus to main and open command search
            ui.focus = crate::models::FocusTarget::Main;
            ui.open_search();
            if let Some(query) = ui.search_query_mut() {
                query.push(':');
            }
            events::update_command_filter(ui);
            false
        }
        _ => false,
    }
}

fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

impl JobStatus {
    pub fn to_string(&self) -> String {
        match self {
            JobStatus::Queued => "queued".to_string(),
            JobStatus::Running => "running".to_string(),
            JobStatus::Completed => "completed".to_string(),
            JobStatus::Failed => "failed".to_string(),
            JobStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::UiState;
    use crossterm::event::KeyCode;

    #[test]
    fn test_jobs_panel_state_default() {
        let state = JobsPanelState::default();
        assert!(state.selected_index.is_none());
        assert_eq!(state.scroll_offset, 0);
        assert!(state.job_ids.is_empty());
    }

    #[test]
    fn test_jobs_panel_state_with_values() {
        let state = JobsPanelState {
            selected_index: Some(5),
            scroll_offset: 2,
            job_ids: vec![1, 2, 3],
        };
        assert_eq!(state.selected_index, Some(5));
        assert_eq!(state.scroll_offset, 2);
        assert_eq!(state.job_ids.len(), 3);
    }

    #[test]
    fn test_job_status_to_string() {
        assert_eq!(JobStatus::Queued.to_string(), "queued");
        assert_eq!(JobStatus::Running.to_string(), "running");
        assert_eq!(JobStatus::Completed.to_string(), "completed");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
        assert_eq!(JobStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_handle_jobs_key_escape_closes_sidebar() {
        let mut ui = UiState::new(false);
        ui.jobs_sidebar_open = true;
        ui.focus = crate::models::FocusTarget::JobsSidebar;

        handle_jobs_key(KeyCode::Esc, crossterm::event::KeyModifiers::empty(), &mut ui);

        assert!(!ui.jobs_sidebar_open);
        assert_eq!(ui.focus, crate::models::FocusTarget::Main);
    }

    #[test]
    fn test_handle_jobs_key_escape_does_nothing_when_closed() {
        let mut ui = UiState::new(false);
        ui.jobs_sidebar_open = false;

        handle_jobs_key(KeyCode::Esc, crossterm::event::KeyModifiers::empty(), &mut ui);

        assert!(!ui.jobs_sidebar_open);
    }
}
