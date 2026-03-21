use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::executor::{self, Job, JobStatus};
use crate::state::State;
use crate::ui::Screen;

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

pub fn render_jobs_panel(
    f: &mut Frame,
    state: &State,
    ui: &mut crate::ui::UiState,
    area: Rect,
) {
    let theme = &state.theme;
    let jobs = executor::list_jobs();

    let title = format!(" Jobs ({} jobs) ", jobs.len());

    let block = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.border));

    f.render_widget(block, area);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);

    if jobs.is_empty() {
        let empty_msg = Paragraph::new("No jobs yet. Run a spell with Alt+Enter to start.")
            .style(Style::new().fg(theme.muted));
        f.render_widget(empty_msg, inner);
        return;
    }

    let visible_height = inner.height as usize;
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

            let elevated_marker = if job.elevated { " ⚡" } else { "" };

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
                Span::raw(elevated_marker.to_string()),
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

    f.render_widget(list, inner);

    let help_text = Line::from(vec![
        Span::raw("["),
        Span::styled("k", Style::new().fg(theme.accent)),
        Span::raw("]ill  ["),
        Span::styled("c", Style::new().fg(theme.accent)),
        Span::raw("]ancel  ["),
        Span::styled("d", Style::new().fg(theme.accent)),
        Span::raw("]ismiss  ["),
        Span::styled("v", Style::new().fg(theme.accent)),
        Span::raw("]iew  ["),
        Span::styled("Esc", Style::new().fg(theme.accent)),
        Span::raw("] close"),
    ]);

    let help_para = Paragraph::new(help_text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::new().fg(theme.muted));

    let help_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
    f.render_widget(help_para, help_area);
}

pub fn handle_jobs_key(
    key: crossterm::event::KeyCode,
    ui: &mut crate::ui::UiState,
) -> bool {
    let jobs = executor::list_jobs();
    let job_ids: Vec<u64> = jobs.iter().map(|j| j.id).collect();

    match key {
        crossterm::event::KeyCode::Esc => {
            ui.screen = Screen::SearchOverlay;
            ui.jobs_panel_state = JobsPanelState::default();
            return false;
        }
        crossterm::event::KeyCode::Up => {
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                if idx > 0 {
                    ui.jobs_panel_state.selected_index = Some(idx - 1);
                } else {
                    ui.jobs_panel_state.selected_index = Some(jobs.len().saturating_sub(1));
                }
            }
            return false;
        }
        crossterm::event::KeyCode::Down => {
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                if idx < jobs.len().saturating_sub(1) {
                    ui.jobs_panel_state.selected_index = Some(idx + 1);
                } else {
                    ui.jobs_panel_state.selected_index = Some(0);
                }
            }
            return false;
        }
        crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Char('K') => {
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                if idx < job_ids.len() {
                    let job_id = job_ids[idx];
                    if let Err(e) = executor::kill_job(job_id) {
                        ui.copy_feedback = Some(format!("Kill failed: {}", e));
                    } else {
                        ui.copy_feedback = Some(format!("Job {} killed", job_id));
                    }
                }
            }
            return false;
        }
        crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                if idx < job_ids.len() {
                    let job_id = job_ids[idx];
                    if let Err(e) = executor::cancel_job(job_id) {
                        ui.copy_feedback = Some(format!("Cancel failed: {}", e));
                    } else {
                        ui.copy_feedback = Some(format!("Job {} cancelled", job_id));
                    }
                }
            }
            return false;
        }
        crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                if idx < job_ids.len() {
                    let job_id = job_ids[idx];
                    if let Err(e) = executor::dismiss_job(job_id) {
                        ui.copy_feedback = Some(format!("Dismiss failed: {}", e));
                    } else {
                        ui.copy_feedback = Some(format!("Job {} dismissed", job_id));
                        if jobs.len() > 1 {
                            ui.jobs_panel_state.selected_index = Some(idx.min(jobs.len() - 2));
                        } else {
                            ui.jobs_panel_state.selected_index = None;
                        }
                    }
                }
            }
            return false;
        }
        crossterm::event::KeyCode::Char('v') | crossterm::event::KeyCode::Char('V') => {
            if let Some(idx) = ui.jobs_panel_state.selected_index {
                if idx < job_ids.len() {
                    let job_id = job_ids[idx];
                    if let Some(job) = executor::get_job(job_id) {
                        let (stdout, stderr) = executor::get_job_output(job_id);
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
            }
            return false;
        }
        _ => {}
    }

    false
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
