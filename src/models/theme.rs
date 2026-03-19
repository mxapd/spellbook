use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatatuiColors {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub muted: Color,
    pub selection: Color,
    pub border: Color,
}

impl Default for RatatuiColors {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl RatatuiColors {
    pub fn dark_default() -> Self {
        Self {
            bg: Color::Indexed(0),
            fg: Color::Indexed(7),
            accent: Color::Indexed(4),
            muted: Color::Indexed(8),
            selection: Color::Indexed(12),
            border: Color::Indexed(8),
        }
    }

    pub fn light_default() -> Self {
        Self {
            bg: Color::Indexed(7),
            fg: Color::Indexed(0),
            accent: Color::Indexed(4),
            muted: Color::Indexed(8),
            selection: Color::Indexed(14),
            border: Color::Indexed(8),
        }
    }

    pub fn dracula() -> Self {
        Self {
            bg: Color::Indexed(0),
            fg: Color::Indexed(213),
            accent: Color::Indexed(97),
            muted: Color::Indexed(139),
            selection: Color::Indexed(141),
            border: Color::Indexed(98),
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            bg: Color::Indexed(234),
            fg: Color::Indexed(223),
            accent: Color::Indexed(208),
            muted: Color::Indexed(246),
            selection: Color::Indexed(66),
            border: Color::Indexed(246),
        }
    }

    pub fn gruvbox_light() -> Self {
        Self {
            bg: Color::Indexed(229),
            fg: Color::Indexed(235),
            accent: Color::Indexed(166),
            muted: Color::Indexed(244),
            selection: Color::Indexed(65),
            border: Color::Indexed(244),
        }
    }

    pub fn nord() -> Self {
        Self {
            bg: Color::Indexed(0),
            fg: Color::Indexed(188),
            accent: Color::Indexed(68),
            muted: Color::Indexed(244),
            selection: Color::Indexed(73),
            border: Color::Indexed(60),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            bg: Color::Indexed(234),
            fg: Color::Indexed(205),
            accent: Color::Indexed(204),
            muted: Color::Indexed(243),
            selection: Color::Indexed(149),
            border: Color::Indexed(145),
        }
    }

    pub fn one_dark() -> Self {
        Self {
            bg: Color::Indexed(0),
            fg: Color::Indexed(188),
            accent: Color::Indexed(167),
            muted: Color::Indexed(145),
            selection: Color::Indexed(139),
            border: Color::Indexed(60),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            bg: Color::Indexed(234),
            fg: Color::Indexed(223),
            accent: Color::Indexed(166),
            muted: Color::Indexed(244),
            selection: Color::Indexed(136),
            border: Color::Indexed(240),
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            bg: Color::Indexed(7),
            fg: Color::Indexed(22),
            accent: Color::Indexed(166),
            muted: Color::Indexed(244),
            selection: Color::Indexed(166),
            border: Color::Indexed(250),
        }
    }
}

use serde::{Deserialize, Serialize};

/// Controls how spellbooks are displayed in the search browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum ViewMode {
    #[default]
    Auto, // Responsive: cards when they fit, spines otherwise
    Cards,  // Always show cards
    Spines, // Always show spines
}

impl ViewMode {
    pub fn next(self) -> Self {
        match self {
            ViewMode::Auto => ViewMode::Cards,
            ViewMode::Cards => ViewMode::Spines,
            ViewMode::Spines => ViewMode::Auto,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ViewMode::Auto => "auto",
            ViewMode::Cards => "cards",
            ViewMode::Spines => "spines",
        }
    }
}

/// User preferences and UI settings (persisted to config)
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct UserSettings {
    #[serde(default)]
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct ThemeConfig {
    #[serde(default)]
    pub selected_theme: usize,
    #[serde(default)]
    pub settings: UserSettings,
}
