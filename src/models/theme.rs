use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ThemeColors {
    pub bg: u8,
    pub fg: u8,
    pub accent: u8,
    pub muted: u8,
    pub selection: u8,
    pub border: u8,
}

impl ThemeColors {
    pub fn to_ratatui(&self) -> RatatuiColors {
        RatatuiColors {
            bg: Color::Indexed(self.bg),
            fg: Color::Indexed(self.fg),
            accent: Color::Indexed(self.accent),
            muted: Color::Indexed(self.muted),
            selection: Color::Indexed(self.selection),
            border: Color::Indexed(self.border),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Theme {
    pub name: String,
    #[serde(default = "default_dark")]
    pub bg: u8,
    #[serde(default = "default_fg")]
    pub fg: u8,
    #[serde(default = "default_accent")]
    pub accent: u8,
    #[serde(default = "default_muted")]
    pub muted: u8,
    #[serde(default = "default_selection")]
    pub selection: u8,
    #[serde(default = "default_border")]
    pub border: u8,
}

fn default_dark() -> u8 {
    0
}
fn default_fg() -> u8 {
    7
}
fn default_accent() -> u8 {
    4
}
fn default_muted() -> u8 {
    8
}
fn default_selection() -> u8 {
    12
}
fn default_border() -> u8 {
    8
}

impl Theme {
    pub fn to_colors(&self) -> RatatuiColors {
        RatatuiColors {
            bg: Color::Indexed(self.bg),
            fg: Color::Indexed(self.fg),
            accent: Color::Indexed(self.accent),
            muted: Color::Indexed(self.muted),
            selection: Color::Indexed(self.selection),
            border: Color::Indexed(self.border),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct ThemeConfig {
    #[serde(default)]
    pub selected_theme: usize,
    #[serde(default)]
    pub default: Option<Theme>,
    #[serde(default)]
    pub default_light: Option<Theme>,
    #[serde(default)]
    pub dracula: Option<Theme>,
}

impl ThemeConfig {
    pub fn get_theme(&self, name: &str) -> Option<Theme> {
        match name {
            "default" => self.default.clone(),
            "default-light" => self.default_light.clone(),
            "dracula" => self.dracula.clone(),
            _ => None,
        }
    }

    pub fn get_available_themes(&self) -> Vec<&'static str> {
        let mut themes = vec!["default", "default-light", "dracula"];
        if self.dracula.is_some() {
            themes.push("dracula");
        }
        themes
    }
}
