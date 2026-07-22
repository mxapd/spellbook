use crate::archivist::Archivist;
use crate::log_info;
use crate::models::{Codex, RecentAction, RecentEntry, Spell, RatatuiColors, UserSettings};

pub(crate) const CONFIG_PATH: &str = "config.toml";

#[derive(Debug, Clone, Default)]
pub struct State {
    pub codex: Codex,
    pub user_settings: UserSettings,
    pub recents: Vec<RecentEntry>,
}

impl State {
    /// Pure constructor — all loading happens in caller.
    pub fn new(codex: Codex, user_settings: UserSettings) -> Self {
        Self {
            codex,
            user_settings,
            recents: Vec::new(),
        }
    }

    /// Derived color palette from current theme setting.
    pub fn theme(&self) -> RatatuiColors {
        self.user_settings.theme.colors()
    }

    // ── Settings ──────────────────────────────────────

    pub fn cycle_theme(&mut self) {
        self.user_settings.theme = self.user_settings.theme.next();
        log_info!("Theme changed to: {}", self.user_settings.theme.name());
        if let Err(e) = Archivist::save_user_settings(CONFIG_PATH, &self.user_settings) {
            log_info!("Failed to save theme: {}", e);
        }
    }

    pub fn cycle_view_mode(&mut self) {
        self.user_settings.view_mode = self.user_settings.view_mode.next();
        let mode_str = self.user_settings.view_mode.as_str();
        log_info!("View mode changed to: {}", mode_str);
        if let Err(e) = Archivist::save_user_settings(CONFIG_PATH, &self.user_settings) {
            log_info!("Failed to save view mode: {}", e);
        }
    }

    // ── Spell CRUD ────────────────────────────────────

    /// Update an existing spell in the codex. Persists on success.
    pub fn update_spell(&mut self, spell: Spell) -> Result<(), String> {
        if let Some(existing) = self.codex.spells.iter_mut().find(|s| s.id == spell.id) {
            *existing = spell;
            Archivist::save(&self.codex, "codex.toml")
                .map_err(|e| format!("Failed to save: {}", e))?;
            log_info!("Spell updated successfully");
            Ok(())
        } else {
            Err("Spell not found".to_string())
        }
    }

    /// Delete a spell by id. Also removes it from all spellbooks. Persists on success.
    pub fn delete_spell(&mut self, spell_id: &str) -> Result<(), String> {
        let initial_len = self.codex.spells.len();
        self.codex.spells.retain(|s| s.id != spell_id);

        if self.codex.spells.len() < initial_len {
            for spellbook in &mut self.codex.spellbooks {
                spellbook.spell_ids.retain(|id| id != spell_id);
            }
            Archivist::save(&self.codex, "codex.toml")
                .map_err(|e| format!("Failed to save: {}", e))?;
            log_info!("Spell deleted successfully");
            Ok(())
        } else {
            Err("Spell not found".to_string())
        }
    }

    /// Delete a spellbook by name. Persists on success.
    pub fn delete_spellbook(&mut self, spellbook_name: &str) -> Result<(), String> {
        let initial_len = self.codex.spellbooks.len();
        self.codex.spellbooks.retain(|sb| sb.name != spellbook_name);

        if self.codex.spellbooks.len() < initial_len {
            Archivist::save(&self.codex, "codex.toml")
                .map_err(|e| format!("Failed to save: {}", e))?;
            log_info!("Spellbook '{}' deleted successfully", spellbook_name);
            Ok(())
        } else {
            Err("Spellbook not found".to_string())
        }
    }

    /// Toggle favorite. Returns new state on success.
    pub fn toggle_favorite(&mut self, id: &str) -> Result<bool, String> {
        // Mutable borrow must end before the save below
        let is_fav = {
            let spell = self.codex.spells.iter_mut().find(|s| s.id == id);
            match spell {
                Some(s) => {
                    s.favorite = !s.favorite;
                    s.favorite
                }
                None => return Err("Spell not found".to_string()),
            }
        };

        Archivist::save(&self.codex, "codex.toml")
            .map_err(|e| format!("Failed to save: {}", e))?;
        Ok(is_fav)
    }

    /// Read-only spell lookup by id.
    pub fn get_spell(&self, spell_id: &str) -> Option<&Spell> {
        self.codex.spells.iter().find(|s| s.id == spell_id)
    }

    // ── Recents ───────────────────────────────────────

    /// Add a recent entry, deduplicate, evict beyond 100, then persist.
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

    // ── Codex management ──────────────────────────────

    /// Reload the codex from disk.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RunMode, Spell, Spellbook, ViewMode};
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

        // Write an empty codex so Archivist can load it
        let empty = Codex {
            spells: vec![],
            spellbooks: vec![],
        };
        let content = toml::to_string_pretty(&empty).expect("Failed to serialize empty codex");
        fs::write("codex.toml", content).expect("Failed to write test codex");

        TestGuard {
            _temp_dir: temp_dir,
            original_cwd,
        }
    }

    fn make_test_state(codex: Codex) -> State {
        let user_settings = UserSettings {
            view_mode: ViewMode::List,
            ..Default::default()
        };
        State {
            codex,
            user_settings,
            recents: vec![],
        }
    }

    // === update_spell ===

    #[test]
    #[serial]
    fn update_spell_modifies_existing() {
        let _guard = setup_test_env();
        let spell_id = uuid::Uuid::new_v4().to_string();
        let original_spell = Spell {
            id: spell_id.clone(),
            name: "Original".to_string(),
            command: "echo original".to_string(),
            description: String::new(),
            category: String::new(),
            tags: vec![],
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
            command: "echo updated".to_string(),
            description: "New lore".to_string(),
            category: String::new(),
            tags: vec![],
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
        assert_eq!(retrieved.unwrap().description, "New lore");
        assert_eq!(retrieved.unwrap().run_mode, RunMode::Tui);
    }

    #[test]
    fn update_spell_not_found_error() {
        let mut state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        let spell = Spell {
            id: "nonexistent-id".to_string(),
            name: "Ghost".to_string(),
            command: "echo ghost".to_string(),
            description: String::new(),
            category: String::new(),
            tags: vec![],
            confirm: false,
            run_mode: RunMode::Simple,
            working_dir: String::new(),
            favorite: false,
        };

        let result = state.update_spell(spell);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // === delete_spell ===

    #[test]
    #[serial]
    fn delete_spell_removes_from_codex() {
        let _guard = setup_test_env();
        let spell_id = uuid::Uuid::new_v4().to_string();
        let mut state = make_test_state(Codex {
            spells: vec![Spell {
                id: spell_id.clone(),
                name: "ToDelete".to_string(),
                command: "echo delete".to_string(),
                description: String::new(),
                category: String::new(),
                tags: vec![],
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
    fn delete_spell_removes_from_spellbooks() {
        let _guard = setup_test_env();
        let spell_id = uuid::Uuid::new_v4().to_string();

        let mut state = make_test_state(Codex {
            spells: vec![Spell {
                id: spell_id.clone(),
                name: "TestSpell".to_string(),
                command: "echo test".to_string(),
                description: String::new(),
                category: String::new(),
                tags: vec![],
                confirm: false,
                run_mode: RunMode::Simple,
                working_dir: String::new(),
                favorite: false,
            }],
            spellbooks: vec![Spellbook {
                name: "TestSpellbook".to_string(),
                cover: String::new(),
                decoration: "*".to_string(),
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
    fn delete_spell_not_found_error() {
        let mut state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        let result = state.delete_spell("nonexistent-id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // === get_spell ===

    #[test]
    fn get_spell_found() {
        let spell_id = uuid::Uuid::new_v4().to_string();
        let state = make_test_state(Codex {
            spells: vec![Spell {
                id: spell_id.clone(),
                name: "FindMe".to_string(),
                command: "echo findme".to_string(),
                description: String::new(),
                category: String::new(),
                tags: vec![],
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
    fn get_spell_not_found() {
        let state = make_test_state(Codex {
            spells: vec![],
            spellbooks: vec![],
        });

        let found = state.get_spell("nonexistent");
        assert!(found.is_none());
    }

    // === recents ===

    #[test]
    fn add_recent_inserts_at_front() {
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
    fn add_recent_deduplicates() {
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
    fn add_recent_evicts_at_100() {
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
