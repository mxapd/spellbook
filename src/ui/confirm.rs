use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::models::{RunMode, Spell};

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteSpell(Spell),
    DeleteSpellbook(String),
    ExecuteSpell(Spell),
}

#[derive(Debug, Clone)]
pub struct ConfirmDialogState {
    pub action: ConfirmAction,
    pub typed_confirmation: String,
}

impl Default for ConfirmDialogState {
    fn default() -> Self {
        Self {
            action: ConfirmAction::DeleteSpell(Spell::default()),
            typed_confirmation: String::new(),
        }
    }
}

impl ConfirmDialogState {
    pub fn delete_spell(spell: Spell) -> Self {
        Self {
            action: ConfirmAction::DeleteSpell(spell),
            typed_confirmation: String::new(),
        }
    }

    pub fn delete_spellbook(name: String) -> Self {
        Self {
            action: ConfirmAction::DeleteSpellbook(name),
            typed_confirmation: String::new(),
        }
    }

    pub fn execute_spell(spell: Spell) -> Self {
        Self {
            action: ConfirmAction::ExecuteSpell(spell),
            typed_confirmation: String::new(),
        }
    }

    pub fn requires_typed_confirmation(&self) -> bool {
        match &self.action {
            ConfirmAction::DeleteSpell(s) => s.confirm,
            ConfirmAction::DeleteSpellbook(_) => true,
            ConfirmAction::ExecuteSpell(s) => s.confirm,
        }
    }

    pub fn title(&self) -> &'static str {
        match &self.action {
            ConfirmAction::DeleteSpell(_) => " Delete Spell ",
            ConfirmAction::DeleteSpellbook(_) => " Delete Spellbook ",
            ConfirmAction::ExecuteSpell(_) => " Confirm ",
        }
    }

    pub fn confirmation_word(&self) -> &'static str {
        match &self.action {
            ConfirmAction::DeleteSpell(_) => "DELETE",
            ConfirmAction::DeleteSpellbook(_) => "DELETE",
            ConfirmAction::ExecuteSpell(_) => "yes",
        }
    }
}

pub fn render_confirm_dialog(
    f: &mut Frame,
    area: Rect,
    theme: &crate::models::RatatuiColors,
    dialog: &ConfirmDialogState,
) {
    let block = Block::default()
        .title(dialog.title())
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent));

    f.render_widget(&block, area);

    let inner = block.inner(area);
    let inner_width = inner.width as usize;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

    match &dialog.action {
        ConfirmAction::DeleteSpell(spell) => {
            render_delete_spell(f, chunks, theme, spell, inner_width);
        }
        ConfirmAction::DeleteSpellbook(name) => {
            render_delete_spellbook(f, chunks, theme, name);
        }
        ConfirmAction::ExecuteSpell(spell) => {
            render_execute_spell(f, chunks, theme, spell, inner_width, dialog);
        }
    }
}

fn render_delete_spell(
    f: &mut Frame,
    chunks: std::rc::Rc<[Rect]>,
    theme: &crate::models::RatatuiColors,
    spell: &Spell,
    inner_width: usize,
) {
    let line1 = Line::from(vec![
        Span::styled("Delete spell '", Style::new().fg(theme.fg)),
        Span::styled(&spell.name, Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("'?", Style::new().fg(theme.fg)),
    ]);
    let para1 = Paragraph::new(line1);
    f.render_widget(para1, chunks[0]);

    let line2 = Line::from(vec![
        Span::styled("Command: ", Style::new().fg(theme.muted)),
        Span::styled(truncate_string(&spell.incantation, inner_width.saturating_sub(12)), Style::new().fg(theme.fg)),
    ]);
    let para2 = Paragraph::new(line2);
    f.render_widget(para2, chunks[1]);

    let instruction = "Type 'DELETE' to confirm, Esc to cancel";
    let line3 = Line::from(vec![Span::styled(instruction, Style::new().fg(theme.muted))]);
    let para3 = Paragraph::new(line3);
    f.render_widget(para3, chunks[3]);
}

fn render_delete_spellbook(
    f: &mut Frame,
    chunks: std::rc::Rc<[Rect]>,
    theme: &crate::models::RatatuiColors,
    name: &str,
) {
    let line1 = Line::from(vec![
        Span::styled("Delete spellbook '", Style::new().fg(theme.fg)),
        Span::styled(name, Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("'?", Style::new().fg(theme.fg)),
    ]);
    let para1 = Paragraph::new(line1);
    f.render_widget(para1, chunks[0]);

    let line2 = Line::from(vec![
        Span::styled("This will also delete all spells in this spellbook.", Style::new().fg(theme.muted)),
    ]);
    let para2 = Paragraph::new(line2);
    f.render_widget(para2, chunks[1]);

    let instruction = "Type 'DELETE' to confirm, Esc to cancel";
    let line3 = Line::from(vec![Span::styled(instruction, Style::new().fg(theme.muted))]);
    let para3 = Paragraph::new(line3);
    f.render_widget(para3, chunks[3]);
}

fn render_execute_spell(
    f: &mut Frame,
    chunks: std::rc::Rc<[Rect]>,
    theme: &crate::models::RatatuiColors,
    spell: &Spell,
    inner_width: usize,
    dialog: &ConfirmDialogState,
) {
    let line1 = if spell.confirm {
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
            &spell.name,
            Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]);
    let para2 = Paragraph::new(line2);
    f.render_widget(para2, chunks[1]);

    let line3 = Line::from(vec![Span::styled(
        "Command: ",
        Style::new().fg(theme.muted),
    )]);
    let cmd_display = truncate_string(&spell.incantation, inner_width.saturating_sub(10));
    let cmd_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(&cmd_display, Style::new().fg(theme.accent)),
    ]);
    let para3 = Paragraph::new(vec![line3, cmd_line]);
    f.render_widget(para3, chunks[2]);

    let run_mode_hint = match spell.run_mode {
        RunMode::Simple => "",
        RunMode::Tui => " (TUI mode)",
        RunMode::Background => " (background)",
    };
    let instruction = if spell.confirm {
        "Type 'yes' or press Enter to confirm, Esc to cancel"
    } else {
        "Press Enter to confirm, Esc to cancel"
    };
    let line5 = Line::from(vec![
        Span::styled(instruction, Style::new().fg(theme.muted)),
        Span::styled(run_mode_hint, Style::new().fg(theme.accent)),
    ]);
    let para5 = Paragraph::new(line5);
    f.render_widget(para5, chunks[3]);
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
