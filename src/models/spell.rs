use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Spell {
    /// Internal ID generated on load (not in the file)
    #[serde(default)]
    pub id: u64,
    pub name: String,
    pub incantation: String,
    pub lore: String,
    pub school: String,
    pub glyphs: Vec<String>,
}
