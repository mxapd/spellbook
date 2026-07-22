use crate::models::{Codex, RecentEntry, ThemeConfig, UserSettings};
use crate::error::{LoadError, SaveError};
use crate::validation::validate_codex;
use crate::{log_debug, log_info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
//
fn atomic_write(path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = format!("{}.tmp", path);
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}
fn ensure_spellbook_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".spellbook");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}
//
pub struct Archivist;
//
impl Archivist {
    pub fn load(path: &str) -> Result<Codex, LoadError> {
        log_info!("Loading codex from: {}", path);

        let contents = fs::read_to_string(path.to_string()).map_err(|e| LoadError::Read {
            path: path.to_string(),
            cause: e.to_string(),
        })?;

        log_debug!("Loaded {} bytes from codex", contents.len());

        let mut codex: Codex = toml::from_str(&contents).map_err(|e| LoadError::Parse {
            path: path.to_string(),
            cause: e.to_string(),
        })?;

        for spell in &mut codex.spells {
            if spell.id.is_empty() {
                spell.id = uuid::Uuid::new_v4().to_string();
            }
        }

        validate_codex(&codex)?;

        log_info!(
            "Loaded {} spells and {} spellbooks",
            codex.spells.len(),
            codex.spellbooks.len()
        );
        Ok(codex)
    }

    pub fn save(codex: &Codex, path: &str) -> Result<(), SaveError> {
        log_info!("Saving codex to: {}", path);

        let content = toml::to_string_pretty(codex).map_err(|e| SaveError::Serialize {
            cause: e.to_string(),
        })?;

        atomic_write(path, &content).map_err(|e| SaveError::Write {
            path: path.to_string(),
            cause: e.to_string(),
        })?;

        log_info!("Codex saved successfully");
        Ok(())
    }

    pub fn load_user_settings(path: &str) -> UserSettings {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return UserSettings::default(),
        };

        let config: ThemeConfig = match toml::from_str(&contents) {
            Ok(c) => c,
            Err(_) => return UserSettings::default(),
        };

        config.user_settings()
    }

    pub fn save_user_settings(
        path: &str,
        settings: &UserSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = ThemeConfig::from(settings.clone());
        let new_content = toml::to_string_pretty(&config)?;
        atomic_write(path, &new_content)?;
        Ok(())
    }
    //
    //    pub fn load_jobs() -> Result<JobManager, Box<dyn std::error::Error>> {
    //        let path = ensure_spellbook_dir().join("jobs.toml");
    //        if !path.exists() {
    //            return Ok(JobManager::default());
    //        }
    //        let contents = fs::read_to_string(&path)?;
    //        let data: crate::models::JobsData = toml::from_str(&contents)?;
    //        Ok(JobManager {
    //            jobs: data.jobs,
    //            next_id: data.next_id,
    //            ..Default::default()
    //        })
    //    }
    //
    //    pub fn save_jobs(jobs: &JobManager) -> Result<(), Box<dyn std::error::Error>> {
    //        let path = ensure_spellbook_dir().join("jobs.toml");
    //        let data = crate::models::JobsData {
    //            jobs: jobs.jobs.clone(),
    //            next_id: jobs.next_id,
    //        };
    //        let content = toml::to_string_pretty(&data)?;
    //        atomic_write(path.to_str().unwrap_or("jobs.toml"), &content)?;
    //        Ok(())
    //    }
    //
    pub fn load_recents() -> Result<Vec<RecentEntry>, Box<dyn std::error::Error>> {
        let path = ensure_spellbook_dir().join("recents.toml");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let contents = fs::read_to_string(&path)?;
        #[derive(Deserialize)]
        struct RecentsFile {
            recents: Vec<RecentEntry>,
        }
        let data: RecentsFile = toml::from_str(&contents)?;
        Ok(data.recents)
    }

    pub fn save_recents(recents: &[RecentEntry]) -> Result<(), Box<dyn std::error::Error>> {
        let path = ensure_spellbook_dir().join("recents.toml");
        #[derive(Serialize)]
        struct RecentsFile<'a> {
            recents: &'a [RecentEntry],
        }
        let data = RecentsFile { recents };
        let content = toml::to_string_pretty(&data)?;
        atomic_write(path.to_str().unwrap_or("recents.toml"), &content)?;
        Ok(())
    }
    //
    //    pub fn append_spell(
    //        path: &str,
    //        spell: &crate::models::Spell,
    //        spellbook: Option<&str>,
    //    ) -> Result<(), Box<dyn std::error::Error>> {
    //        log_info!("Saving spell: {} to codex", spell.name);
    //
    //        let mut contents = fs::read_to_string(path)?;
    //
    //        contents.push_str("\n[[spells]]\n");
    //        contents.push_str(&format!("name = \"{}\"\n", spell.name));
    //        contents.push_str(&format!("command = \"{}\"\n", spell.command));
    //        contents.push_str(&format!("description = \"{}\"\n", spell.description));
    //        contents.push_str(&format!("category = \"{}\"\n", spell.category));
    //        contents.push_str("tags = [");
    //        for (i, tag) in spell.tags.iter().enumerate() {
    //            if i > 0 {
    //                contents.push_str(", ");
    //            }
    //            contents.push_str(&format!("\"{}\"", glyph));
    //        }
    //        contents.push_str("]\n");
    //
    //        if let Some(book_name) = spellbook {
    //            let spellbook_section = format!("[[spellbooks]]\nname = \"{}\"", book_name);
    //            if let Some(pos) = contents.find(&spellbook_section) {
    //                let rest = &contents[pos..];
    //
    //                let insert_pos = if let Some(next_pos) = rest[2..].find("[[spellbooks]]") {
    //                    pos + 2 + next_pos
    //                } else {
    //                    contents.len()
    //                };
    //                contents.insert_str(insert_pos, &format!(", \"{}\"", spell.name));
    //            }
    //        }
    //
    //        fs::write(path, contents)?;
    //        if let Some(book) = spellbook {
    //            log_info!(
    //                "Spell '{}' saved successfully (added to spellbook: {})",
    //                spell.name,
    //                book
    //            );
    //        } else {
    //            log_info!("Spell '{}' saved successfully (no spellbook)", spell.name);
    //        }
    //        Ok(())
    //    }
    //
    //    pub fn append_spellbook(
    //        path: &str,
    //        name: String,
    //        cover: Option<String>,
    //        decoration: Option<String>,
    //    ) -> Result<(), Box<dyn std::error::Error>> {
    //        log_info!("Creating spellbook: {}", name);
    //
    //        let contents = fs::read_to_string(path)?;
    //
    //        let mut new_spellbook = format!("\n[[spellbooks]]\nname = \"{}\"\n", name);
    //
    //        if let Some(c) = cover {
    //            if !c.is_empty() {
    //                new_spellbook.push_str(&format!("cover = \"{}\"\n", c));
    //            }
    //        }
    //
    //        if let Some(s) = decoration {
    //            if !s.is_empty() {
    //                new_spellbook.push_str(&format!("decoration = \"{}\"\n", s));
    //            }
    //        }
    //
    //        fs::write(path, &contents)?;
    //        std::io::Write::write_all(
    //            &mut std::fs::OpenOptions::new().append(true).open(path)?,
    //            new_spellbook.as_bytes(),
    //        )?;
    //
    //        log_info!("Spellbook '{}' created successfully", name);
    //        Ok(())
    //    }
    //
    //    pub fn update_spell_background(
    //        path: &str,
    //        spell_name: &str,
    //        background: bool,
    //    ) -> Result<(), Box<dyn std::error::Error>> {
    //        log_info!(
    //            "Updating spell '{}' background={} in codex",
    //            spell_name,
    //            background
    //        );
    //
    //        let contents = fs::read_to_string(path)?;
    //        let mut lines: Vec<String> = contents.lines().map(String::from).collect();
    //
    //        let mut in_target_spell = false;
    //        let mut spell_start = 0;
    //        let mut spell_end = 0;
    //        let mut found_background_line = None;
    //
    //        for (i, line) in lines.iter().enumerate() {
    //            let trimmed = line.trim();
    //
    //            if trimmed == "[[spells]]" {
    //                if in_target_spell {
    //                    spell_end = i;
    //                    break;
    //                }
    //                spell_start = i;
    //                in_target_spell = true;
    //            } else if in_target_spell && trimmed.starts_with("name = ") {
    //                let name_value = trimmed
    //                    .strip_prefix("name = ")
    //                    .and_then(|s| s.strip_prefix('"'))
    //                    .and_then(|s| s.strip_suffix('"'))
    //                    .unwrap_or("");
    //
    //                if name_value == spell_name {
    //                    continue;
    //                } else {
    //                    spell_end = i;
    //                    break;
    //                }
    //            } else if in_target_spell && trimmed.starts_with("background = ") {
    //                found_background_line = Some(i);
    //            }
    //        }
    //
    //        if !in_target_spell {
    //            return Err(format!("Spell '{}' not found in codex", spell_name).into());
    //        }
    //
    //        if spell_end == 0 {
    //            spell_end = lines.len();
    //        }
    //
    //        if let Some(line_idx) = found_background_line {
    //            lines[line_idx] = format!("background = {}", background);
    //        } else {
    //            let insert_idx = spell_start + 1;
    //            lines.insert(insert_idx, format!("background = {}", background));
    //        }
    //
    //        let new_contents = lines.join("\n");
    //        atomic_write(path, &new_contents)?;
    //
    //        log_info!("Spell '{}' updated successfully", spell_name);
    //        Ok(())
    //    }
    //
    //    pub fn export_codex(codex: &Codex, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    //        log_info!("Exporting codex to: {}", path);
    //        let content = toml::to_string_pretty(codex)?;
    //        atomic_write(path, &content)?;
    //        log_info!("Codex exported successfully");
    //        Ok(())
    //    }
    //
    //    pub fn export_spellbook(
    //        codex: &Codex,
    //        spellbook_name: &str,
    //        path: &str,
    //    ) -> Result<(), Box<dyn std::error::Error>> {
    //        log_info!("Exporting spellbook '{}' to: {}", spellbook_name, path);
    //
    //        let spellbook = codex
    //            .spellbooks
    //            .iter()
    //            .find(|sb| sb.name == spellbook_name)
    //            .ok_or_else(|| format!("Spellbook '{}' not found", spellbook_name))?;
    //
    //        let spells: Vec<crate::models::Spell> = spellbook
    //            .spell_ids
    //            .iter()
    //            .filter_map(|id| codex.spells.iter().find(|s| &s.id == id))
    //            .cloned()
    //            .collect();
    //
    //        #[derive(Serialize)]
    //        struct ExportedSpellbook {
    //            #[serde(skip_serializing_if = "Vec::is_empty")]
    //            spells: Vec<crate::models::Spell>,
    //            spellbooks: Vec<ExportedSpellbookData>,
    //        }
    //
    //        #[derive(Serialize)]
    //        struct ExportedSpellbookData {
    //            name: String,
    //            #[serde(skip_serializing_if = "String::is_empty")]
    //            cover: String,
    //            #[serde(skip_serializing_if = "String::is_empty")]
    //            decoration: String,
    //            #[serde(skip_serializing_if = "Vec::is_empty")]
    //            spell_ids: Vec<String>,
    //            #[serde(skip_serializing_if = "Vec::is_empty")]
    //            spells: Vec<String>,
    //            #[serde(skip_serializing_if = "Option::is_none")]
    //            style: Option<SpineStyle>,
    //        }
    //
    //        let export = ExportedSpellbook {
    //            spells,
    //            spellbooks: vec![ExportedSpellbookData {
    //                name: spellbook.name.clone(),
    //                cover: spellbook.cover.clone(),
    //                decoration: spellbook.decoration.clone(),
    //                spell_ids: spellbook.spell_ids.clone(),
    //                spells: spellbook.spells.clone(),
    //                style: spellbook.style,
    //            }],
    //        };
    //
    //        let content = toml::to_string_pretty(&export)?;
    //        atomic_write(path, &content)?;
    //        log_info!("Spellbook '{}' exported successfully", spellbook_name);
    //        Ok(())
    //    }
    //
    //    pub fn import_codex(path: &str) -> Result<Codex, Box<dyn std::error::Error>> {
    //        log_info!("Importing codex from: {}", path);
    //        let contents = fs::read_to_string(path)?;
    //        let mut codex: Codex = toml::from_str(&contents)?;
    //        validate_codex(&codex)?;
    //
    //        for spell in codex.spells.iter_mut() {
    //            if spell.id.is_empty() {
    //                spell.id = uuid::Uuid::new_v4().to_string();
    //            }
    //        }
    //
    //        log_info!(
    //            "Imported {} spells and {} spellbooks",
    //            codex.spells.len(),
    //            codex.spellbooks.len()
    //        );
    //        Ok(codex)
    //    }
    //
    //    pub fn merge_codex(target: &mut Codex, source: Codex, strategy: MergeStrategy) -> MergeResult {
    //        log_info!("Merging codex with {} spells", source.spells.len());
    //
    //        let mut added_spells = Vec::new();
    //        let mut added_spellbooks = Vec::new();
    //        let mut conflicts = Vec::new();
    //
    //        for spell in source.spells {
    //            let existing = target.spells.iter().find(|s| s.id == spell.id);
    //
    //            if let Some(_existing) = existing {
    //                match strategy {
    //                    MergeStrategy::Skip => {
    //                        conflicts.push(MergeConflict::Spell {
    //                            id: spell.id.clone(),
    //                            name: spell.name.clone(),
    //                        });
    //                    }
    //                    MergeStrategy::Overwrite => {
    //                        let idx = target.spells.iter().position(|s| s.id == spell.id).unwrap();
    //                        target.spells[idx] = spell.clone();
    //                    }
    //                    MergeStrategy::Rename => {
    //                        let mut new_spell = spell.clone();
    //                        new_spell.id = uuid::Uuid::new_v4().to_string();
    //                        new_spell.name = format!("{} (imported)", spell.name);
    //                        added_spells.push(new_spell.name.clone());
    //                        target.spells.push(new_spell);
    //                    }
    //                }
    //            } else {
    //                added_spells.push(spell.name.clone());
    //                target.spells.push(spell);
    //            }
    //        }
    //
    //        for spellbook in source.spellbooks {
    //            let existing = target
    //                .spellbooks
    //                .iter()
    //                .find(|sb| sb.name == spellbook.name);
    //
    //            if let Some(_existing) = existing {
    //                match strategy {
    //                    MergeStrategy::Skip => {
    //                        conflicts.push(MergeConflict::Spellbook {
    //                            name: spellbook.name.clone(),
    //                        });
    //                    }
    //                    MergeStrategy::Overwrite => {
    //                        let idx = target
    //                            .spellbooks
    //                            .iter()
    //                            .position(|sb| sb.name == spellbook.name)
    //                            .unwrap();
    //                        target.spellbooks[idx] = spellbook;
    //                    }
    //                    MergeStrategy::Rename => {
    //                        let mut new_spellbook = spellbook;
    //                        new_spellbook.name = format!("{} (imported)", new_spellbook.name);
    //                        added_spellbooks.push(new_spellbook.name.clone());
    //                        target.spellbooks.push(new_spellbook);
    //                    }
    //                }
    //            } else {
    //                added_spellbooks.push(spellbook.name.clone());
    //                target.spellbooks.push(spellbook);
    //            }
    //        }
    //
    //        MergeResult {
    //            added_spells,
    //            added_spellbooks,
    //            conflicts,
    //        }
    //    }
    //}

    // TESTS

    //// for merging codexes
    //#[derive(Debug, Clone, Copy, PartialEq)]
    //pub enum MergeStrategy {
    //    Skip,
    //    Overwrite,
    //    Rename,
    //}
    //
    //#[derive(Debug)]
    //pub enum MergeConflict {
    //    Spell { id: String, name: String },
    //    Spellbook { name: String },
    //}
    //
    //#[derive(Debug)]
    //pub struct MergeResult {
    //    pub added_spells: Vec<String>,
    //    pub added_spellbooks: Vec<String>,
    //    pub conflicts: Vec<MergeConflict>,
    //}
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ValidationError;
    use crate::models::Codex;
    use crate::test_utils::{make_codex, make_spell, make_spellbook};
    use std::path::PathBuf;

    /// Temp directory that cleans itself up on drop.
    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("spellbook_test_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn write_file(path: &PathBuf, content: &str) {
        std::fs::write(path, content).unwrap();
    }

    // === loading ===

    #[test]
    fn load_valid_codex() {
        let dir = TestDir::new();
        let path = dir.path("codex.toml");

        let spell = make_spell("List Files");
        let spellbook = make_spellbook("Test Commands", vec![spell.id.clone()]);
        let codex = Codex {
            spells: vec![spell],
            spellbooks: vec![spellbook],
        };

        // Save a valid codex first, then load it.
        Archivist::save(&codex, path.to_str().unwrap()).unwrap();
        let loaded = Archivist::load(path.to_str().unwrap()).unwrap();

        assert_eq!(loaded.spells.len(), 1);
        assert_eq!(loaded.spells[0].name, "List Files");
        assert_eq!(loaded.spellbooks.len(), 1);
        assert_eq!(loaded.spellbooks[0].name, "Test Commands");
    }

    #[test]
    fn load_file_not_found() {
        let dir = TestDir::new();
        let path = dir.path("missing.toml");

        let result = Archivist::load(path.to_str().unwrap());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LoadError::Read { .. }));
    }

    #[test]
    fn load_invalid_toml() {
        let dir = TestDir::new();
        let path = dir.path("bad.toml");
        write_file(&path, "this is not valid toml [[[");

        let result = Archivist::load(path.to_str().unwrap());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LoadError::Parse { .. }));
    }

    #[test]
    fn load_validation_failure() {
        let dir = TestDir::new();
        let path = dir.path("invalid.toml");
        write_file(
            &path,
            r#"
[[spells]]
id = "spell-1"
name = "List Files"
command = "echo list files"
description = ""
category = ""
tags = []
confirm = false
run_mode = "simple"
working_dir = ""
favorite = false

[[spells]]
id = "spell-2"
name = "List Files"
command = "echo list files"
description = ""
category = ""
tags = []
confirm = false
run_mode = "simple"
working_dir = ""
favorite = false
"#,
        );

        let result = Archivist::load(path.to_str().unwrap());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LoadError::Validation(ValidationError::DuplicateSpellName { .. })
        ));
    }

    #[test]
    fn load_generates_missing_ids() {
        let dir = TestDir::new();
        let path = dir.path("noids.toml");
        write_file(
            &path,
            r#"
[[spells]]
id = ""
name = "List Files"
command = "echo list files"
description = ""
category = ""
tags = []
confirm = false
run_mode = "simple"
working_dir = ""
favorite = false
"#,
        );

        let result = Archivist::load(path.to_str().unwrap());
        assert!(result.is_ok());
        let codex = result.unwrap();
        assert!(!codex.spells[0].id.is_empty());
    }

    // === saving ===

    #[test]
    fn save_creates_file() {
        let dir = TestDir::new();
        let path = dir.path("saved.toml");

        let codex = make_codex();
        let result = Archivist::save(&codex, path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn save_roundtrip() {
        let dir = TestDir::new();
        let path = dir.path("roundtrip.toml");

        let spell = make_spell("List Files");
        let spellbook = make_spellbook("My Commands", vec![spell.id.clone()]);
        let codex = Codex {
            spells: vec![spell],
            spellbooks: vec![spellbook],
        };

        Archivist::save(&codex, path.to_str().unwrap()).unwrap();
        let loaded = Archivist::load(path.to_str().unwrap()).unwrap();

        assert_eq!(loaded.spells.len(), 1);
        assert_eq!(loaded.spells[0].name, "List Files");
        assert_eq!(loaded.spellbooks.len(), 1);
        assert_eq!(loaded.spellbooks[0].name, "My Commands");
    }

    #[test]
    fn save_pretty_toml() {
        let dir = TestDir::new();
        let path = dir.path("pretty.toml");

        let codex = make_codex();
        Archivist::save(&codex, path.to_str().unwrap()).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("spells = []"));
        assert!(content.contains("spellbooks = []"));
    }

    // === atomic write ===

    #[test]
    fn atomic_write_roundtrip() {
        let dir = TestDir::new();
        let path = dir.path("atomic.toml");

        atomic_write(path.to_str().unwrap(), "hello world").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");
        assert!(!path.with_extension("tmp").exists());
    }
}
