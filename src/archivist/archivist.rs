use crate::models::{Codex, JobManager, RecentEntry, Theme, ThemeConfig, UserSettings};
use crate::validation::validate_codex;
use crate::{log_debug, log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

pub struct Archivist;

impl Archivist {
    pub fn load(path: &str) -> Result<Codex, Box<dyn std::error::Error>> {
        log_info!("Loading codex from: {}", path);
        let contents = fs::read_to_string(path)?;
        log_debug!("Loaded {} bytes from codex", contents.len());

        let mut codex: Codex = toml::from_str(&contents)?;

        let needs_migration = codex.spells.iter().any(|s| s.id.is_empty())
            || codex.spellbooks.iter().any(|sb| !sb.spells.is_empty());

        if needs_migration {
            log_info!("Migrating codex from v1 to v2 format...");
            for spell in &mut codex.spells {
                if spell.id.is_empty() {
                    spell.id = uuid::Uuid::new_v4().to_string();
                }
            }
            for spellbook in &mut codex.spellbooks {
                if spellbook.spell_ids.is_empty() && !spellbook.spells.is_empty() {
                    let resolved_ids: Vec<String> = spellbook
                        .spells
                        .iter()
                        .filter_map(|name| {
                            codex
                                .spells
                                .iter()
                                .find(|s| &s.name == name)
                                .map(|s| s.id.clone())
                        })
                        .collect();
                    spellbook.spell_ids = resolved_ids;
                }
            }
            if let Err(e) = Self::save(&codex, path) {
                log_warn!("Failed to save migrated codex: {}", e);
            }
        } else {
            for spell in &mut codex.spells {
                if spell.id.is_empty() {
                    spell.id = uuid::Uuid::new_v4().to_string();
                }
            }
        }

        for spellbook in &mut codex.spellbooks {
            if spellbook.spell_ids.is_empty() && !spellbook.spells.is_empty() {
                let resolved_ids: Vec<String> = spellbook
                    .spells
                    .iter()
                    .filter_map(|name| {
                        codex
                            .spells
                            .iter()
                            .find(|s| &s.name == name)
                            .map(|s| s.id.clone())
                    })
                    .collect();
                spellbook.spell_ids = resolved_ids;
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

    pub fn save(codex: &Codex, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Saving codex to: {}", path);
        let content = toml::to_string_pretty(codex)?;
        atomic_write(path, &content)?;
        log_info!("Codex saved successfully");
        Ok(())
    }

    pub fn load_theme(path: &str) -> Theme {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Theme::default(),
        };

        let config: ThemeConfig = match toml::from_str(&contents) {
            Ok(c) => c,
            Err(_) => return Theme::default(),
        };

        config.selected_theme
    }

    pub fn save_theme(path: &str, theme: Theme) -> Result<(), Box<dyn std::error::Error>> {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                let config = ThemeConfig {
                    selected_theme: theme,
                    ..Default::default()
                };
                let new_content = toml::to_string_pretty(&config)?;
                fs::write(path, new_content)?;
                return Ok(());
            }
        };

        let mut config: ThemeConfig = toml::from_str(&contents).unwrap_or_default();
        config.selected_theme = theme;

        let new_content = toml::to_string_pretty(&config)?;
        atomic_write(path, &new_content)?;
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

        config.settings
    }

    pub fn save_user_settings(
        path: &str,
        settings: &UserSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                let config = ThemeConfig {
                    selected_theme: Theme::default(),
                    settings: settings.clone(),
                };
                let new_content = toml::to_string_pretty(&config)?;
                fs::write(path, new_content)?;
                return Ok(());
            }
        };

        let mut config: ThemeConfig = toml::from_str(&contents).unwrap_or_default();
        config.settings = settings.clone();

        let new_content = toml::to_string_pretty(&config)?;
        atomic_write(path, &new_content)?;
        Ok(())
    }

    pub fn load_jobs() -> Result<JobManager, Box<dyn std::error::Error>> {
        let path = ensure_spellbook_dir().join("jobs.toml");
        if !path.exists() {
            return Ok(JobManager::default());
        }
        let contents = fs::read_to_string(&path)?;
        let data: crate::models::JobsData = toml::from_str(&contents)?;
        Ok(JobManager {
            jobs: data.jobs,
            next_id: data.next_id,
            ..Default::default()
        })
    }

    pub fn save_jobs(jobs: &JobManager) -> Result<(), Box<dyn std::error::Error>> {
        let path = ensure_spellbook_dir().join("jobs.toml");
        let data = crate::models::JobsData {
            jobs: jobs.jobs.clone(),
            next_id: jobs.next_id,
        };
        let content = toml::to_string_pretty(&data)?;
        atomic_write(path.to_str().unwrap_or("jobs.toml"), &content)?;
        Ok(())
    }

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

    pub fn append_spell(
        path: &str,
        spell: &crate::models::Spell,
        spellbook: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Saving spell: {} to codex", spell.name);

        let mut contents = fs::read_to_string(path)?;

        contents.push_str("\n[[spells]]\n");
        contents.push_str(&format!("name = \"{}\"\n", spell.name));
        contents.push_str(&format!("incantation = \"{}\"\n", spell.incantation));
        contents.push_str(&format!("lore = \"{}\"\n", spell.lore));
        contents.push_str(&format!("school = \"{}\"\n", spell.school));
        contents.push_str("glyphs = [");
        for (i, glyph) in spell.glyphs.iter().enumerate() {
            if i > 0 {
                contents.push_str(", ");
            }
            contents.push_str(&format!("\"{}\"", glyph));
        }
        contents.push_str("]\n");

        if let Some(book_name) = spellbook {
            let spellbook_section = format!("[[spellbooks]]\nname = \"{}\"", book_name);
            if let Some(pos) = contents.find(&spellbook_section) {
                let rest = &contents[pos..];

                let insert_pos = if let Some(next_pos) = rest[2..].find("[[spellbooks]]") {
                    pos + 2 + next_pos
                } else {
                    contents.len()
                };
                contents.insert_str(insert_pos, &format!(", \"{}\"", spell.name));
            }
        }

        fs::write(path, contents)?;
        if let Some(book) = spellbook {
            log_info!(
                "Spell '{}' saved successfully (added to spellbook: {})",
                spell.name,
                book
            );
        } else {
            log_info!("Spell '{}' saved successfully (no spellbook)", spell.name);
        }
        Ok(())
    }

    pub fn append_spellbook(
        path: &str,
        name: &str,
        cover: Option<&str>,
        sigil: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Creating spellbook: {}", name);

        let contents = fs::read_to_string(path)?;

        let mut new_spellbook = format!("\n[[spellbooks]]\nname = \"{}\"\n", name);

        if let Some(c) = cover {
            if !c.is_empty() {
                new_spellbook.push_str(&format!("cover = \"{}\"\n", c));
            }
        }

        if let Some(s) = sigil {
            if !s.is_empty() {
                new_spellbook.push_str(&format!("sigil = \"{}\"\n", s));
            }
        }

        fs::write(path, &contents)?;
        std::io::Write::write_all(
            &mut std::fs::OpenOptions::new().append(true).open(path)?,
            new_spellbook.as_bytes(),
        )?;

        log_info!("Spellbook '{}' created successfully", name);
        Ok(())
    }

    pub fn update_spell_background(
        path: &str,
        spell_name: &str,
        background: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info!(
            "Updating spell '{}' background={} in codex",
            spell_name,
            background
        );

        let contents = fs::read_to_string(path)?;
        let mut lines: Vec<String> = contents.lines().map(String::from).collect();

        let mut in_target_spell = false;
        let mut spell_start = 0;
        let mut spell_end = 0;
        let mut found_background_line = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed == "[[spells]]" {
                if in_target_spell {
                    spell_end = i;
                    break;
                }
                spell_start = i;
                in_target_spell = true;
            } else if in_target_spell && trimmed.starts_with("name = ") {
                let name_value = trimmed
                    .strip_prefix("name = ")
                    .and_then(|s| s.strip_prefix('"'))
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or("");

                if name_value == spell_name {
                    continue;
                } else {
                    spell_end = i;
                    break;
                }
            } else if in_target_spell && trimmed.starts_with("background = ") {
                found_background_line = Some(i);
            }
        }

        if !in_target_spell {
            return Err(format!("Spell '{}' not found in codex", spell_name).into());
        }

        if spell_end == 0 {
            spell_end = lines.len();
        }

        if let Some(line_idx) = found_background_line {
            lines[line_idx] = format!("background = {}", background);
        } else {
            let insert_idx = spell_start + 1;
            lines.insert(insert_idx, format!("background = {}", background));
        }

        let new_contents = lines.join("\n");
        atomic_write(path, &new_contents)?;

        log_info!("Spell '{}' updated successfully", spell_name);
        Ok(())
    }
}
