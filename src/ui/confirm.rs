use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::models::Spell;
use crate::state::State;
use crate::ui::Screen;

pub struct ConfirmDialogState {
    pub spell: Spell,
    pub typed_confirmation: String,
}

pub fn render_confirm_dialog(
    f: &mut Frame,
    state: &State,
    ui: &crate::ui::UiState,
    area: Rect,
) {
    let theme = &state.theme;
    let dialog = ui.confirm_dialog.as_ref().unwrap();

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent));

    f.render_widget(&block, area);

    let inner = block.inner(area);
    let inner_width = inner.width as usize;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let line1 = Line::from(vec![
        Span::raw("Execute "),
        Span::styled("elevated", Style::new().fg(theme.accent)),
        Span::raw(" command?"),
    ]);
    let para1 = Paragraph::new(line1).style(Style::new().fg(theme.fg));
    f.render_widget(para1, chunks[0]);

    let line2 = Line::from(vec![
        Span::styled("Spell: ", Style::new().fg(theme.muted)),
        Span::styled(&dialog.spell.name, Style::new().fg(theme.fg).add_modifier(Modifier::BOLD)),
    ]);
    let para2 = Paragraph::new(line2);
    f.render_widget(para2, chunks[1]);

    let line3 = Line::from(vec![
        Span::styled("Command: ", Style::new().fg(theme.muted)),
    ]);
    let cmd_display = truncate_string(&dialog.spell.incantation, inner_width.saturating_sub(10));
    let cmd_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(&cmd_display, Style::new().fg(theme.accent)),
    ]);
    let para3 = Paragraph::new(vec![line3, cmd_line]);
    f.render_widget(para3, chunks[2]);

    let instruction = if dialog.spell.elevated {
        "Type 'yes' or press Enter to confirm, Esc to cancel"
    } else {
        "Press Enter to confirm, Esc to cancel"
    };
    let line5 = Line::from(vec![
        Span::styled(instruction, Style::new().fg(theme.muted)),
    ]);
    let para5 = Paragraph::new(line5);
    f.render_widget(para5, chunks[5]);
}

pub fn handle_confirm_key(
    key: crossterm::event::KeyCode,
    _c: Option<char>,
    ui: &mut crate::ui::UiState,
    _state: &mut State,
) -> Option<bool> {
    let dialog = ui.confirm_dialog.as_mut()?;

    match key {
        crossterm::event::KeyCode::Esc => {
            ui.confirm_dialog = None;
            ui.screen = Screen::SearchOverlay;
            ui.copy_feedback = Some("Cancelled".to_string());
            return Some(false);
        }
        crossterm::event::KeyCode::Enter => {
            let spell = dialog.spell.clone();
            ui.confirm_dialog = None;

            if spell.background {
                // Run as background job
                match crate::invoker::start_spell(
                    spell.name.clone(),
                    spell.incantation.clone(),
                    spell.elevated,
                    if spell.working_dir.is_empty() { None } else { Some(spell.working_dir.clone()) },
                ) {
                    Ok(job_id) => {
                        ui.copy_feedback = Some(format!("Job {} started: {}", job_id, spell.name));
                        ui.screen = Screen::SearchOverlay;
                    }
                    Err(e) => {
                        ui.copy_feedback = Some(format!("Failed to start: {}", e));
                        ui.screen = Screen::SearchOverlay;
                    }
                }
            } else {
                // Run synchronously and show output
                let result = crate::invoker::execute_sync(&spell.incantation, spell.elevated);
                let exec_result = crate::clipboard::ExecutionResult {
                    command: spell.incantation.clone(),
                    stdout: result.stdout.clone(),
                    stderr: result.stderr.clone(),
                    exit_code: result.exit_code,
                    full_stdout: result.stdout,
                    full_stderr: result.stderr,
                    pid: Some(result.pid),
                    spell_name: Some(spell.name.clone()),
                };
                ui.show_output_popup(exec_result);
            }
            return Some(false);
        }
        crossterm::event::KeyCode::Backspace => {
            dialog.typed_confirmation.pop();
            return Some(false);
        }
        crossterm::event::KeyCode::Char(ch) => {
            dialog.typed_confirmation.push(ch);
            return Some(false);
        }
        _ => {}
    }

    None
}

fn truncate_string(s: &str, max_width: usize) -> String {
    if max_width < 4 {
        return String::new();
    }

    let visual_width: usize = s.chars().map(|c| if c == '\t' { 8 } else { 1 }).sum();
    if visual_width <= max_width {
        return s.to_string();
    }

    let available = max_width.saturating_sub(3);
    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let char_width = if c == '\t' { 8 } else { 1 };
        if current_width + char_width > available {
            break;
        }
        result.push(c);
        current_width += char_width;
    }

    result + "..."
}
