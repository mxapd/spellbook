use crate::archivist::Archivist;
use crate::log_info;
use crate::models::{Codex, RatatuiColors, Theme, UserSettings};

const THEME_CONFIG_PATH: &str = "theme.toml";

pub struct State {
    pub codex: Codex,
    pub theme: RatatuiColors,
    pub current_theme: Theme,
    pub user_settings: UserSettings,
}

impl State {
    pub fn new(codex: Codex) -> Self {
        let saved_theme = Archivist::load_theme(THEME_CONFIG_PATH);
        let current_theme = saved_theme;
        let theme = current_theme.colors();

        let user_settings = Archivist::load_user_settings(THEME_CONFIG_PATH);

        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &user_settings);

        Self {
            codex,
            theme,
            current_theme,
            user_settings,
        }
    }

    pub fn cycle_theme(&mut self) {
        self.current_theme = self.current_theme.next();
        self.theme = self.current_theme.colors();

        log_info!("Theme changed to: {}", self.current_theme.name());

        let _ = Archivist::save_theme(THEME_CONFIG_PATH, self.current_theme);
    }

    pub fn cycle_view_mode(&mut self) {
        self.user_settings.view_mode = self.user_settings.view_mode.next();
        let mode_str = self.user_settings.view_mode.as_str();
        log_info!("View mode changed to: {}", mode_str);
        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &self.user_settings);
    }

    pub fn reload_codex(&mut self) {
        match Archivist::load("codex.toml") {
            Ok(new_codex) => {
                self.codex = new_codex;
                log_info!("Codex reloaded successfully");
            }
            Err(e) => {
                log_info!("Failed to reload codex: {}", e);
            }
        }
    }
}
