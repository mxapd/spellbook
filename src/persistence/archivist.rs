use crate::models::{Codex, RatatuiColors, ThemeConfig};
use std::collections::HashSet;
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
        let contents = fs::read_to_string(path)?;

        // First pass: load (IDs default to 0)
        let mut codex: Codex = toml::from_str(&contents)?;

        // Generate IDs for spells (1, 2, 3, ...)
        for (i, spell) in codex.spells.iter_mut().enumerate() {
            spell.id = (i + 1) as u64;
        }

        // Build set of valid spell names for validation
        let spell_names: HashSet<&String> = codex.spells.iter().map(|s| &s.name).collect();

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
        validate_codex(&codex, &spell_names)?;

        Ok(codex)
    }

    /// Loads a theme from a TOML file.
    /// Returns default theme if file doesn't exist or fails to parse.
    pub fn load_theme(path: &str) -> RatatuiColors {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return RatatuiColors::default(),
        };

        let config: ThemeConfig = match toml::from_str(&contents) {
            Ok(c) => c,
            Err(_) => return RatatuiColors::default(),
        };

        if let Some(theme) = config.get_theme("default") {
            return theme.to_colors();
        }

        RatatuiColors::default()
    }

    /// Loads the selected theme index from config file.
    pub fn load_theme_index(path: &str) -> usize {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let config: ThemeConfig = match toml::from_str(&contents) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        config.selected_theme
    }

    /// Saves the selected theme index to config file.
    pub fn save_theme_index(path: &str, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                let config = ThemeConfig {
                    selected_theme: index,
                    ..Default::default()
                };
                let new_content = toml::to_string_pretty(&config)?;
                fs::write(path, new_content)?;
                return Ok(());
            }
        };

        let mut config: ThemeConfig = toml::from_str(&contents).unwrap_or_default();
        config.selected_theme = index;

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
            // Find and update the spellbook
            let spellbook_section = format!("[[spellbooks]]\nname = \"{}\"", book_name);
            if let Some(pos) = contents.find(&spellbook_section) {
                // Find the end of this spellbook section (next [[spellbooks]] or end of file)
                let rest = &contents[pos..];
                if let Some(end_pos) = rest[2..].find("[[spellbooks]]") {
                    let insert_pos = pos + 2 + end_pos;
                    contents.insert_str(insert_pos, &format!(", \"{}\"", spell.name));
                }
            }
        }

        fs::write(path, contents)?;
        Ok(())
    }
}

/// Validates a Codex for data integrity issues.
fn validate_codex(
    codex: &Codex,
    spell_names: &HashSet<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check for duplicate spell names
    let mut seen_spell_names: HashSet<&String> = HashSet::new();
    for spell in &codex.spells {
        if seen_spell_names.contains(&spell.name) {
            return Err(format!("Duplicate spell name: {}", spell.name).into());
        }
        seen_spell_names.insert(&spell.name);

        // Check for empty name
        if spell.name.trim().is_empty() {
            return Err("Spell name cannot be empty".into());
        }
    }

    // Check for duplicate spellbook names
    let mut seen_spellbook_names: HashSet<&String> = HashSet::new();
    for spellbook in &codex.spellbooks {
        if seen_spellbook_names.contains(&spellbook.name) {
            return Err(format!("Duplicate spellbook name: {}", spellbook.name).into());
        }
        seen_spellbook_names.insert(&spellbook.name);

        // Check for empty name
        if spellbook.name.trim().is_empty() {
            return Err("Spellbook name cannot be empty".into());
        }

        // Check all spell names reference valid spells
        for spell_name in &spellbook.spells {
            if !spell_names.contains(spell_name) {
                return Err(format!(
                    "Spellbook '{}' references non-existent spell: {}",
                    spellbook.name, spell_name
                )
                .into());
            }
        }
    }

    Ok(())
}
