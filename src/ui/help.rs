use crate::models::RatatuiColors;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_help(frame: &mut Frame, area: Rect, theme: &RatatuiColors) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent))
        .title_style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(inner);

    let key_style = Style::new().fg(theme.accent).add_modifier(Modifier::BOLD);
    let title_style = Style::new().fg(theme.fg).add_modifier(Modifier::BOLD);
    let desc_style = Style::new().fg(theme.muted);

    // Navigation
    let nav_title = Line::from(vec![Span::styled("Navigation", title_style)]);
    frame.render_widget(Paragraph::new(nav_title), chunks[0]);

    let nav_keys = Line::from(vec![
        Span::styled("arrows/hjkl", key_style),
        Span::raw("  "),
        Span::styled("navigate", desc_style),
        Span::raw("    "),
        Span::styled("h", key_style),
        Span::raw("  "),
        Span::styled("back", desc_style),
        Span::raw("    "),
        Span::styled("Enter", key_style),
        Span::raw("  "),
        Span::styled("open/execute", desc_style),
        Span::raw("    "),
        Span::styled("Esc", key_style),
        Span::raw("  "),
        Span::styled("close", desc_style),
    ]);
    frame.render_widget(Paragraph::new(nav_keys), chunks[1]);

    // Actions
    let action_title = Line::from(vec![Span::styled("Actions", title_style)]);
    frame.render_widget(Paragraph::new(action_title), chunks[2]);

    let action_keys = Line::from(vec![
        Span::styled("f", key_style),
        Span::raw("  "),
        Span::styled("toggle favorite", desc_style),
        Span::raw("    "),
        Span::styled("e", key_style),
        Span::raw("  "),
        Span::styled("edit spell", desc_style),
        Span::raw("    "),
        Span::styled("d", key_style),
        Span::raw("  "),
        Span::styled("delete spell", desc_style),
        Span::raw("    "),
        Span::styled("Ctrl+v", key_style),
        Span::raw("  "),
        Span::styled("view details", desc_style),
    ]);
    frame.render_widget(Paragraph::new(action_keys), chunks[3]);

    // Execution
    let exec_title = Line::from(vec![Span::styled("Execution", title_style)]);
    frame.render_widget(Paragraph::new(exec_title), chunks[4]);

    let exec_keys = Line::from(vec![
        Span::styled("s", key_style),
        Span::raw("  "),
        Span::styled("simple (exit)", desc_style),
        Span::raw("    "),
        Span::styled("Ctrl+r", key_style),
        Span::raw("  "),
        Span::styled("TUI mode", desc_style),
        Span::raw("    "),
        Span::styled("Ctrl+b", key_style),
        Span::raw("  "),
        Span::styled("background", desc_style),
    ]);
    frame.render_widget(Paragraph::new(exec_keys), chunks[5]);

    // Commands
    let cmd_title = Line::from(vec![Span::styled("Commands", title_style)]);
    frame.render_widget(Paragraph::new(cmd_title), chunks[6]);

    let cmd_keys = Line::from(vec![
        Span::styled(":", key_style),
        Span::raw("  "),
        Span::styled("command palette", desc_style),
        Span::raw("    "),
        Span::styled("/", key_style),
        Span::raw("  "),
        Span::styled("search", desc_style),
    ]);
    frame.render_widget(Paragraph::new(cmd_keys), chunks[7]);

    // Jobs
    let jobs_title = Line::from(vec![Span::styled("Jobs Sidebar", title_style)]);
    frame.render_widget(Paragraph::new(jobs_title), chunks[8]);

    let jobs_keys = Line::from(vec![
        Span::styled(":jobs", key_style),
        Span::raw("  "),
        Span::styled("toggle sidebar", desc_style),
        Span::raw("    "),
        Span::styled("Tab", key_style),
        Span::raw("  "),
        Span::styled("cycle focus", desc_style),
        Span::raw("    "),
        Span::styled("k", key_style),
        Span::raw("  "),
        Span::styled("kill job", desc_style),
        Span::raw("  "),
        Span::styled("c", key_style),
        Span::raw("  "),
        Span::styled("cancel", desc_style),
        Span::raw("  "),
        Span::styled("d", key_style),
        Span::raw("  "),
        Span::styled("dismiss", desc_style),
        Span::raw("  "),
        Span::styled("v", key_style),
        Span::raw("  "),
        Span::styled("view", desc_style),
    ]);
    frame.render_widget(Paragraph::new(jobs_keys), chunks[8]);

    // Footer
    let footer = Line::from(vec![Span::styled(
        "Press Esc to close",
        Style::new().fg(theme.muted),
    )]);
    let footer_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
    frame.render_widget(Paragraph::new(footer), footer_area);
}
