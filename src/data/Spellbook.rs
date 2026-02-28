pub struct Spellbook {
    id: u64,
    name: String,
    cover: String, // description
    sigil: String, // for future asci art or something
    spells: Vec<Spell>,
}
