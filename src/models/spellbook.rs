// spellbook.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Spellbook {
    id: u64,
    name: String,
    cover: String, // description
    sigil: String, // for future asci art or something
    spell_ids: Vec<u64>,
}
