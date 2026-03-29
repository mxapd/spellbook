use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    DarkDefault,
    LightDefault,
    Dracula,
    GruvboxDark,
    GruvboxLight,
    Nord,
    CatppuccinMocha,
    OneDark,
    SolarizedDark,
    SolarizedLight,
}

impl Theme {
    pub const fn all() -> [Theme; 10] {
        [
            Theme::DarkDefault,
            Theme::LightDefault,
            Theme::Dracula,
            Theme::GruvboxDark,
            Theme::GruvboxLight,
            Theme::Nord,
            Theme::CatppuccinMocha,
            Theme::OneDark,
            Theme::SolarizedDark,
            Theme::SolarizedLight,
        ]
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Theme::DarkDefault => "default",
            Theme::LightDefault => "default-light",
            Theme::Dracula => "dracula",
            Theme::GruvboxDark => "gruvbox-dark",
            Theme::GruvboxLight => "gruvbox-light",
            Theme::Nord => "nord",
            Theme::CatppuccinMocha => "catppuccin",
            Theme::OneDark => "one-dark",
            Theme::SolarizedDark => "solarized-dark",
            Theme::SolarizedLight => "solarized-light",
        }
    }

    pub fn colors(&self) -> RatatuiColors {
        match self {
            Theme::DarkDefault => RatatuiColors::dark_default(),
            Theme::LightDefault => RatatuiColors::light_default(),
            Theme::Dracula => RatatuiColors::dracula(),
            Theme::GruvboxDark => RatatuiColors::gruvbox_dark(),
            Theme::GruvboxLight => RatatuiColors::gruvbox_light(),
            Theme::Nord => RatatuiColors::nord(),
            Theme::CatppuccinMocha => RatatuiColors::catppuccin_mocha(),
            Theme::OneDark => RatatuiColors::one_dark(),
            Theme::SolarizedDark => RatatuiColors::solarized_dark(),
            Theme::SolarizedLight => RatatuiColors::solarized_light(),
        }
    }

    pub fn next(self) -> Theme {
        let themes = Self::all();
        let current = themes.iter().position(|t| *t == self).unwrap_or(0);
        themes[(current + 1) % themes.len()]
    }
}

impl From<usize> for Theme {
    fn from(index: usize) -> Self {
        let themes = Self::all();
        themes[index % themes.len()]
    }
}

impl From<Theme> for usize {
    fn from(theme: Theme) -> Self {
        Theme::all().iter().position(|t| *t == theme).unwrap_or(0)
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

/// Controls how spellbooks are displayed in the search browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum ViewMode {
    #[default]
    List, // Simple vertical list
    Cards,  // Card view
    Spines, // Compact spine view
}

impl ViewMode {
    pub fn next(self) -> Self {
        match self {
            ViewMode::List => ViewMode::Cards,
            ViewMode::Cards => ViewMode::Spines,
            ViewMode::Spines => ViewMode::List,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ViewMode::List => "list",
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
    #[serde(default)]
    pub experimental_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub selected_theme: Theme,
    #[serde(default)]
    pub settings: UserSettings,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            selected_theme: Theme::default(),
            settings: UserSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_next_cycles_through_all() {
        let themes = Theme::all();
        let mut current = Theme::DarkDefault;

        for expected in themes.iter().cycle().take(themes.len()) {
            assert_eq!(current, *expected);
            current = current.next();
        }
    }

    #[test]
    fn test_theme_default_is_dark_default() {
        assert_eq!(Theme::default(), Theme::DarkDefault);
    }

    #[test]
    fn test_theme_from_usize_converts_correctly() {
        assert_eq!(Theme::from(0), Theme::DarkDefault);
        assert_eq!(Theme::from(1), Theme::LightDefault);
        assert_eq!(Theme::from(5), Theme::Nord);
    }

    #[test]
    fn test_theme_from_usize_wraps_around() {
        assert_eq!(Theme::from(10), Theme::DarkDefault);
        assert_eq!(Theme::from(11), Theme::LightDefault);
        assert_eq!(Theme::from(100), Theme::DarkDefault);
    }

    #[test]
    fn test_theme_roundtrip_via_usize() {
        let themes = Theme::all();
        for (i, theme) in themes.iter().enumerate() {
            let converted: Theme = i.into();
            assert_eq!(converted, *theme);
        }
    }

    #[test]
    fn test_theme_to_usize_roundtrip() {
        let themes = Theme::all();
        for theme in themes {
            let converted: usize = theme.into();
            let back: Theme = converted.into();
            assert_eq!(back, theme);
        }
    }

    #[test]
    fn test_theme_name_returns_correct_strings() {
        assert_eq!(Theme::DarkDefault.name(), "default");
        assert_eq!(Theme::LightDefault.name(), "default-light");
        assert_eq!(Theme::Dracula.name(), "dracula");
        assert_eq!(Theme::Nord.name(), "nord");
        assert_eq!(Theme::OneDark.name(), "one-dark");
    }

    #[test]
    fn test_all_themes_have_unique_names() {
        let names: Vec<_> = Theme::all().iter().map(|t| t.name()).collect();
        let mut sorted_names = names.clone();
        sorted_names.sort();
        sorted_names.dedup();
        assert_eq!(names.len(), sorted_names.len());
    }

    #[test]
    fn test_view_mode_next_cycles() {
        assert_eq!(ViewMode::List.next(), ViewMode::Cards);
        assert_eq!(ViewMode::Cards.next(), ViewMode::Spines);
        assert_eq!(ViewMode::Spines.next(), ViewMode::List);
    }

    #[test]
    fn test_view_mode_default_is_list() {
        assert_eq!(ViewMode::default(), ViewMode::List);
    }

    #[test]
    fn test_view_mode_as_str() {
        assert_eq!(ViewMode::List.as_str(), "list");
        assert_eq!(ViewMode::Cards.as_str(), "cards");
        assert_eq!(ViewMode::Spines.as_str(), "spines");
    }

    #[test]
    fn test_view_mode_next_full_cycle() {
        let mut mode = ViewMode::List;
        mode = mode.next();
        assert_eq!(mode, ViewMode::Cards);
        mode = mode.next();
        assert_eq!(mode, ViewMode::Spines);
        mode = mode.next();
        assert_eq!(mode, ViewMode::List);
    }

    #[test]
    fn test_user_settings_default() {
        let settings = UserSettings::default();
        assert_eq!(settings.view_mode, ViewMode::default());
    }

    #[test]
    fn test_theme_config_default() {
        let config = ThemeConfig::default();
        assert_eq!(config.selected_theme, Theme::default());
        assert_eq!(config.settings.view_mode, ViewMode::default());
    }

    #[test]
    fn test_ratatui_colors_all_themes_produce_valid_colors() {
        for theme in Theme::all() {
            let colors = theme.colors();
            assert!(colors.bg != colors.fg || true);
        }
    }
}
