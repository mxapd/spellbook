use crate::archivist::Archivist;
use crate::log_info;
use crate::models::{Codex, RatatuiColors, RecentAction, RecentEntry, Theme, UserSettings};

const THEME_CONFIG_PATH: &str = "theme.toml";

#[derive(Debug, Clone, Default)]
pub struct State {
    pub codex: Codex,
    pub theme: RatatuiColors,
    pub current_theme: Theme,
    pub user_settings: UserSettings,
    pub recents: Vec<RecentEntry>,
    pub launch_dir: String,
}

#[derive(Debug, Clone, Default)]
pub struct OutputModalState {
    pub content: Vec<String>,
    pub scroll_offset: usize,
    pub is_streaming: bool,
    pub exit_code: Option<i32>,
}

impl OutputModalState {
    pub const MAX_LINES: usize = 10000;

    pub fn add_line(&mut self, line: String) {
        if self.content.len() >= Self::MAX_LINES {
            self.content.remove(0);
        }
        self.content.push(line);
    }

    pub fn is_truncated(&self) -> bool {
        self.content.len() >= Self::MAX_LINES
    }
}

impl State {
    pub fn new(codex: Codex) -> Self {
        let saved_theme = Archivist::load_theme(THEME_CONFIG_PATH);
        let current_theme = saved_theme;
        let theme = current_theme.colors();

        let user_settings = Archivist::load_user_settings(THEME_CONFIG_PATH);

        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &user_settings);

        let recents = Archivist::load_recents().unwrap_or_default();

        let launch_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| "/".to_string()));

        Self {
            codex,
            theme,
            current_theme,
            user_settings,
            recents,
            launch_dir,
        }
    }

    #[cfg(test)]
    pub fn new_test(codex: Codex) -> Self {
        let current_theme = crate::models::Theme::default();
        let theme = current_theme.colors();
        let user_settings = crate::models::UserSettings::default();

        let launch_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".to_string());

        Self {
            codex,
            theme,
            current_theme,
            user_settings,
            recents: Vec::new(),
            launch_dir,
        }
    }

    pub fn add_recent(&mut self, spell_id: String, spell_name: String, action: RecentAction) {
        self.recents.retain(|r| r.spell_id != spell_id);
        self.recents
            .insert(0, RecentEntry::new(spell_id, spell_name, action));
        while self.recents.len() > 100 {
            self.recents.pop();
        }
        if let Err(e) = Archivist::save_recents(&self.recents) {
            log_info!("Failed to save recents: {}", e);
        }
    }

    pub fn save_recents(&self) {
        if let Err(e) = Archivist::save_recents(&self.recents) {
            log_info!("Failed to save recents: {}", e);
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

    pub fn update_spell(&mut self, spell: crate::models::Spell) -> Result<(), String> {
        if let Some(existing) = self.codex.spells.iter_mut().find(|s| s.id == spell.id) {
            *existing = spell;
            if let Err(e) = Archivist::save(&self.codex, "codex.toml") {
                return Err(format!("Failed to save: {}", e));
            }
            log_info!("Spell updated successfully");
            Ok(())
        } else {
            Err("Spell not found".to_string())
        }
    }

    pub fn delete_spell(&mut self, spell_id: &str) -> Result<(), String> {
        let initial_len = self.codex.spells.len();
        self.codex.spells.retain(|s| s.id != spell_id);

        if self.codex.spells.len() < initial_len {
            for spellbook in &mut self.codex.spellbooks {
                spellbook.spell_ids.retain(|id| id != spell_id);
            }
            if let Err(e) = Archivist::save(&self.codex, "codex.toml") {
                return Err(format!("Failed to save: {}", e));
            }
            log_info!("Spell deleted successfully");
            Ok(())
        } else {
            Err("Spell not found".to_string())
        }
    }

    pub fn delete_spellbook(&mut self, spellbook_name: &str) -> Result<(), String> {
        let initial_len = self.codex.spellbooks.len();
        self.codex.spellbooks.retain(|sb| sb.name != spellbook_name);

        if self.codex.spellbooks.len() < initial_len {
            if let Err(e) = Archivist::save(&self.codex, "codex.toml") {
                return Err(format!("Failed to save: {}", e));
            }
            log_info!("Spellbook '{}' deleted successfully", spellbook_name);
            Ok(())
        } else {
            Err("Spellbook not found".to_string())
        }
    }

    pub fn get_spell(&self, spell_id: &str) -> Option<&crate::models::Spell> {
        self.codex.spells.iter().find(|s| s.id == spell_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RunMode, Spell, Theme, UserSettings, ViewMode};
    use serial_test::serial;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct TestGuard {
        _temp_dir: TempDir,
        original_cwd: PathBuf,
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_cwd);
        }
    }

    fn setup_test_env() -> TestGuard {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let original_cwd = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");

        let content = r#"[spellbook]"#;
        fs::write("codex.toml", content).expect("Failed to write test codex");

        TestGuard {
            _temp_dir: temp_dir,
            original_cwd,
        }
    }

    fn make_test_state(codex: Codex) -> State {
        let theme = Theme::default();
        let user_settings = UserSettings {
            view_mode: ViewMode::List,
            ..Default::default()
        };
        State {
            codex,
            theme: theme.colors(),
            current_theme: theme,
            launch_dir: String::from("/home/xam/"),
            user_settings,
            recents: vec![],
        }
    }

    #[test]
    #[serial]
    fn test_update_spell_modifies_existing() {
        let _guard = setup_test_env();
        let spell_id = uuid::Uuid::new_v4().to_string();
        let original_spell = Spell {
            id: spell_id.clone(),
            name: "Original".to_string(),
            incantation: "echo original".to_string(),
            lore: String::new(),
            school: String::new(),
            glyphs: vec![],
            confirm: false,
            run_mode: RunMode::Simple,
            working_dir: String::new(),
            favorite: false,
        };

        let mut state = make_test_state(Codex {
            spells: vec![original_spell],
            spellbooks: vec![],
        });

        let updated_spell = Spell {
            id: spell_id.clone(),
            name: "Updated".to_string(),
            incantation: "echo updated".to_string(),
            lore: "New lore".to_string(),
            school: String::new(),
            glyphs: vec![],
            confirm: false,
            run_mode: RunMode::Tui,
            working_dir: String::new(),
            favorite: true,
        };

        let result = state.update_spell(updated_spell);
        assert!(result.is_ok(), "Update should succeed");

        let retrieved = state.get_spell(&spell_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Updated");
        assert_eq!(retrieved.unwrap().lore, "New lore");
        assert_eq!(retrieved.unwrap().run_mode, RunMode::Tui);
    }

    #[test]
    fn test_update_spell_not_found_error() {
        let mut state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        let spell = Spell {
            id: "nonexistent-id".to_string(),
            name: "Ghost".to_string(),
            incantation: "echo ghost".to_string(),
            lore: String::new(),
            school: String::new(),
            glyphs: vec![],
            confirm: false,
            run_mode: RunMode::Simple,
            working_dir: String::new(),
            favorite: false,
        };

        let result = state.update_spell(spell);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    #[serial]
    fn test_delete_spell_removes_from_codex() {
        let _guard = setup_test_env();
        let spell_id = uuid::Uuid::new_v4().to_string();
        let mut state = make_test_state(Codex {
            spells: vec![Spell {
                id: spell_id.clone(),
                name: "ToDelete".to_string(),
                incantation: "echo delete".to_string(),
                lore: String::new(),
                school: String::new(),
                glyphs: vec![],
                confirm: false,
                run_mode: RunMode::Simple,
                working_dir: String::new(),
                favorite: false,
            }],
            spellbooks: vec![],
        });

        assert!(state.get_spell(&spell_id).is_some());
        let result = state.delete_spell(&spell_id);
        assert!(result.is_ok());
        assert!(state.get_spell(&spell_id).is_none());
    }

    #[test]
    #[serial]
    fn test_delete_spell_removes_from_spellbooks() {
        let _guard = setup_test_env();
        let spell_id = uuid::Uuid::new_v4().to_string();

        let mut state = make_test_state(Codex {
            spells: vec![Spell {
                id: spell_id.clone(),
                name: "TestSpell".to_string(),
                incantation: "echo test".to_string(),
                lore: String::new(),
                school: String::new(),
                glyphs: vec![],
                confirm: false,
                run_mode: RunMode::Simple,
                working_dir: String::new(),
                favorite: false,
            }],
            spellbooks: vec![crate::models::Spellbook {
                name: "TestSpellbook".to_string(),
                cover: String::new(),
                sigil: "*".to_string(),
                spell_ids: vec![spell_id.clone()],
                spells: vec![],
                style: None,
                color: None,
            }],
        });

        state.delete_spell(&spell_id).unwrap();

        let spellbook = state
            .codex
            .spellbooks
            .iter()
            .find(|s| s.name == "TestSpellbook");
        assert!(spellbook.is_some());
        assert!(!spellbook.unwrap().spell_ids.contains(&spell_id));
    }

    #[test]
    fn test_delete_spell_not_found_error() {
        let mut state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        let result = state.delete_spell("nonexistent-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_spell_by_id_found() {
        let spell_id = uuid::Uuid::new_v4().to_string();
        let state = make_test_state(Codex {
            spells: vec![Spell {
                id: spell_id.clone(),
                name: "FindMe".to_string(),
                incantation: "echo findme".to_string(),
                lore: String::new(),
                school: String::new(),
                glyphs: vec![],
                confirm: false,
                run_mode: RunMode::Simple,
                working_dir: String::new(),
                favorite: false,
            }],
            spellbooks: vec![],
        });

        let found = state.get_spell(&spell_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "FindMe");
    }

    #[test]
    fn test_get_spell_by_id_not_found() {
        let state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        let found = state.get_spell("nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_add_recent_inserts_at_front() {
        let mut state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        state.add_recent("id1".to_string(), "Spell1".to_string(), RecentAction::Run);
        state.add_recent("id2".to_string(), "Spell2".to_string(), RecentAction::Copy);

        assert_eq!(state.recents.len(), 2);
        assert_eq!(state.recents[0].spell_name, "Spell2");
        assert_eq!(state.recents[1].spell_name, "Spell1");
    }

    #[test]
    fn test_add_recent_deduplicates() {
        let mut state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        state.add_recent("id1".to_string(), "Spell1".to_string(), RecentAction::Run);
        state.add_recent("id1".to_string(), "Spell1".to_string(), RecentAction::Copy);

        assert_eq!(state.recents.len(), 1);
        assert!(matches!(state.recents[0].action, RecentAction::Copy));
    }

    #[test]
    fn test_add_recent_evicts_at_100() {
        let mut state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        for i in 0..105 {
            state.add_recent(format!("id{}", i), format!("Spell{}", i), RecentAction::Run);
        }

        assert_eq!(state.recents.len(), 100);
        assert_eq!(state.recents[0].spell_name, "Spell104");
        assert_eq!(state.recents[99].spell_name, "Spell5");
    }
}
