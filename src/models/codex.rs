use super::{Spell, Spellbook};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Codex {
    pub spells: Vec<Spell>,
    pub spellbooks: Vec<Spellbook>,
}

impl Default for Codex {
    fn default() -> Self {
        Self {
            spells: Vec::new(),
            spellbooks: Vec::new(),
        }
    }
}
