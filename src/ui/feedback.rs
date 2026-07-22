use std::time::Instant;

use ratatui::{
    layout::Alignment,
    style::Style,
    text::Line,
    widgets::Paragraph,
};

/// Visual severity level for footer feedback messages.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FeedbackLevel {
    Success,
    Error,
    Info,
}

/// A timed feedback message shown in the footer/status bar.
#[derive(PartialEq, Clone, Debug)]
pub struct Feedback {
    pub message: String,
    pub level: FeedbackLevel,
    pub until: Instant,
}

impl Feedback {
    pub fn paragraph<'a>(
        &self,
        theme: &crate::models::RatatuiColors,
    ) -> Paragraph<'a> {
        let color = match self.level {
            FeedbackLevel::Success => ratatui::style::Color::Green,
            FeedbackLevel::Error => ratatui::style::Color::Red,
            FeedbackLevel::Info => theme.accent,
        };
        let single_line = self
            .message
            .lines()
            .next()
            .unwrap_or(&self.message)
            .to_string();
        Paragraph::new(single_line)
            .style(Style::new().fg(color).bg(theme.bg))
            .alignment(Alignment::Center)
    }
}

/// A brief visual flash on a specific action target.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FlashAction {
    Spell {
        spellbook_index: usize,
        spell_index: usize,
    },
    Spellbook {
        spellbook_index: usize,
    },
    SearchResult {
        index: usize,
    },
}
