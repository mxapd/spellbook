use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Spellbook {
    pub name: String,
    pub cover: String,
    pub sigil: String,
    /// References spells by ID (populated after load)
    #[serde(default)]
    pub spell_ids: Vec<u64>,
    /// References spells by name (from TOML file)
    #[serde(default)]
    pub spells: Vec<String>,
}
