use crate::models::{Codex, Theme, ThemeConfig, UserSettings};
use crate::validation::validate_codex;
use crate::{log_debug, log_info};
use std::fs;

pub struct Archivist;

impl Archivist {
    /// Loads and validates a Codex from a TOML file.
    ///
    /// On load:
    /// - Generates internal IDs for spells (1, 2, 3...)
    /// - Resolves spell names to IDs in spellbooks
    ///
    /// Validation checks:
    /// - All spell references in spellbooks exist
    /// - No duplicate spell or spellbook names
    /// - No empty names
    pub fn load(path: &str) -> Result<Codex, Box<dyn std::error::Error>> {
        log_info!("Loading codex from: {}", path);
        let contents = fs::read_to_string(path)?;
        log_debug!("Loaded {} bytes from codex", contents.len());

        // First pass: load (IDs default to 0)
        let mut codex: Codex = toml::from_str(&contents)?;

        // Generate IDs for spells (1, 2, 3, ...)
        for (i, spell) in codex.spells.iter_mut().enumerate() {
            spell.id = (i + 1) as u64;
        }

        // Resolve spell names to IDs in spellbooks
        for spellbook in &mut codex.spellbooks {
            let resolved_ids: Vec<u64> = spellbook
                .spells
                .iter()
                .filter_map(|name| codex.spells.iter().find(|s| &s.name == name).map(|s| s.id))
                .collect();
            spellbook.spell_ids = resolved_ids;
        }

        // Validate the loaded data
        validate_codex(&codex)?;

        log_info!(
            "Loaded {} spells and {} spellbooks",
            codex.spells.len(),
            codex.spellbooks.len()
        );
        Ok(codex)
    }

    /// Loads the selected theme from config file.
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

    /// Saves the selected theme to config file.
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
        fs::write(path, new_content)?;
        Ok(())
    }

    /// Loads user settings (view mode, etc.) from config file.
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

    /// Saves user settings to config file.
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
        fs::write(path, new_content)?;
        Ok(())
    }

    /// Appends a spell to the codex file.
    pub fn append_spell(
        path: &str,
        spell: &crate::models::Spell,
        spellbook: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Saving spell: {} to codex", spell.name);

        // Read existing content
        let mut contents = fs::read_to_string(path)?;

        // Append the new spell
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

        // If spellbook is specified, add the spell to that spellbook
        if let Some(book_name) = spellbook {
            let spellbook_section = format!("[[spellbooks]]\nname = \"{}\"", book_name);
            if let Some(pos) = contents.find(&spellbook_section) {
                // Find the end of this spellbook section
                let rest = &contents[pos..];

                // Find next spellbook or end of file
                let insert_pos = if let Some(next_pos) = rest[2..].find("[[spellbooks]]") {
                    pos + 2 + next_pos
                } else {
                    // Last spellbook - insert at end of file
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

    /// Appends a new spellbook to the codex file.
    pub fn append_spellbook(
        path: &str,
        name: &str,
        cover: Option<&str>,
        sigil: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_info!("Creating spellbook: {}", name);

        // Read existing content
        let contents = fs::read_to_string(path)?;

        // Build the new spellbook section
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

        // Append to file
        fs::write(path, &contents)?;
        std::io::Write::write_all(
            &mut std::fs::OpenOptions::new().append(true).open(path)?,
            new_spellbook.as_bytes(),
        )?;

        log_info!("Spellbook '{}' created successfully", name);
        Ok(())
    }

    /// Updates a spell's background preference in the codex file.
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
        fs::write(path, new_contents)?;

        log_info!("Spell '{}' updated successfully", spell_name);
        Ok(())
    }
}
