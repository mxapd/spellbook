use crate::state::State;
use crate::ui::{add_spell, input, jobs, search_overlay, spell_list, Screen, UiState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    if ui.jobs_sidebar_open {
        render_with_sidebar(frame, state, ui);
        return;
    }

    match &ui.screen {
        Screen::SpellList => {
            spell_list::render(frame, state, ui);
        }
        Screen::SearchOverlay => {
            search_overlay::render(frame, state, ui);
        }
        Screen::AddSpell => {
            add_spell::render(frame, state, ui);
        }
        Screen::OutputPopup => {
            search_overlay::render(frame, state, ui);
            render_output_popup(frame, state, ui);
        }
        Screen::JobsPanel => {
            search_overlay::render(frame, state, ui);
            render_jobs_popup(frame, state, ui);
        }
        Screen::ConfirmDialog => {
            search_overlay::render(frame, state, ui);
            if let Some(_) = &ui.confirm_dialog {
                render_confirm_popup(frame, state, ui);
            }
        }
        Screen::InputPopup => {
            search_overlay::render(frame, state, ui);
            if let Some(_) = &ui.input_popup {
                render_input_popup_overlay(frame, state, ui);
            }
        }
    }
}

fn render_with_sidebar(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let area = frame.area();
    let theme = &state.theme;
    let sidebar_width: u16 = 40;
    let sidebar_min_width: u16 = 30;

    let actual_sidebar_width = sidebar_width.min(area.width / 3).max(sidebar_min_width);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(actual_sidebar_width)])
        .split(area);

    match &ui.screen {
        Screen::SpellList => {
            spell_list::render_in_area(frame, state, ui, chunks[0]);
        }
        Screen::SearchOverlay => {
            search_overlay::render_in_area(frame, state, ui, chunks[0]);
        }
        Screen::AddSpell => {
            add_spell::render_in_area(frame, state, ui, chunks[0]);
        }
        Screen::OutputPopup => {
            search_overlay::render_in_area(frame, state, ui, chunks[0]);
            render_output_popup(frame, state, ui);
        }
        Screen::ConfirmDialog => {
            search_overlay::render_in_area(frame, state, ui, chunks[0]);
            if ui.confirm_dialog.is_some() {
                render_confirm_popup(frame, state, ui);
            }
        }
        Screen::InputPopup => {
            search_overlay::render_in_area(frame, state, ui, chunks[0]);
            if ui.input_popup.is_some() {
                render_input_popup_overlay(frame, state, ui);
            }
        }
        _ => {
            search_overlay::render_in_area(frame, state, ui, chunks[0]);
        }
    }

    let border_style = if ui.focus == crate::models::FocusTarget::JobsSidebar {
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme.muted)
    };

    let jobs_block = Block::default()
        .title(" Jobs ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .title_style(if ui.focus == crate::models::FocusTarget::JobsSidebar {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.muted)
        });

    frame.render_widget(&jobs_block, chunks[1]);
    let inner = jobs_block.inner(chunks[1]);
    jobs::render_jobs_panel(frame, state, ui, inner);

    let focus_hint = if ui.focus == crate::models::FocusTarget::JobsSidebar {
        "[Tab] Main  [Esc] Close"
    } else {
        "[Tab] Jobs"
    };
    let focus_para = Paragraph::new(focus_hint)
        .style(Style::new().fg(theme.muted))
        .alignment(Alignment::Center);
    let focus_area = ratatui::layout::Rect {
        x: chunks[1].x,
        y: chunks[1].y + chunks[1].height - 1,
        width: chunks[1].width,
        height: 1,
    };
    frame.render_widget(focus_para, focus_area);
}

fn render_output_popup(frame: &mut Frame, state: &State, ui: &UiState) {
    let theme = &state.theme;
    let area = frame.area();

    // 1. Dim the entire background FIRST (before popup)
    let overlay_rect = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    let overlay = Paragraph::new("").style(Style::new().bg(ratatui::style::Color::Indexed(0)));
    frame.render_widget(overlay, overlay_rect);

    // 2. Calculate popup dimensions
    let popup_width = (area.width * 3).min(80).max(40);
    let popup_height = (area.height * 3 / 5).max(10).min(area.height - 4);

    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent))
        .title(" Output ")
        .title_style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD));

    // 3. Render popup background (clears the dimmed area)
    frame.render_widget(&block, popup_area);

    let inner = block.inner(popup_area);
    let inner_width = inner.width as usize;
    let inner_height = inner.height as usize;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Command line + spacing
            Constraint::Min(1),    // Output area
            Constraint::Length(1), // Footer
        ])
        .split(inner);

    // 4. Render popup content
    if let Some(ref result) = ui.output_popup {
        let exit_indicator = match result.exit_code {
            Some(0) => "✓",
            Some(_code) => "✗",
            None => "?",
        };
        let command_line = Line::from(vec![
            Span::raw(format!("$ {} ", exit_indicator)),
            Span::styled(
                truncate_string(&result.command, inner_width.saturating_sub(5)),
                Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]);
        let cmd_para = Paragraph::new(command_line).style(Style::new().bg(theme.bg).fg(theme.fg));
        frame.render_widget(cmd_para, layout[0]);

        // Output area
        let mut output_lines = Vec::new();

        // Add stderr first (in red/muted) if present
        if !result.stderr.is_empty() {
            for line in result
                .display_stderr()
                .lines()
                .take(inner_height.saturating_sub(3))
            {
                output_lines.push(Line::from(vec![Span::styled(
                    truncate_string(line, inner_width),
                    Style::new().fg(ratatui::style::Color::Indexed(196)), // Red
                )]));
            }
        }

        // Add stdout (in fg color) if present
        if !result.stdout.is_empty() {
            for line in result
                .display_stdout()
                .lines()
                .take(inner_height.saturating_sub(3))
            {
                output_lines.push(Line::from(vec![Span::styled(
                    truncate_string(line, inner_width),
                    Style::new().fg(theme.fg),
                )]));
            }
        }

        // If no output, show a message
        if output_lines.is_empty() {
            output_lines.push(Line::from(vec![Span::styled(
                "(no output)",
                Style::new().fg(theme.muted),
            )]));
        }

        let output_text = Text::from(output_lines);
        let output_para = Paragraph::new(output_text)
            .style(Style::new().bg(theme.bg).fg(theme.fg))
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        frame.render_widget(output_para, layout[1]);

        // Footer
        let exit_str = match result.exit_code {
            Some(code) => format!("exit: {}", code),
            None => "exit: ?".to_string(),
        };
        let footer = Line::from(vec![
            Span::styled("any key: close  ", Style::new().fg(theme.muted)),
            Span::styled("s: save  ", Style::new().fg(theme.accent)),
            Span::styled(exit_str, Style::new().fg(theme.muted)),
        ]);
        let footer_para = Paragraph::new(footer)
            .style(Style::new().bg(theme.bg).fg(theme.fg))
            .alignment(Alignment::Center);
        frame.render_widget(footer_para, layout[2]);
    }
}

fn truncate_string(s: &str, max_width: usize) -> String {
    if max_width < 4 {
        return String::new();
    }

    let visual_width = string_visual_width(s);
    if visual_width <= max_width {
        return s.to_string();
    }

    // Truncate and add "..."
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

fn string_visual_width(s: &str) -> usize {
    s.chars().map(|c| if c == '\t' { 8 } else { 1 }).sum()
}

fn render_jobs_popup(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let theme = &state.theme;
    let area = frame.area();

    let overlay_rect = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    let overlay = Paragraph::new("").style(Style::new().bg(ratatui::style::Color::Indexed(0)));
    frame.render_widget(overlay, overlay_rect);

    let popup_width = 60.min(area.width - 4);
    let popup_height = (area.height * 3 / 5).max(8).min(area.height - 4);

    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    jobs::render_jobs_panel(frame, state, ui, popup_area);
}

fn render_confirm_popup(frame: &mut Frame, state: &State, ui: &UiState) {
    let theme = &state.theme;
    let area = frame.area();

    let overlay_rect = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    let overlay = Paragraph::new("").style(Style::new().bg(ratatui::style::Color::Indexed(0)));
    frame.render_widget(overlay, overlay_rect);

    let popup_width = 70.min(area.width - 4);
    let popup_height = 12.min(area.height - 4);

    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent))
        .title(" Confirm ")
        .title_style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD));

    frame.render_widget(&block, popup_area);

    let inner = block.inner(popup_area);
    let inner_width = inner.width as usize;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    if let Some(ref dialog) = ui.confirm_dialog {
        let line1 = Line::from(vec![
            Span::raw("Confirm "),
            Span::styled("execution", Style::new().fg(theme.accent)),
            Span::raw("?"),
        ]);
        let para1 = Paragraph::new(line1).style(Style::new().fg(theme.fg));
        frame.render_widget(para1, layout[0]);

        let line2 = Line::from(vec![
            Span::styled("Spell: ", Style::new().fg(theme.muted)),
            Span::styled(
                &dialog.spell.name,
                Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ]);
        let para2 = Paragraph::new(line2);
        frame.render_widget(para2, layout[1]);

        let line3 = Line::from(vec![Span::styled(
            "Command: ",
            Style::new().fg(theme.muted),
        )]);
        let cmd_display =
            truncate_string(&dialog.spell.incantation, inner_width.saturating_sub(10));
        let cmd_line = Line::from(vec![
            Span::raw("  "),
            Span::styled(&cmd_display, Style::new().fg(theme.accent)),
        ]);
        let para3 = Paragraph::new(vec![line3, cmd_line]);
        frame.render_widget(para3, layout[2]);

        let instruction = "Press Enter to confirm, Esc to cancel";
        let line5 = Line::from(vec![Span::styled(
            instruction,
            Style::new().fg(theme.muted),
        )]);
        let para5 = Paragraph::new(line5);
        frame.render_widget(para5, layout[5]);
    }
}

fn render_input_popup_overlay(frame: &mut Frame, state: &State, ui: &UiState) {
    use ratatui::widgets::Borders;

    let area = frame.area();
    let theme = &state.theme;

    let overlay_rect = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    let overlay = Paragraph::new("").style(Style::new().bg(ratatui::style::Color::Indexed(0)));
    frame.render_widget(overlay, overlay_rect);

    let input_state = match ui.input_popup.as_ref() {
        Some(s) => s,
        None => return,
    };

    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - 10) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: 10,
    };

    let block = Block::default()
        .title(" Arguments ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent));

    frame.render_widget(&block, popup_area);

    let inner = block.inner(popup_area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("Command: ", Style::new().fg(theme.muted)),
        Span::styled(&input_state.command, Style::new().fg(theme.accent)),
    ]));
    lines.push(Line::from(""));

    for p in &input_state.placeholders {
        let val = if p.value.is_empty() { "_" } else { &p.value };
        lines.push(Line::from(vec![
            Span::styled(&p.placeholder.display_name, Style::new().fg(theme.fg)),
            Span::raw(": "),
            Span::styled(val, Style::new().fg(theme.accent)),
        ]));
    }

    lines.push(Line::from(""));

    let help = "[Enter] Execute  [Esc] Cancel";
    lines.push(Line::from(vec![Span::styled(
        help,
        Style::new().fg(theme.muted),
    )]));

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}
