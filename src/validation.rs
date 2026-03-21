use crate::models::Codex;
use std::collections::HashSet;

pub fn validate_codex(codex: &Codex) -> Result<(), Box<dyn std::error::Error>> {
    let spell_names: HashSet<&String> = codex.spells.iter().map(|s| &s.name).collect();

    let mut seen_spell_names: HashSet<&String> = HashSet::new();
    for spell in &codex.spells {
        if seen_spell_names.contains(&spell.name) {
            return Err(format!("Duplicate spell name: {}", spell.name).into());
        }
        seen_spell_names.insert(&spell.name);

        if spell.name.trim().is_empty() {
            return Err("Spell name cannot be empty".into());
        }
    }

    let mut seen_spellbook_names: HashSet<&String> = HashSet::new();
    for spellbook in &codex.spellbooks {
        if seen_spellbook_names.contains(&spellbook.name) {
            return Err(format!("Duplicate spellbook name: {}", spellbook.name).into());
        }
        seen_spellbook_names.insert(&spellbook.name);

        if spellbook.name.trim().is_empty() {
            return Err("Spellbook name cannot be empty".into());
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Codex, Spell, Spellbook};

    fn make_spell(name: &str) -> Spell {
        Spell {
            id: 0,
            name: name.to_string(),
            incantation: "test command".to_string(),
            lore: "Test lore".to_string(),
            school: "Test".to_string(),
            glyphs: vec![],
            elevated: false,
            dangerous: false,
            confirm: false,
            working_dir: String::new(),
            background: false,
        }
    }

    fn make_spellbook(name: &str, spells: Vec<&str>) -> Spellbook {
        Spellbook {
            name: name.to_string(),
            cover: "Test cover".to_string(),
            sigil: "*".to_string(),
            spell_ids: vec![],
            spells: spells.into_iter().map(String::from).collect(),
            style: None,
        }
    }

    #[test]
    fn test_valid_empty_codex() {
        let codex = Codex {
            spells: vec![],
            spellbooks: vec![],
        };
        assert!(validate_codex(&codex).is_ok());
    }

    #[test]
    fn test_valid_single_spell() {
        let codex = Codex {
            spells: vec![make_spell("Fireball")],
            spellbooks: vec![],
        };
        assert!(validate_codex(&codex).is_ok());
    }

    #[test]
    fn test_valid_spell_and_spellbook() {
        let codex = Codex {
            spells: vec![make_spell("Fireball"), make_spell("Ice Bolt")],
            spellbooks: vec![make_spellbook("Fire Magic", vec!["Fireball"])],
        };
        assert!(validate_codex(&codex).is_ok());
    }

    #[test]
    fn test_valid_multiple_spellbooks() {
        let codex = Codex {
            spells: vec![make_spell("Fireball"), make_spell("Heal")],
            spellbooks: vec![
                make_spellbook("Fire Magic", vec!["Fireball"]),
                make_spellbook("Healing Arts", vec!["Heal"]),
            ],
        };
        assert!(validate_codex(&codex).is_ok());
    }

    #[test]
    fn test_valid_spellbook_referencing_multiple_spells() {
        let codex = Codex {
            spells: vec![
                make_spell("Fireball"),
                make_spell("Frostbite"),
                make_spell("Lightning"),
            ],
            spellbooks: vec![make_spellbook(
                "Elemental Magic",
                vec!["Fireball", "Frostbite", "Lightning"],
            )],
        };
        assert!(validate_codex(&codex).is_ok());
    }

    #[test]
    fn test_duplicate_spell_names_rejected() {
        let codex = Codex {
            spells: vec![make_spell("Fireball"), make_spell("Fireball")],
            spellbooks: vec![],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Duplicate spell name"));
        assert!(err.contains("Fireball"));
    }

    #[test]
    fn test_empty_spell_name_rejected() {
        let codex = Codex {
            spells: vec![make_spell("")],
            spellbooks: vec![],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Spell name cannot be empty");
    }

    #[test]
    fn test_whitespace_only_spell_name_rejected() {
        let codex = Codex {
            spells: vec![make_spell("   ")],
            spellbooks: vec![],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Spell name cannot be empty");
    }

    #[test]
    fn test_duplicate_spellbook_names_rejected() {
        let codex = Codex {
            spells: vec![make_spell("Fireball")],
            spellbooks: vec![
                make_spellbook("Wizardry", vec!["Fireball"]),
                make_spellbook("Wizardry", vec!["Fireball"]),
            ],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Duplicate spellbook name"));
        assert!(err.contains("Wizardry"));
    }

    #[test]
    fn test_empty_spellbook_name_rejected() {
        let codex = Codex {
            spells: vec![make_spell("Fireball")],
            spellbooks: vec![make_spellbook("", vec!["Fireball"])],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Spellbook name cannot be empty");
    }

    #[test]
    fn test_spellbook_referencing_nonexistent_spell_rejected() {
        let codex = Codex {
            spells: vec![make_spell("Fireball")],
            spellbooks: vec![make_spellbook("Necromancy", vec!["Raise Dead"])],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("references non-existent spell"));
        assert!(err.contains("Raise Dead"));
        assert!(err.contains("Necromancy"));
    }

    #[test]
    fn test_spellbook_referencing_nonexistent_spell_in_multiple_rejected() {
        let codex = Codex {
            spells: vec![make_spell("Fireball"), make_spell("Ice Bolt")],
            spellbooks: vec![make_spellbook(
                "Magic",
                vec!["Fireball", "Missing Spell", "Ice Bolt"],
            )],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("references non-existent spell"));
        assert!(err.contains("Missing Spell"));
    }

    #[test]
    fn test_multiple_validation_errors_returns_first() {
        let codex = Codex {
            spells: vec![make_spell("Fireball"), make_spell("Fireball"), make_spell("")],
            spellbooks: vec![],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
    }
}
