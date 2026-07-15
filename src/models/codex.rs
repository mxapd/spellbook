use super::{Spell, Spellbook};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Codex {
    pub spells: Vec<Spell>,
    pub spellbooks: Vec<Spellbook>,
}

impl Codex {
    /// Build a set of all spell IDs referenced by any spellbook.
    fn assigned_spell_ids(&self) -> HashSet<&str> {
        self.spellbooks
            .iter()
            .flat_map(|sb| sb.spell_ids.iter())
            .map(|id| id.as_str())
            .collect()
    }

    /// Return spells that are not in any spellbook.
    pub fn unassigned_spells(&self) -> Vec<&Spell> {
        let assigned = self.assigned_spell_ids();
        self.spells
            .iter()
            .filter(|s| !assigned.contains(s.id.as_str()))
            .collect()
    }

    /// Count spells that are not in any spellbook.
    pub fn unassigned_count(&self) -> usize {
        self.unassigned_spells().len()
    }
}

impl Default for Codex {
    fn default() -> Self {
        Self {
            spells: Vec::new(),
            spellbooks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Spell;
    use crate::models::Spellbook;

    fn spell(id: &str, name: &str) -> Spell {
        Spell {
            id: id.to_string(),
            name: name.to_string(),
            incantation: String::new(),
            lore: String::new(),
            school: String::new(),
            glyphs: Vec::new(),
            confirm: false,
            run_mode: crate::models::RunMode::Simple,
            working_dir: String::new(),
            favorite: false,
        }
    }

    fn book(name: &str, ids: &[&str]) -> Spellbook {
        Spellbook {
            name: name.to_string(),
            cover: String::new(),
            sigil: String::new(),
            color: None,
            style: None,
            spell_ids: ids.iter().map(|s| s.to_string()).collect(),
            spells: Vec::new(),
        }
    }

    #[test]
    fn unassigned_count_excludes_assigned_spells() {
        let codex = Codex {
            spells: vec![spell("a", "A"), spell("b", "B"), spell("c", "C")],
            spellbooks: vec![book("main", &["a", "b"])],
        };
        assert_eq!(codex.unassigned_count(), 1);
        assert_eq!(codex.unassigned_spells().len(), 1);
        assert_eq!(codex.unassigned_spells()[0].id, "c");
    }

    #[test]
    fn unassigned_count_is_zero_when_all_assigned() {
        let codex = Codex {
            spells: vec![spell("a", "A")],
            spellbooks: vec![book("main", &["a"])],
        };
        assert_eq!(codex.unassigned_count(), 0);
    }

    #[test]
    fn unassigned_count_is_all_when_no_spellbooks() {
        let codex = Codex {
            spells: vec![spell("a", "A"), spell("b", "B")],
            spellbooks: vec![],
        };
        assert_eq!(codex.unassigned_count(), 2);
    }
}
