// spellbook.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Spellbook {
    pub id: u64,
    pub name: String,
    pub cover: String, // description
    pub sigil: String, // for future asci art or something
    pub spell_ids: Vec<u64>,
}
