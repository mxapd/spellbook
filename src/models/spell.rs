// spell.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Spell {
    pub id: u64,
    pub name: String,
    pub incantation: String, // actual command/commands
    pub lore: String,        // description
    pub school: String,      // category
    pub glyphs: Vec<String>, // search terms to help search
}
