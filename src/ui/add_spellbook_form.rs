#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddSpellbookField {
    Name,
    Cover,
    Decoration,
}

impl Default for AddSpellbookField {
    fn default() -> Self {
        AddSpellbookField::Name
    }
}

#[derive(Debug, Clone, Default)]
pub struct AddSpellbookForm {
    pub field: AddSpellbookField,
    pub name: String,
    pub cover: String,
    pub decoration: String,
    pub is_editing: bool,
    pub message: Option<(String, bool)>, // (message, is_error)
    pub has_unsaved: bool,
}

impl AddSpellbookForm {
    pub fn clear(&mut self) {
        self.name.clear();
        self.cover.clear();
        self.decoration.clear();
        self.field = AddSpellbookField::Name;
        self.is_editing = false;
        self.message = None;
        self.has_unsaved = false;
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }

    pub fn next_field(&mut self) {
        self.field = match self.field {
            AddSpellbookField::Name => AddSpellbookField::Cover,
            AddSpellbookField::Cover => AddSpellbookField::Decoration,
            AddSpellbookField::Decoration => AddSpellbookField::Name,
        };
    }

    pub fn prev_field(&mut self) {
        self.field = match self.field {
            AddSpellbookField::Name => AddSpellbookField::Decoration,
            AddSpellbookField::Cover => AddSpellbookField::Name,
            AddSpellbookField::Decoration => AddSpellbookField::Cover,
        };
    }
}

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, state: &crate::state::State, ui: &mut crate::ui::UiState) {
    let area = frame.area();
    render_in_area(frame, state, ui, area);
}

pub fn render_in_area(
    frame: &mut Frame,
    state: &crate::state::State,
    ui: &mut crate::ui::UiState,
    area: Rect,
) {
    let theme = state.theme();
    let form = &ui.add_spellbook;

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Form fields
            Constraint::Length(3), // Message area
        ])
        .split(area);

    // Title - changes based on edit mode
    let title_text = if form.is_editing {
        "Add New Spellbook [EDITING]"
    } else {
        "Add New Spellbook"
    };
    let title = Paragraph::new(title_text)
        .style(Style::new().fg(theme.accent).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Form fields
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Cover
            Constraint::Length(3), // Decoration
        ])
        .split(chunks[1]);

    // Name field
    render_field(
        frame,
        form_chunks[0],
        "Name",
        '*',
        &form.name,
        form.field == AddSpellbookField::Name,
        form.is_editing && form.field == AddSpellbookField::Name,
        &theme,
    );

    // Cover field
    render_field(
        frame,
        form_chunks[1],
        "Cover",
        ':',
        &form.cover,
        form.field == AddSpellbookField::Cover,
        form.is_editing && form.field == AddSpellbookField::Cover,
        &theme,
    );

    // Decoration field
    render_field(
        frame,
        form_chunks[2],
        "Decoration",
        '@',
        &form.decoration,
        form.field == AddSpellbookField::Decoration,
        form.is_editing && form.field == AddSpellbookField::Decoration,
        &theme,
    );

    // Message area
    let message_text = if let Some((msg, is_error)) = &form.message {
        Line::from(vec![Span::styled(
            msg.clone(),
            Style::new().fg(if *is_error { Color::Red } else { theme.accent }),
        )])
    } else if form.is_editing {
        Line::from("Type to edit | Enter: done | Esc: cancel edit")
    } else {
        Line::from("Tab/Enter: edit field | Ctrl+S: save | Esc: cancel | arrows: navigate")
    };
    let message = Paragraph::new(message_text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::new().fg(theme.muted));
    frame.render_widget(message, chunks[2]);
}

fn render_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    icon: char,
    value: &str,
    is_active: bool,
    is_editing: bool,
    theme: &crate::models::RatatuiColors,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_editing {
            Style::new().fg(theme.selection)
        } else if is_active {
            Style::new().fg(theme.accent)
        } else {
            Style::new().fg(theme.muted)
        })
        .title(format!(
            "{} {}{}",
            icon,
            label,
            if is_editing { " [EDIT]" } else { "" }
        ))
        .title_style(if is_editing {
            Style::new()
                .fg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.fg)
        });

    let text = Paragraph::new(value)
        .block(block)
        .style(Style::new().fg(theme.fg).bg(theme.bg));

    frame.render_widget(text, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_spellbook_field_default() {
        assert_eq!(AddSpellbookField::default(), AddSpellbookField::Name);
    }

    #[test]
    fn test_add_spellbook_form_default() {
        let form = AddSpellbookForm::default();
        assert_eq!(form.field, AddSpellbookField::Name);
        assert!(form.name.is_empty());
        assert!(form.cover.is_empty());
        assert!(form.decoration.is_empty());
        assert!(form.message.is_none());
        assert!(!form.has_unsaved);
    }

    #[test]
    fn test_add_spellbook_form_clear() {
        let mut form = AddSpellbookForm {
            field: AddSpellbookField::Decoration,
            name: "Test".to_string(),
            cover: "Cover".to_string(),
            decoration: "*".to_string(),
            message: Some(("Error".to_string(), true)),
            has_unsaved: true,
            is_editing: true,
        };

        form.clear();

        assert_eq!(form.field, AddSpellbookField::Name);
        assert!(form.name.is_empty());
        assert!(form.cover.is_empty());
        assert!(form.decoration.is_empty());
        assert!(form.message.is_none());
        assert!(!form.has_unsaved);
    }

    #[test]
    fn test_add_spellbook_form_is_valid() {
        let mut form = AddSpellbookForm::default();
        assert!(!form.is_valid());

        form.name = "Test Spellbook".to_string();
        assert!(form.is_valid());

        form.name = "   ".to_string();
        assert!(!form.is_valid());
    }

    #[test]
    fn test_add_spellbook_form_next_field() {
        let mut form = AddSpellbookForm::default();

        form.next_field();
        assert_eq!(form.field, AddSpellbookField::Cover);

        form.next_field();
        assert_eq!(form.field, AddSpellbookField::Decoration);

        form.next_field();
        assert_eq!(form.field, AddSpellbookField::Name);
    }

    #[test]
    fn test_add_spellbook_form_prev_field() {
        let mut form = AddSpellbookForm::default();

        form.prev_field();
        assert_eq!(form.field, AddSpellbookField::Decoration);

        form.prev_field();
        assert_eq!(form.field, AddSpellbookField::Cover);

        form.prev_field();
        assert_eq!(form.field, AddSpellbookField::Name);
    }
}
