#[derive(Debug, Clone, Default)]
pub struct AddSpellbookForm {
    pub name: String,
    pub cover: String,
    pub sigil: String,
}

impl AddSpellbookForm {
    pub fn clear(&mut self) {
        self.name.clear();
        self.cover.clear();
        self.sigil.clear();
    }
}
