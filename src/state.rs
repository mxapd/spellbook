use crate::log_info;
use crate::models::{Codex, RatatuiColors, UserSettings};
use crate::persistence::Archivist;

const THEME_CONFIG_PATH: &str = "theme.toml";

pub struct State {
    pub codex: Codex,
    pub theme: RatatuiColors,
    pub current_theme_index: usize,
    pub theme_names: Vec<&'static str>,
    pub user_settings: UserSettings,
}

impl State {
    pub fn new(codex: Codex) -> Self {
        let theme_names = vec![
            "default",
            "default-light",
            "dracula",
            "gruvbox-dark",
            "gruvbox-light",
            "nord",
            "catppuccin",
            "one-dark",
            "solarized-dark",
            "solarized-light",
        ];

        let saved_index = Archivist::load_theme_index(THEME_CONFIG_PATH);
        let current_theme_index = saved_index.min(theme_names.len() - 1);
        let theme = Self::get_theme_by_index(current_theme_index);

        let user_settings = Archivist::load_user_settings(THEME_CONFIG_PATH);

        // Ensure settings are saved to config (for first-time setup or updates)
        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &user_settings);

        Self {
            codex,
            theme,
            current_theme_index,
            theme_names,
            user_settings,
        }
    }

    fn get_theme_by_index(index: usize) -> RatatuiColors {
        match index {
            0 => RatatuiColors::dark_default(),
            1 => RatatuiColors::light_default(),
            2 => RatatuiColors::dracula(),
            3 => RatatuiColors::gruvbox_dark(),
            4 => RatatuiColors::gruvbox_light(),
            5 => RatatuiColors::nord(),
            6 => RatatuiColors::catppuccin_mocha(),
            7 => RatatuiColors::one_dark(),
            8 => RatatuiColors::solarized_dark(),
            9 => RatatuiColors::solarized_light(),
            _ => RatatuiColors::dark_default(),
        }
    }

    pub fn cycle_theme(&mut self) {
        self.current_theme_index = (self.current_theme_index + 1) % self.theme_names.len();
        self.theme = Self::get_theme_by_index(self.current_theme_index);

        let theme_name = self.theme_names[self.current_theme_index];
        log_info!("Theme changed to: {}", theme_name);

        let _ = Archivist::save_theme_index(THEME_CONFIG_PATH, self.current_theme_index);
    }

    pub fn cycle_view_mode(&mut self) {
        self.user_settings.view_mode = self.user_settings.view_mode.next();
        let mode_str = self.user_settings.view_mode.as_str();
        log_info!("View mode changed to: {}", mode_str);
        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &self.user_settings);
    }
}
