use super::{Spell, Spellbook};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Codex {
    pub spells: Vec<Spell>,
    pub spellbooks: Vec<Spellbook>,
}
