// spell.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Spell {
    id: u64,
    name: String,
    incantation: String, // actual command/commands
    lore: String,        // description
    school: String,      // category
    glyphs: Vec<String>, // search terms to help search
}
