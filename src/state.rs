use crate::archivist::Archivist;
use crate::log_info;
use crate::models::{
    Codex, FocusTarget, JobManager, RatatuiColors, RecentAction, RecentEntry, RunMode, Spell,
    SpellId, SpellbookRef, Theme, UserSettings, ViewMode,
};
use ratatui::widgets::ListState;
use std::path::PathBuf;

const THEME_CONFIG_PATH: &str = "theme.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    BrowseSpellbooks,
    BrowseSpells,
    AddSpell,
    EditSpell,
    AddSpellbook,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::BrowseSpellbooks
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    OutputModal,
    ConfirmDialog,
    CommandPalette,
    Help,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub codex: Codex,
    pub jobs: JobManager,
    pub recents: Vec<RecentEntry>,
    pub mode: Mode,
    pub overlays: Vec<Overlay>,
    pub jobs_sidebar_open: bool,
    pub focus: FocusTarget,
    pub theme: RatatuiColors,
    pub current_theme: Theme,
    pub config: UserSettings,
    pub spellbook_browser: SpellbookBrowserState,
    pub spell_browser: SpellBrowserState,
    pub spell_form: SpellFormState,
    pub spellbook_form: SpellbookFormState,
    pub output_modal: OutputModalState,
    pub command_palette: CommandPaletteState,
    pub confirm_dialog: ConfirmDialogState,
    pub jobs_sidebar: JobsSidebarState,
}

impl AppState {
    pub fn new(codex: Codex) -> Self {
        let saved_theme = Archivist::load_theme(THEME_CONFIG_PATH);
        let current_theme = saved_theme;
        let theme = current_theme.colors();

        let config = Archivist::load_user_settings(THEME_CONFIG_PATH);
        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &config);

        let jobs = Archivist::load_jobs().unwrap_or_default();
        let recents = Archivist::load_recents().unwrap_or_default();

        Self {
            codex,
            jobs,
            recents,
            mode: Mode::default(),
            overlays: Vec::new(),
            jobs_sidebar_open: false,
            focus: FocusTarget::Main,
            theme,
            current_theme,
            config,
            spellbook_browser: SpellbookBrowserState::default(),
            spell_browser: SpellBrowserState::default(),
            spell_form: SpellFormState::default(),
            spellbook_form: SpellbookFormState::default(),
            output_modal: OutputModalState::default(),
            command_palette: CommandPaletteState::default(),
            confirm_dialog: ConfirmDialogState::default(),
            jobs_sidebar: JobsSidebarState::default(),
        }
    }

    pub fn cycle_theme(&mut self) {
        self.current_theme = self.current_theme.next();
        self.theme = self.current_theme.colors();
        log_info!("Theme changed to: {}", self.current_theme.name());
        let _ = Archivist::save_theme(THEME_CONFIG_PATH, self.current_theme);
    }

    pub fn cycle_view_mode(&mut self) {
        self.config.view_mode = self.config.view_mode.next();
        let mode_str = self.config.view_mode.as_str();
        log_info!("View mode changed to: {}", mode_str);
        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &self.config);
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

    pub fn push_overlay(&mut self, overlay: Overlay) {
        if !self.overlays.contains(&overlay) {
            self.overlays.push(overlay);
        }
    }

    pub fn pop_overlay(&mut self) {
        self.overlays.pop();
    }

    pub fn has_overlay(&self, overlay: Overlay) -> bool {
        self.overlays.contains(&overlay)
    }

    pub fn top_overlay(&self) -> Option<Overlay> {
        self.overlays.last().copied()
    }

    pub fn save_recents(&self) {
        if let Err(e) = Archivist::save_recents(&self.recents) {
            log_info!("Failed to save recents: {}", e);
        }
    }

    pub fn add_recent(
        &mut self,
        spell_id: String,
        spell_name: String,
        action: crate::models::RecentAction,
    ) {
        self.recents.retain(|r| r.spell_id != spell_id);
        self.recents
            .insert(0, RecentEntry::new(spell_id, spell_name, action));
        while self.recents.len() > 100 {
            self.recents.pop();
        }
        self.save_recents();
    }

    pub fn favorites(&self) -> Vec<&Spell> {
        self.codex.spells.iter().filter(|s| s.favorite).collect()
    }

    pub fn recent_spells(&self) -> Vec<&Spell> {
        self.recents
            .iter()
            .filter_map(|r| self.codex.spells.iter().find(|s| s.id == r.spell_id))
            .collect()
    }

    pub fn find_spell(&self, id: &str) -> Option<&Spell> {
        self.codex.spells.iter().find(|s| s.id == id)
    }

    pub fn find_spell_mut(&mut self, id: &str) -> Option<&mut Spell> {
        self.codex.spells.iter_mut().find(|s| s.id == id)
    }
}

#[derive(Debug, Clone)]
pub struct SpellbookBrowserState {
    pub selected_index: usize,
    pub items_per_row: usize,
    pub search_query: String,
    pub search_active: bool,
    pub filtered_indices: Vec<usize>,
    pub view_mode: ViewMode,
}

impl Default for SpellbookBrowserState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            items_per_row: 1,
            search_query: String::new(),
            search_active: false,
            filtered_indices: Vec::new(),
            view_mode: ViewMode::default(),
        }
    }
}

impl SpellbookBrowserState {
    pub fn total_spellbooks(&self) -> usize {
        if self.search_active && !self.search_query.is_empty() {
            self.filtered_indices.len()
        } else {
            0
        }
    }

    pub fn activate_search(&mut self) {
        self.search_active = true;
    }

    pub fn deactivate_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
        self.filtered_indices.clear();
    }
}

#[derive(Debug, Clone)]
pub struct SpellBrowserState {
    pub spellbook_ref: SpellbookRef,
    pub selected_index: usize,
    pub search_active: bool,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
}

impl Default for SpellBrowserState {
    fn default() -> Self {
        Self {
            spellbook_ref: SpellbookRef::Codex(0),
            selected_index: 0,
            search_active: false,
            search_query: String::new(),
            filtered_indices: Vec::new(),
        }
    }
}

impl SpellBrowserState {
    pub fn activate_search(&mut self) {
        self.search_active = true;
    }

    pub fn deactivate_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
        self.filtered_indices.clear();
    }

    pub fn total_spells(&self, app: &AppState) -> usize {
        if self.search_active && !self.search_query.is_empty() {
            self.filtered_indices.len()
        } else {
            match self.spellbook_ref {
                SpellbookRef::Virtual(kind) => match kind {
                    crate::models::VirtualKind::Favorites => app.favorites().len(),
                    crate::models::VirtualKind::Recent => app.recent_spells().len(),
                },
                SpellbookRef::Codex(idx) => app
                    .codex
                    .spellbooks
                    .get(idx)
                    .map(|sb| sb.spell_ids.len())
                    .unwrap_or(0),
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum SpellFormField {
    #[default]
    Name,
    Incantation,
    Lore,
    School,
    Glyphs,
    RunMode,
    Confirm,
    WorkingDir,
    Spellbook,
}

#[derive(Debug, Clone)]
pub struct SpellFormState {
    pub active_field: SpellFormField,
    pub name: String,
    pub incantation: String,
    pub lore: String,
    pub school: String,
    pub glyphs: String,
    pub run_mode: RunMode,
    pub confirm: bool,
    pub working_dir: String,
    pub spellbook_index: Option<usize>,
    pub dropdown_open: bool,
    pub dropdown_index: usize,
    pub editing_spell_id: Option<SpellId>,
    pub dirty: bool,
}

impl Default for SpellFormState {
    fn default() -> Self {
        Self {
            active_field: SpellFormField::Name,
            name: String::new(),
            incantation: String::new(),
            lore: String::new(),
            school: String::new(),
            glyphs: String::new(),
            run_mode: RunMode::Simple,
            confirm: false,
            working_dir: String::new(),
            spellbook_index: None,
            dropdown_open: false,
            dropdown_index: 0,
            editing_spell_id: None,
            dirty: false,
        }
    }
}

impl SpellFormState {
    pub fn start_edit(&mut self, spell: &Spell) {
        self.editing_spell_id = Some(spell.id.clone());
        self.name = spell.name.clone();
        self.incantation = spell.incantation.clone();
        self.lore = spell.lore.clone();
        self.school = spell.school.clone();
        self.glyphs = spell.glyphs.join(", ");
        self.run_mode = spell.run_mode;
        self.confirm = spell.confirm;
        self.working_dir = spell.working_dir.clone();
        self.dirty = false;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn to_spell(&self) -> Spell {
        let glyphs: Vec<String> = self
            .glyphs
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Spell {
            id: self
                .editing_spell_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: self.name.clone(),
            incantation: self.incantation.clone(),
            lore: self.lore.clone(),
            school: self.school.clone(),
            glyphs,
            confirm: self.confirm,
            run_mode: self.run_mode,
            working_dir: self.working_dir.clone(),
            favorite: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpellbookFormState {
    pub name: String,
    pub cover: String,
    pub sigil: String,
    pub dirty: bool,
}

impl Default for SpellbookFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            cover: String::new(),
            sigil: String::new(),
            dirty: false,
        }
    }
}

impl SpellbookFormState {
    pub fn clear(&mut self) {
        self.name.clear();
        self.cover.clear();
        self.sigil.clear();
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
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

#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    pub query: String,
    pub selected_index: usize,
    pub results: Vec<CommandItem>,
}

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub name: String,
    pub description: String,
    pub action: CommandAction,
}

#[derive(Debug, Clone)]
pub enum CommandAction {
    NewSpell,
    NewSpellbook,
    BrowseSpellbooks,
    BrowseSpells,
    ToggleJobs,
    CardView,
    SpineView,
    AutoView,
    CycleTheme,
    Help,
    Import,
    Export,
}

#[derive(Debug, Clone, Default)]
pub struct ConfirmDialogState {
    pub message: String,
    pub confirmed: bool,
}

impl ConfirmDialogState {
    pub fn new(message: String) -> Self {
        Self {
            message,
            confirmed: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JobsSidebarState {
    pub selected_index: usize,
    pub scroll_offset: usize,
}

impl JobsSidebarState {
    pub fn total_jobs(&self, app: &AppState) -> usize {
        app.jobs.jobs.len()
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub codex: Codex,
    pub theme: RatatuiColors,
    pub current_theme: Theme,
    pub user_settings: UserSettings,
    pub recents: Vec<RecentEntry>,
}

impl State {
    pub fn new(codex: Codex) -> Self {
        let saved_theme = Archivist::load_theme(THEME_CONFIG_PATH);
        let current_theme = saved_theme;
        let theme = current_theme.colors();

        let user_settings = Archivist::load_user_settings(THEME_CONFIG_PATH);

        let _ = Archivist::save_user_settings(THEME_CONFIG_PATH, &user_settings);

        let recents = Archivist::load_recents().unwrap_or_default();

        Self {
            codex,
            theme,
            current_theme,
            user_settings,
            recents,
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

    pub fn get_spell(&self, spell_id: &str) -> Option<&crate::models::Spell> {
        self.codex.spells.iter().find(|s| s.id == spell_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Theme, ThemeConfig, UserSettings, ViewMode};

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
            user_settings,
            recents: vec![],
        }
    }

    #[test]
    fn test_update_spell_modifies_existing() {
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
    fn test_delete_spell_removes_from_codex() {
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
    fn test_delete_spell_removes_from_spellbooks() {
        let spell_id = uuid::Uuid::new_v4().to_string();
        let spellbook_id = uuid::Uuid::new_v4().to_string();

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
                id: Some(spellbook_id.clone()),
                name: "TestSpellbook".to_string(),
                cover: String::new(),
                sigil: "*".to_string(),
                spell_ids: vec![spell_id.clone()],
                spells: vec![],
                style: None,
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
