use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::models::{RunMode, Spell};

#[derive(Debug, Clone)]
pub struct ConfirmDialogState {
    pub spell: Spell,
    pub typed_confirmation: String,
}

impl Default for ConfirmDialogState {
    fn default() -> Self {
        Self {
            spell: Spell::default(),
            typed_confirmation: String::new(),
        }
    }
}

impl ConfirmDialogState {
    pub fn new(spell: Spell) -> Self {
        Self {
            spell,
            typed_confirmation: String::new(),
        }
    }

    pub fn requires_typed_confirmation(&self) -> bool {
        self.spell.confirm
    }
}

pub fn render_confirm_dialog(
    f: &mut Frame,
    area: Rect,
    theme: &crate::models::RatatuiColors,
    dialog: &ConfirmDialogState,
) {
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

    let line1 = if dialog.spell.confirm {
        Line::from(vec![
            Span::raw("Confirm "),
            Span::styled("execution", Style::new().fg(theme.accent)),
            Span::raw("?"),
        ])
    } else {
        Line::from(vec![
            Span::raw("Execute "),
            Span::styled("command", Style::new().fg(theme.accent)),
            Span::raw("?"),
        ])
    };
    let para1 = Paragraph::new(line1).style(Style::new().fg(theme.fg));
    f.render_widget(para1, chunks[0]);

    let line2 = Line::from(vec![
        Span::styled("Spell: ", Style::new().fg(theme.muted)),
        Span::styled(
            &dialog.spell.name,
            Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]);
    let para2 = Paragraph::new(line2);
    f.render_widget(para2, chunks[1]);

    let line3 = Line::from(vec![Span::styled(
        "Command: ",
        Style::new().fg(theme.muted),
    )]);
    let cmd_display = truncate_string(&dialog.spell.incantation, inner_width.saturating_sub(10));
    let cmd_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(&cmd_display, Style::new().fg(theme.accent)),
    ]);
    let para3 = Paragraph::new(vec![line3, cmd_line]);
    f.render_widget(para3, chunks[2]);

    let run_mode_hint = match dialog.spell.run_mode {
        RunMode::Simple => "",
        RunMode::Tui => " (TUI mode)",
        RunMode::Background => " (background)",
    };
    let instruction = if dialog.spell.confirm {
        "Type 'yes' or press Enter to confirm, Esc to cancel"
    } else {
        "Press Enter to confirm, Esc to cancel"
    };
    let line5 = Line::from(vec![
        Span::styled(instruction, Style::new().fg(theme.muted)),
        Span::styled(run_mode_hint, Style::new().fg(theme.accent)),
    ]);
    let para5 = Paragraph::new(line5);
    f.render_widget(para5, chunks[5]);
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
