use crate::state::State;
use crate::ui::{
    add_spell, add_spellbook_form, confirm, help, jobs, quick_add_spell, search_overlay,
    streaming_modal, Mode, Overlay, UiState,
};
use crate::ui::search_overlay::format_full_spell_details;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Main render dispatcher - renders current mode, then overlays on top
pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    // Collect overlays first to avoid borrow issues
    let overlays: Vec<Overlay> = ui.overlays.clone();

    if ui.jobs_sidebar_open {
        render_with_sidebar(frame, state, ui, &overlays);
    } else {
        render_mode(ui.mode.clone(), frame, state, ui, frame.area());
        // Render overlays on top (modal windows)
        for overlay in &overlays {
            render_overlay(*overlay, frame, state, ui);
        }
    }

    // Render loading indicator on top if active
    if ui.is_loading() {
        render_loading_indicator(frame, ui, &state.theme);
    }
}

/// Render a loading indicator overlay
fn render_loading_indicator(frame: &mut Frame, ui: &UiState, theme: &crate::models::RatatuiColors) {
    use ratatui::layout::{Alignment, Rect};
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let area = frame.area();

    // Create a small popup in the bottom-right corner
    let message = ui.loading_message.as_deref().unwrap_or("Loading...");
    let spinner = ui.spinner_char();
    let text = format!("{} {}", spinner, message);

    let width = text.len() as u16 + 4; // padding
    let height = 3;
    let x = area.width.saturating_sub(width + 2);
    let y = area.height.saturating_sub(height + 1);

    let popup_area = Rect::new(x, y, width, height);

    // Semi-transparent background effect by rendering a block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.accent))
        .style(Style::new().bg(theme.bg));

    let line = Line::from(vec![Span::styled(
        format!(" {} ", text),
        Style::new()
            .fg(theme.fg)
            .bg(theme.bg)
            .add_modifier(Modifier::BOLD),
    )]);

    let paragraph = Paragraph::new(line)
        .alignment(Alignment::Center)
        .block(block);

    frame.render_widget(paragraph, popup_area);
}

/// Render the current mode
fn render_mode(
    mode: Mode,
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    match mode {
        Mode::BrowseSpellbooks(_) | Mode::BrowseSpells(_) => {
            // Both browse modes use the search overlay for consistent UI with search bar
            search_overlay::render_in_area(frame, state, ui, area);
        }
        Mode::AddSpell(_) | Mode::EditSpell(_) => {
            add_spell::render_in_area(frame, state, ui, area);
        }
        Mode::AddSpellbook(_) => {
            add_spellbook_form::render_in_area(frame, state, ui, area);
        }
    }
}

/// Render an overlay on top of the current mode
fn render_overlay(overlay: Overlay, frame: &mut Frame, state: &State, ui: &mut UiState) {
    match overlay {
        Overlay::OutputModal => {
            // Use new streaming modal if streaming is active, otherwise fall back to legacy
            if ui.streaming_modal.streaming.is_some() || ui.streaming_modal.output.is_streaming {
                streaming_modal::render_streaming_modal(frame, ui, &state.theme);
            } else if ui.output_popup.is_some() {
                render_output_popup(frame, state, ui);
            }
        }
        Overlay::ConfirmDialog => {
            if ui.confirm_dialog.is_some() {
                render_confirm_popup(frame, state, ui);
            }
        }
        Overlay::CommandPalette => {
            // Command palette is rendered as part of search overlay
        }
        Overlay::Help => {
            render_help_overlay(frame, state);
        }
        Overlay::InputPopup => {
            if ui.input_popup.is_some() {
                render_input_popup_overlay(frame, state, ui);
            }
        }
        Overlay::SpellDetails => {
            if ui.spell_details_spell_id.is_some() {
                render_spell_details_overlay(frame, state, ui);
            }
        }
        Overlay::QuickAddSpell => {
            quick_add_spell::render(frame, state, ui);
        }
    }
}

fn render_spell_details_overlay(frame: &mut Frame, state: &State, ui: &UiState) {
    let theme = &state.theme;
    let area = frame.area();

    // Dim the background
    let overlay_rect = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    let overlay = Paragraph::new("").style(Style::new().bg(ratatui::style::Color::Indexed(0)));
    frame.render_widget(overlay, overlay_rect);

    // Calculate popup size
    let popup_width = 70.min(area.width - 4);
    let popup_height = 15.min(area.height - 4);

    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Get spell details
    if let Some(spell_id) = &ui.spell_details_spell_id {
        if let Some(spell) = state.codex.spells.iter().find(|s| s.id == *spell_id) {
            let details = format_full_spell_details(spell, theme);
            
            let details_block = Paragraph::new(details)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Spell Details ")
                        .border_style(Style::new().fg(theme.accent))
                        .title_style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)),
                )
                .style(Style::new().bg(theme.bg).fg(theme.fg))
                .wrap(Wrap { trim: true });

            frame.render_widget(details_block, popup_area);
        }
    }
}

fn render_with_sidebar(frame: &mut Frame, state: &State, ui: &mut UiState, overlays: &[Overlay]) {
    let area = frame.area();
    let theme = &state.theme;
    let sidebar_width: u16 = 40;
    let sidebar_min_width: u16 = 30;

    let actual_sidebar_width = sidebar_width.min(area.width / 3).max(sidebar_min_width);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(actual_sidebar_width)])
        .split(area);

    // Render the current mode in the main area
    render_mode(ui.mode.clone(), frame, state, ui, chunks[0]);

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

    // Render overlays on top of everything (including sidebar)
    for overlay in overlays {
        render_overlay(*overlay, frame, state, ui);
    }
}

fn render_output_popup(frame: &mut Frame, state: &State, ui: &UiState) {
    let theme = &state.theme;
    let area = frame.area();
    render_output_popup_in_area(frame, state, ui, area);
}

fn render_output_popup_in_area(
    frame: &mut Frame,
    state: &State,
    ui: &UiState,
    area: ratatui::layout::Rect,
) {
    let theme = &state.theme;

    // Full-screen overlay to hide content behind popup
    let overlay_rect = area;
    let overlay = Paragraph::new("").style(Style::new().bg(theme.bg));
    frame.render_widget(overlay, overlay_rect);

    // Calculate popup dimensions (relative to available area)
    let max_width = area.width.saturating_sub(4).max(20);
    let max_height = area.height.saturating_sub(4).max(6);
    let popup_width = max_width.min(80);
    let popup_height = max_height.min(area.height.saturating_sub(2)).max(6);

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

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

    // Draw border
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

    let popup_bg = theme.bg;

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
        let cmd_para = Paragraph::new(command_line).style(Style::new().bg(popup_bg).fg(theme.fg));
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
            .style(Style::new().bg(popup_bg).fg(theme.fg))
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
            .style(Style::new().bg(popup_bg).fg(theme.fg))
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

    if let Some(ref dialog) = ui.confirm_dialog {
        confirm::render_confirm_dialog(frame, popup_area, theme, dialog);
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

fn render_help_overlay(frame: &mut Frame, state: &State) {
    let theme = &state.theme;
    let area = frame.area();

    // Dim the background
    let overlay_rect = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    let overlay = Paragraph::new("").style(Style::new().bg(ratatui::style::Color::Indexed(0)));
    frame.render_widget(overlay, overlay_rect);

    // Calculate popup size
    let popup_width = 60.min(area.width - 4);
    let popup_height = 20.min(area.height - 4);

    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear popup area so it appears over background
    frame.render_widget(Clear, popup_area);

    help::render_help(frame, popup_area, theme);
}
