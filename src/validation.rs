use crate::error::ValidationError;
use crate::models::Codex;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

pub fn validate_codex(codex: &Codex) -> Result<(), ValidationError> {
    let spell_ids: HashSet<&String> = codex.spells.iter().map(|s| &s.id).collect();

    // validate spells
    let mut seen_spell_names: HashSet<&String> = HashSet::new();
    for spell in &codex.spells {
        if seen_spell_names.contains(&spell.name) {
            return Err(ValidationError::DuplicateSpellName {
                name: spell.name.clone(),
            });
        }
        seen_spell_names.insert(&spell.name);
        if spell.name.trim().is_empty() {
            return Err(ValidationError::EmptySpellName);
        }
    }

    // validate spellbooks
    let mut seen_spellbook_names: HashSet<&String> = HashSet::new();
    for spellbook in &codex.spellbooks {
        if seen_spellbook_names.contains(&spellbook.name) {
            return Err(ValidationError::DuplicateSpellbookName {
                name: spellbook.name.clone(),
            });
        }
        seen_spellbook_names.insert(&spellbook.name);
        if spellbook.name.trim().is_empty() {
            return Err(ValidationError::EmptySpellbookName);
        }

        for spell_id in &spellbook.spell_ids {
            if !spell_ids.contains(spell_id) {
                return Err(ValidationError::BrokenReference {
                    spellbook: spellbook.name.clone(),
                    spell: spell_id.clone(),
                });
            }
        }
    }

    Ok(())
}

pub fn validate_codex_warnings(codex: &Codex) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();
    let spell_ids: HashSet<&String> = codex.spells.iter().map(|s| &s.id).collect();

    // Check for duplicate spell names (non-blocking, report as error)
    let mut seen_spell_names: HashSet<&String> = HashSet::new();
    for spell in &codex.spells {
        if seen_spell_names.contains(&spell.name) {
            warnings.push(ValidationWarning {
                message: format!("Duplicate spell name: {}", spell.name),
                severity: WarningSeverity::Error,
            });
        }
        seen_spell_names.insert(&spell.name);

        if spell.name.trim().is_empty() {
            warnings.push(ValidationWarning {
                message: "Spell has empty name".to_string(),
                severity: WarningSeverity::Error,
            });
        }

        if spell.incantation.trim().is_empty() {
            warnings.push(ValidationWarning {
                message: format!("Spell '{}' has empty incantation", spell.name),
                severity: WarningSeverity::Warning,
            });
        }
    }

    // Check for duplicate spellbook names
    let mut seen_spellbook_names: HashSet<&String> = HashSet::new();
    for spellbook in &codex.spellbooks {
        if seen_spellbook_names.contains(&spellbook.name) {
            warnings.push(ValidationWarning {
                message: format!("Duplicate spellbook name: {}", spellbook.name),
                severity: WarningSeverity::Error,
            });
        }
        seen_spellbook_names.insert(&spellbook.name);

        // Check for empty spellbook
        if spellbook.spell_ids.is_empty() {
            warnings.push(ValidationWarning {
                message: format!("Spellbook '{}' has no spells", spellbook.name),
                severity: WarningSeverity::Info,
            });
        }

        // Check for broken references
        for spell_id in &spellbook.spell_ids {
            if !spell_ids.contains(spell_id) {
                warnings.push(ValidationWarning {
                    message: format!(
                        "Spellbook '{}' references non-existent spell: {}",
                        spellbook.name, spell_id
                    ),
                    severity: WarningSeverity::Error,
                });
            }
        }
    }

    // Check for duplicate spell IDs (shouldn't happen after migration)
    let mut seen_ids: HashSet<&String> = HashSet::new();
    for spell in &codex.spells {
        if seen_ids.contains(&spell.id) {
            warnings.push(ValidationWarning {
                message: format!("Duplicate spell ID: {}", spell.id),
                severity: WarningSeverity::Error,
            });
        }
        seen_ids.insert(&spell.id);
    }

    //    // Check for spells not in any spellbook (orphan spells)
    //    let spells_in_books: HashSet<&String> = codex
    //        .spellbooks
    //        .iter()
    //        .flat_map(|sb| sb.spell_ids.iter())
    //        .collect();
    //    for spell in &codex.spells {
    //        if !spells_in_books.contains(&spell.id) {
    //            warnings.push(ValidationWarning {
    //                message: format!("Spell '{}' is not in any spellbook", spell.name),
    //                severity: WarningSeverity::Info,
    //            });
    //        }
    //    }
    //
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Codex, Spell, Spellbook};

    fn make_spell(name: &str) -> Spell {
        Spell {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            incantation: "test command".to_string(),
            lore: "Test lore".to_string(),
            school: "Test".to_string(),
            glyphs: vec![],
            confirm: false,
            run_mode: crate::models::RunMode::Simple,
            working_dir: String::new(),
            favorite: false,
        }
    }

    fn make_spellbook_with_ids(name: &str, spell_ids: Vec<String>) -> Spellbook {
        Spellbook {
            name: name.to_string(),
            cover: "Test cover".to_string(),
            sigil: "*".to_string(),
            spell_ids,
            spells: vec![],
            style: None,
            color: None,
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
        let fireball = make_spell("Fireball");
        let ice_bolt = make_spell("Ice Bolt");
        let codex = Codex {
            spells: vec![fireball.clone(), ice_bolt.clone()],
            spellbooks: vec![make_spellbook_with_ids(
                "Fire Magic",
                vec![fireball.id.clone()],
            )],
        };
        assert!(validate_codex(&codex).is_ok());
    }

    #[test]
    fn test_valid_multiple_spellbooks() {
        let fireball = make_spell("Fireball");
        let heal = make_spell("Heal");
        let codex = Codex {
            spells: vec![fireball.clone(), heal.clone()],
            spellbooks: vec![
                make_spellbook_with_ids("Fire Magic", vec![fireball.id.clone()]),
                make_spellbook_with_ids("Healing Arts", vec![heal.id.clone()]),
            ],
        };
        assert!(validate_codex(&codex).is_ok());
    }

    #[test]
    fn test_valid_spellbook_referencing_multiple_spells() {
        let fireball = make_spell("Fireball");
        let frostbite = make_spell("Frostbite");
        let lightning = make_spell("Lightning");
        let codex = Codex {
            spells: vec![fireball.clone(), frostbite.clone(), lightning.clone()],
            spellbooks: vec![make_spellbook_with_ids(
                "Elemental Magic",
                vec![
                    fireball.id.clone(),
                    frostbite.id.clone(),
                    lightning.id.clone(),
                ],
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
        assert_eq!(
            result.unwrap_err().to_string(),
            "Spell name cannot be empty"
        );
    }

    #[test]
    fn test_whitespace_only_spell_name_rejected() {
        let codex = Codex {
            spells: vec![make_spell("   ")],
            spellbooks: vec![],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Spell name cannot be empty"
        );
    }

    #[test]
    fn test_duplicate_spellbook_names_rejected() {
        let fireball = make_spell("Fireball");
        let codex = Codex {
            spells: vec![fireball.clone()],
            spellbooks: vec![
                make_spellbook_with_ids("Wizardry", vec![fireball.id.clone()]),
                make_spellbook_with_ids("Wizardry", vec![fireball.id.clone()]),
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
        let fireball = make_spell("Fireball");
        let codex = Codex {
            spells: vec![fireball.clone()],
            spellbooks: vec![make_spellbook_with_ids("", vec![fireball.id.clone()])],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Spellbook name cannot be empty"
        );
    }

    #[test]
    fn test_spellbook_referencing_nonexistent_spell_rejected() {
        let codex = Codex {
            spells: vec![make_spell("Fireball")],
            spellbooks: vec![make_spellbook_with_ids(
                "Necromancy",
                vec!["nonexistent-id".to_string()],
            )],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("references non-existent spell id"));
        assert!(err.contains("Necromancy"));
    }

    #[test]
    fn test_multiple_validation_errors_returns_first() {
        let codex = Codex {
            spells: vec![
                make_spell("Fireball"),
                make_spell("Fireball"),
                make_spell(""),
            ],
            spellbooks: vec![],
        };
        let result = validate_codex(&codex);
        assert!(result.is_err());
    }
}
