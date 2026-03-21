use crate::models::{SpineStyle, ViewMode};
use crate::state::State;
use crate::ui::{SearchMode, UiState};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

fn build_spine_decorations(
    style: SpineStyle,
    spine_width: usize,
    name: &str,
    decor_style: Style,
    name_style: Style,
    spine_height: usize,
) -> Vec<Line<'static>> {
    let pad = |s: &str| {
        let padding = (spine_width.saturating_sub(s.len())) / 2;
        format!("{}{}", " ".repeat(padding), s)
    };

    // Build decorations above and below based on style elaborateness
    let (above, below): (Vec<&str>, Vec<&str>) = match style {
        // Most elaborate - full decorations
        SpineStyle::Alchemy => (
            vec!["☉   ☿", "～◇～", "～", "～", "～", "～"],
            vec!["～", "～", "～", "～", "～◇～", "☉   ☿"],
        ),
        SpineStyle::Celestial => (
            vec!["☽   ☾", "∴   ∴", "～", "～", "～", "～"],
            vec!["～", "～", "～", "～", "∴   ∴", "☽   ☾"],
        ),
        // Medium - moderate decorations
        SpineStyle::StarsAndDiamonds => (
            vec!["✦   ✦", "～", "～", "～"],
            vec!["～", "～", "～", "✦   ✦"],
        ),
        SpineStyle::Geometric => (
            vec!["∵   ∴", "⋯ ⋯ ⋰", "～", "～", "～"],
            vec!["～", "～", "～", "⋯ ⋯ ⋰", "∵   ∴"],
        ),
        // Less elaborate
        SpineStyle::DotsAndTherefore => (
            vec!["∴   ∴", "· · ·", "· · ·"],
            vec!["· · ·", "· · ·", "∴   ∴"],
        ),
        // Minimal
        SpineStyle::Minimal => (vec!["～", "～"], vec!["～", "～"]),
    };

    // Spacing rows around the name
    let space_above = match style {
        SpineStyle::Alchemy | SpineStyle::Celestial => 2,
        _ => 1,
    };
    let space_below = space_above;
    let decorations_above = above.len();
    let decorations_below = below.len();

    // Wrap name to fit spine width
    let name_lines = wrap_text_for_spine(name, spine_width);
    let name_count = name_lines.len();

    // Calculate where name starts to be vertically centered in spine_height
    let name_start = (spine_height.saturating_sub(name_count)) / 2;

    // Calculate positions relative to centered name
    let decor_above_start = name_start
        .saturating_sub(space_above)
        .saturating_sub(decorations_above);
    let decor_below_start = name_start + name_count + space_below;

    // Build spine row by row
    let mut spine_text: Vec<Line<'static>> = Vec::with_capacity(spine_height);

    for row in 0..spine_height {
        if row >= decor_above_start && row < name_start {
            // Decoration row above name (relative to where name starts)
            let decor_idx = row - decor_above_start;
            if (decor_idx as usize) < decorations_above {
                spine_text.push(Line::from(vec![Span::styled(
                    pad(above[decor_idx as usize]),
                    decor_style,
                )]));
            } else {
                spine_text.push(Line::from(""));
            }
        } else if row >= name_start && row < name_start + name_count {
            // Name row
            let name_idx = row - name_start;
            spine_text.push(Line::from(vec![Span::styled(
                pad(&name_lines[name_idx as usize]),
                name_style,
            )]));
        } else if row >= decor_below_start && row < decor_below_start + decorations_below {
            // Decoration row below name
            let decor_idx = row - decor_below_start;
            spine_text.push(Line::from(vec![Span::styled(
                pad(below[decor_idx as usize]),
                decor_style,
            )]));
        } else {
            // Padding row
            spine_text.push(Line::from(""));
        }
    }

    spine_text
}

pub fn render(frame: &mut Frame, state: &State, ui: &mut UiState) {
    let area = frame.area();
    let theme = &state.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    let input_text = format!("/{}", ui.search_query());
    let input_block = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(Style::new().fg(theme.accent))
                .title_style(Style::new().fg(theme.accent)),
        )
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_widget(input_block, chunks[0]);

    // Render main content based on search mode
    match ui.search_mode {
        SearchMode::BrowseSpellbooks | SearchMode::BrowseSpells => {
            if ui.search_mode == SearchMode::BrowseSpells {
                render_spellbook_spells(frame, state, ui, chunks[1]);
            } else if ui.search_query().is_empty() && ui.showing_spellbooks() {
                render_spellbook_browser(frame, state, ui, chunks[1]);
            } else if ui.search_query().starts_with(':') {
                // Command mode - show filtered commands
                render_command_list(frame, state, ui, chunks[1]);
            } else if ui.filtered_indices().is_empty() {
                let message = if ui.search_query().is_empty() {
                    "Type to search all spells..."
                } else {
                    "No spells found"
                };
                let empty = Paragraph::new(message)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::new().fg(theme.border)),
                    )
                    .style(Style::new().fg(theme.muted).bg(theme.bg));
                frame.render_widget(empty, chunks[1]);
            } else {
                render_search_results(frame, state, ui, chunks[1]);
            }
        }
        SearchMode::AddSpell => {
            render_add_spell_form(frame, state, ui, chunks[1]);
        }
        SearchMode::AddSpellbook => {
            render_add_spellbook_form(frame, state, ui, chunks[1]);
        }
    }

    let details = if ui.search_query().is_empty() && ui.showing_spellbooks() {
        render_spellbook_details(state, ui)
    } else if !ui.filtered_indices().is_empty() {
        render_spell_details(state, ui)
    } else {
        vec![Line::from("")]
    };

    let details_block = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details ")
                .border_style(Style::new().fg(theme.border))
                .title_style(Style::new().fg(theme.accent)),
        )
        .style(Style::new().bg(theme.bg).fg(theme.fg))
        .wrap(Wrap { trim: true });

    frame.render_widget(details_block, chunks[2]);

    let hint = if let Some(ref msg) = ui.copy_feedback {
        let single_line = msg.lines().next().unwrap_or(msg).to_string();
        Paragraph::new(single_line)
            .style(Style::new().fg(ratatui::style::Color::Green).bg(theme.bg))
            .alignment(ratatui::layout::Alignment::Center)
    } else {
        let hint_text = match ui.search_mode {
            SearchMode::BrowseSpellbooks
                if ui.search_query().is_empty() && ui.showing_spellbooks() =>
            {
                "←→↑↓ navigate  enter open  : cmd".to_string()
            }
            SearchMode::BrowseSpells => "↑↓ navigate  enter copy  ← back".to_string(),
            _ => {
                if ui.search_query().starts_with(':') {
                    "↑↓ navigate  enter run  esc cancel".to_string()
                } else if ui.filtered_indices().is_empty() && ui.search_query().is_empty() {
                    "type to search".to_string()
                } else {
                    "↑↓ navigate  enter copy  esc clear".to_string()
                }
            }
        };
        Paragraph::new(hint_text).style(Style::new().fg(theme.muted).bg(theme.bg))
    };
    frame.render_widget(hint, chunks[3]);
}

fn render_spellbook_browser(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    let theme = &state.theme;
    let spellbooks = &state.codex.spellbooks;

    if spellbooks.is_empty() {
        let empty = Paragraph::new("No spellbooks yet\n\nRun: spellbook --add")
            .style(Style::new().fg(theme.muted).bg(theme.bg))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(theme.border)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let card_gap = 2;
    let card_width = 14;
    let card_height = 10;
    let min_spine_width = 12;

    let cards_per_row = ((area.width as usize) / (card_width + card_gap)).max(1);
    let spines_per_row = ((area.width as usize) / (min_spine_width + 1)).max(1);

    // Store spines_per_row for use in event handling
    ui.set_search_spines_per_row(spines_per_row);

    // Handle resize - clamp scroll and selection when dimensions change
    let resized = area.width != ui.search_last_width() || area.height != ui.search_last_height();
    if resized {
        ui.set_search_last_width(area.width);
        ui.set_search_last_height(area.height);

        // Clamp scroll to valid range
        let max_scroll = spellbooks.len().saturating_sub(spines_per_row);
        ui.set_search_spellbook_scroll(ui.search_spellbook_scroll().min(max_scroll));

        // Clamp selection to visible range
        if let Some(idx) = ui.search_spellbook_index() {
            if idx >= spellbooks.len() {
                ui.set_search_spellbook_index(Some(spellbooks.len().saturating_sub(1)));
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.border))
        .title(" Spellbooks ")
        .title_style(Style::new().fg(theme.accent));

    let inner = block.inner(area);

    let books_that_fit = (inner.width as usize) / (card_width + card_gap);
    let all_books_fit = spellbooks.len() <= books_that_fit;

    frame.render_widget(block, area);

    let view_mode = state.user_settings.view_mode;
    let show_as = match view_mode {
        ViewMode::Cards => ShowAs::Cards,
        ViewMode::Spines => ShowAs::Spines,
        ViewMode::List => ShowAs::List,
    };

    // Store unified items_per_row for navigation
    match show_as {
        ShowAs::List => {
            ui.set_search_items_per_row(1);
            render_spellbook_list(frame, state, ui, inner);
        }
        ShowAs::Cards => {
            ui.set_search_items_per_row(cards_per_row);
            render_spellbook_cards(
                frame,
                state,
                ui,
                inner,
                card_width,
                card_height,
                card_gap,
                cards_per_row,
            );
        }
        ShowAs::Spines => {
            ui.set_search_items_per_row(spines_per_row);
            render_spellbook_spines(frame, state, ui, inner, min_spine_width, spines_per_row);
        }
    }
}

enum ShowAs {
    List,
    Cards,
    Spines,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CardDirection {
    Up,
    Down,
    Left,
    Right,
}

pub fn find_nearest_card(
    current: usize,
    direction: CardDirection,
    spellbook_count: usize,
    cards_per_row: usize,
    card_width: usize,
    card_height: usize,
    card_gap: usize,
    grid_offset: u16,
) -> usize {
    if spellbook_count == 0 {
        return 0;
    }

    let current_col = current % cards_per_row;
    let current_row = current / cards_per_row;

    let current_x = (grid_offset as i32) + (current_col as i32) * ((card_width + card_gap) as i32);
    let current_y = (current_row as i32) * ((card_height + 1) as i32);

    let mut best_index = current;
    let mut best_distance = i32::MAX;

    for i in 0..spellbook_count {
        if i == current {
            continue;
        }

        let col = i % cards_per_row;
        let row = i / cards_per_row;

        let x = (grid_offset as i32) + (col as i32) * ((card_width + card_gap) as i32);
        let y = (row as i32) * ((card_height + 1) as i32);

        let dx = x - current_x;
        let dy = y - current_y;

        let is_valid = match direction {
            CardDirection::Right => dx > 0,
            CardDirection::Left => dx < 0,
            CardDirection::Down => dy > 0,
            CardDirection::Up => dy < 0,
        };

        if !is_valid {
            continue;
        }

        let distance = match direction {
            CardDirection::Right | CardDirection::Left => dx.abs() + dy.abs() * 3,
            CardDirection::Down | CardDirection::Up => dx.abs() * 3 + dy.abs(),
        };

        if distance < best_distance {
            best_distance = distance;
            best_index = i;
        }
    }

    if best_index == current {
        current
    } else {
        best_index
    }
}

fn render_spellbook_cards(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
    card_width: usize,
    card_height: usize,
    card_gap: usize,
    cards_per_row: usize,
) {
    let theme = &state.theme;
    let spellbooks = &state.codex.spellbooks;

    let selected = ui
        .search_spellbook_index()
        .unwrap_or(0)
        .min(spellbooks.len() - 1);

    let card_unit = card_width as u16 + card_gap as u16;

    let grid_width = cards_per_row as u16 * card_unit - card_gap as u16;
    let grid_offset = ((area.width as i32 - grid_width as i32) / 2).max(0) as u16;

    for (i, spellbook) in spellbooks.iter().enumerate() {
        let row = i / cards_per_row;
        let col = i % cards_per_row;

        let x = area.x + grid_offset + (col as u16) * card_unit;
        let y = area.y + (row as u16) * (card_height as u16 + 1);

        if y >= area.y + area.height || x >= area.x + area.width {
            break;
        }

        let card_area = ratatui::layout::Rect {
            x,
            y,
            width: card_width as u16,
            height: card_height as u16,
        };

        let is_selected = i == selected;
        let card_style = if is_selected {
            Style::new()
                .fg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.fg)
        };

        let spell_count = spellbook.spell_ids.len();
        let spell_count_str = format!(
            "{} spell{}",
            spell_count,
            if spell_count != 1 { "s" } else { "" }
        );

        let sigil_line = if !spellbook.sigil.is_empty() {
            format!(" {}", spellbook.sigil)
        } else {
            String::new()
        };

        let card_text = Text::from(vec![
            Line::from(vec![Span::styled(
                &sigil_line,
                Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(&spellbook.name, card_style)]),
            Line::from(""),
            Line::from(vec![Span::styled(
                &spellbook.cover,
                Style::new().fg(theme.muted),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                &spell_count_str,
                Style::new().fg(theme.muted),
            )]),
        ]);

        let card_block = if is_selected {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(theme.accent))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(theme.border))
        };

        let card = Paragraph::new(card_text)
            .block(card_block)
            .style(Style::new().bg(theme.bg).fg(theme.fg))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(card, card_area);
    }
}

fn render_spellbook_spines(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
    _spine_width: usize,
    spines_per_row: usize,
) {
    let theme = &state.theme;
    let spellbooks = &state.codex.spellbooks;

    let selected = ui
        .search_spellbook_index()
        .unwrap_or(0)
        .min(spellbooks.len().saturating_sub(1));

    let scroll = ui.search_spellbook_scroll();
    let visible_count = spellbooks.len().saturating_sub(scroll);
    let show_right_indicator = visible_count > spines_per_row;
    let visible_items = visible_count.min(spines_per_row);

    // Calculate visible range based on scroll
    let start_idx = scroll;
    let display_end_idx = (scroll + visible_items).min(spellbooks.len());
    let end_idx = display_end_idx;
    let actual_visible = end_idx - start_idx;

    // Calculate number of visible rows based on actual items being displayed
    let visible_rows = (actual_visible + spines_per_row - 1) / spines_per_row;
    let spine_height = (area.height.saturating_sub(1) as usize) / visible_rows.max(1);
    let actual_spine_width = ((area.width as usize) / spines_per_row).saturating_sub(1);
    let spine_unit = actual_spine_width as u16 + 1;

    // Draw indicator row at the bottom
    let indicator_y = area.y + area.height - 1;
    let indicator_area = ratatui::layout::Rect {
        x: area.x,
        y: indicator_y,
        width: area.width,
        height: 1,
    };

    // Position indicator: "1-3 of 7" or "1 of 7"
    let pos_text = if visible_count > spines_per_row {
        format!("{}-{} of {}", scroll + 1, display_end_idx, spellbooks.len())
    } else {
        format!("{}/{}", selected + 1, spellbooks.len())
    };

    let left_indicator = if scroll > 0 { "<" } else { " " };
    let right_indicator = if show_right_indicator { ">" } else { " " };

    let indicator_text = format!("{} {} {}", left_indicator, pos_text, right_indicator);

    let indicator = Paragraph::new(indicator_text)
        .style(Style::new().fg(theme.muted).bg(theme.bg))
        .alignment(Alignment::Center);

    frame.render_widget(indicator, indicator_area);

    // Draw spines
    for i in start_idx..end_idx {
        let local_idx = i - scroll;
        let spellbook = &spellbooks[i];

        let row = local_idx / spines_per_row;
        let col = local_idx % spines_per_row;

        let x = area.x + (col as u16) * spine_unit;
        let y = area.y + (row as u16) * (spine_height as u16);

        if y >= area.y + area.height - 1 || x >= area.x + area.width {
            break;
        }

        let spine_area = ratatui::layout::Rect {
            x,
            y,
            width: actual_spine_width as u16,
            height: spine_height as u16,
        };

        let is_selected = i == selected;

        let name_style = if is_selected {
            Style::new().fg(theme.fg).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.fg)
        };

        let spine_bg = theme.bg;
        let accent = theme.accent;
        let decor_style = Style::new().fg(accent);

        // Get spine style: use explicit style, or derive from book name hash for consistency
        let style = spellbook.style.unwrap_or_else(|| {
            let hash = spellbook
                .name
                .bytes()
                .fold(0u32, |acc, b| acc.wrapping_add(b as u32));
            SpineStyle::from_index((hash % 6) as usize)
        });

        // Build spine text with centered name and style-based decorations
        let spine_text = build_spine_decorations(
            style,
            actual_spine_width,
            &spellbook.name,
            decor_style,
            name_style,
            spine_height as usize,
        );

        let spine_block = if is_selected {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(theme.accent))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(theme.border))
        };

        let spine = Paragraph::new(spine_text)
            .block(spine_block)
            .style(Style::new().bg(spine_bg).fg(theme.fg))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        frame.render_widget(spine, spine_area);
    }

    // Render peek spine at the end of the last row (if there's more content)
    let peek_idx = display_end_idx;
    if peek_idx < spellbooks.len() {
        let total_in_view = display_end_idx - start_idx;
        let items_in_last_row = if total_in_view > 0 {
            let items = total_in_view % spines_per_row;
            if items == 0 {
                spines_per_row
            } else {
                items
            }
        } else {
            0
        };

        let peek_col = items_in_last_row;
        let peek_row = if total_in_view > spines_per_row {
            (total_in_view - 1) / spines_per_row
        } else {
            0
        };
        let peek_x = area.x + (peek_col as u16) * (actual_spine_width as u16 + 1);
        let peek_y = area.y + (peek_row as u16) * (spine_height as u16);

        let spine_area = ratatui::layout::Rect {
            x: peek_x,
            y: peek_y,
            width: actual_spine_width as u16,
            height: spine_height as u16,
        };

        let spine_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.border));

        let spine = Paragraph::new(" ")
            .block(spine_block)
            .style(Style::new().bg(theme.bg));

        frame.render_widget(spine, spine_area);
    }
}

fn render_spellbook_list(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    use ratatui::widgets::{List, ListItem};

    let theme = &state.theme;
    let spellbooks = &state.codex.spellbooks;

    if spellbooks.is_empty() {
        let empty = Paragraph::new("No spellbooks yet")
            .style(Style::new().fg(theme.muted).bg(theme.bg))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(theme.border)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let selected = ui.search_spellbook_index().unwrap_or(0);

    let items: Vec<ListItem> = spellbooks
        .iter()
        .enumerate()
        .map(|(i, sb)| {
            let prefix = if i == selected { "> " } else { "  " };
            ListItem::new(format!("{}{}", prefix, sb.name)).style(Style::new().fg(theme.fg))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(theme.border))
                .title_style(Style::new().fg(theme.accent)),
        )
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_stateful_widget(list, area, ui.search_results_state());
}

fn wrap_text_for_spine(text: &str, max_width: usize) -> Vec<String> {
    if max_width < 2 {
        return vec![String::new(); 6];
    }

    let mut lines = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return vec![String::new(); 6];
    }

    let mut current_line = String::new();
    let max_lines = 6;

    for word in &words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if test_line.len() <= max_width {
            current_line = test_line;
        } else {
            if !current_line.is_empty() && lines.len() < max_lines {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() && lines.len() < max_lines {
        lines.push(current_line);
    }

    while lines.len() < max_lines {
        lines.insert(0, String::new());
    }

    lines.truncate(max_lines);
    lines
}

fn render_command_list(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    use crate::ui::events::filter_commands;

    let theme = &state.theme;
    let query_after_colon = ui.search_query().strip_prefix(':').unwrap_or("");
    let filtered = filter_commands(query_after_colon);

    if filtered.is_empty() {
        let empty = Paragraph::new("No commands match")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Commands ")
                    .border_style(Style::new().fg(theme.border))
                    .title_style(Style::new().fg(theme.accent)),
            )
            .style(Style::new().fg(theme.muted).bg(theme.bg));
        frame.render_widget(empty, area);
        return;
    }

    let results: Vec<ListItem> = filtered
        .iter()
        .map(|(_, alias, description)| {
            let line = format!(":{}  {}", alias, description);
            ListItem::new(line).style(Style::new().fg(theme.fg))
        })
        .collect();

    let list = List::new(results)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Commands ")
                .border_style(Style::new().fg(theme.border))
                .title_style(Style::new().fg(theme.accent)),
        )
        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
        .highlight_symbol(">")
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_stateful_widget(list, area, ui.search_results_state());
}

fn render_search_results(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    let theme = &state.theme;

    let results: Vec<ListItem> = ui
        .filtered_indices()
        .iter()
        .filter_map(|&idx| state.codex.spells.get(idx))
        .map(|spell| {
            let line = format!("{}  [{}]", spell.name, spell.school);
            ListItem::new(line).style(Style::new().fg(theme.fg))
        })
        .collect();

    let list = List::new(results)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Results ")
                .border_style(Style::new().fg(theme.border))
                .title_style(Style::new().fg(theme.accent)),
        )
        .highlight_style(Style::new().add_modifier(Modifier::BOLD))
        .highlight_symbol(">")
        .style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_stateful_widget(list, area, ui.search_results_state());
}

fn render_spellbook_details<'a>(state: &'a State, ui: &mut UiState) -> Vec<Line<'a>> {
    let theme = &state.theme;

    let spellbooks = &state.codex.spellbooks;
    let selected = match ui.search_spellbook_index() {
        Some(i) if i < spellbooks.len() => i,
        _ => return vec![Line::from("")],
    };

    let spellbook = &spellbooks[selected];
    let mut lines = Vec::new();

    lines.push(Line::from(vec![Span::styled(
        &spellbook.name,
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    )]));

    if !spellbook.cover.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            &spellbook.cover,
            Style::new().fg(theme.muted),
        )]));
    }

    lines.push(Line::from(vec![Span::raw("")]));

    let spell_count = spellbook.spell_ids.len();
    if spell_count > 0 {
        lines.push(Line::from(vec![Span::styled(
            "Spells:",
            Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
        )]));

        let spells: Vec<String> = spellbook
            .spell_ids
            .iter()
            .filter_map(|id| state.codex.spells.iter().find(|s| s.id == *id))
            .map(|s| s.name.clone())
            .take(10)
            .collect();

        for spell_name in spells {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw("- "),
                Span::styled(spell_name, Style::new().fg(theme.fg)),
            ]));
        }

        if spell_count > 10 {
            lines.push(Line::from(vec![Span::styled(
                format!("  ... and {} more", spell_count - 10),
                Style::new().fg(theme.muted),
            )]));
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "No spells in this spellbook",
            Style::new().fg(theme.muted),
        )]));
    }

    lines
}

fn render_spell_details<'a>(state: &'a State, ui: &mut UiState) -> Vec<Line<'a>> {
    let theme = &state.theme;

    let selected_idx = ui.search_results_state().selected().unwrap_or(0);

    let spell_opt = ui.filtered_indices().get(selected_idx).copied();

    match spell_opt {
        Some(spell_idx) if spell_idx < state.codex.spells.len() => {
            let spell = &state.codex.spells[spell_idx];
            let glyphs_str = spell.glyphs.join(", ");

            let muted = Style::new().fg(theme.muted);
            let command_style = Style::new().fg(theme.accent);

            vec![
                Line::from(vec![
                    Span::raw("$ "),
                    Span::styled(&spell.incantation, command_style),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&spell.school, muted),
                    Span::styled(" | ", muted),
                    Span::styled(glyphs_str, muted),
                ]),
            ]
        }
        _ => vec![Line::from("")],
    }
}

/// Render spells for the selected spellbook (BrowseSpells mode)
fn render_spellbook_spells(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    let theme = &state.theme;
    let spellbook_index = match ui.selected_spellbook {
        Some(idx) => idx,
        None => return,
    };

    let spellbook = &state.codex.spellbooks[spellbook_index];

    let spells: Vec<ListItem> = spellbook
        .spell_ids
        .iter()
        .filter_map(|spell_id| state.codex.spells.iter().find(|s| s.id == *spell_id))
        .map(|spell| ListItem::new(spell.name.clone()).style(Style::new().fg(theme.fg)))
        .collect();

    let list_block = Block::bordered()
        .title(spellbook.name.clone())
        .border_style(Style::new().fg(theme.border))
        .title_style(Style::new().fg(theme.accent));

    if spells.is_empty() {
        let inner = list_block.inner(area);
        frame.render_widget(list_block, area);
        let empty_message = Paragraph::new("No spells in this spellbook")
            .style(Style::new().fg(theme.muted).bg(theme.bg));
        frame.render_widget(empty_message, inner);
    } else {
        let list = List::new(spells)
            .block(list_block)
            .highlight_style(
                Style::new()
                    .fg(theme.selection)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">")
            .style(Style::new().bg(theme.bg).fg(theme.fg));

        frame.render_stateful_widget(list, area, &mut ui.spell_list_state);
    }
}

/// Render add spell form (AddSpell mode)
fn render_add_spell_form(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    let theme = &state.theme;

    let form_block = Block::bordered()
        .title(" Add New Spell ")
        .border_style(Style::new().fg(theme.border))
        .title_style(Style::new().fg(theme.accent).bold());

    frame.render_widget(form_block, area);

    let inner_area = ratatui::layout::Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let name_value = if ui.add_spell.name.is_empty() {
        "[...]".to_string()
    } else {
        format!("[{}]", ui.add_spell.name)
    };

    let content = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::new().fg(theme.muted)),
            Span::styled(name_value, Style::new().fg(theme.fg)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "↑↓ fields  enter save  esc cancel",
            Style::new().fg(theme.muted),
        )]),
    ];

    let paragraph = Paragraph::new(content).style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_widget(paragraph, inner_area);
}

/// Render add spellbook form (AddSpellbook mode)
fn render_add_spellbook_form(
    frame: &mut Frame,
    state: &State,
    ui: &mut UiState,
    area: ratatui::layout::Rect,
) {
    let theme = &state.theme;

    let form_block = Block::bordered()
        .title(" Add New Spellbook ")
        .border_style(Style::new().fg(theme.border))
        .title_style(Style::new().fg(theme.accent).bold());

    frame.render_widget(form_block, area);

    let inner_area = ratatui::layout::Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let name_value = if ui.add_spellbook.name.is_empty() {
        "[...]".to_string()
    } else {
        format!("[{}]", ui.add_spellbook.name)
    };

    let content = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::new().fg(theme.muted)),
            Span::styled(name_value, Style::new().fg(theme.fg)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "↑↓ fields  enter save  esc cancel",
            Style::new().fg(theme.muted),
        )]),
    ];

    let paragraph = Paragraph::new(content).style(Style::new().bg(theme.bg).fg(theme.fg));

    frame.render_widget(paragraph, inner_area);
}
