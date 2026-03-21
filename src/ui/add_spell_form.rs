#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddSpellField {
    Name,
    Command,
    Lore,
    School,
    Tags,
    Spellbook,
}

impl Default for AddSpellField {
    fn default() -> Self {
        Self::Name
    }
}

#[derive(Debug, Clone)]
pub struct AddSpellForm {
    pub field: AddSpellField,
    pub name: String,
    pub command: String,
    pub lore: String,
    pub school: String,
    pub tags: String,
    pub spellbook_index: Option<usize>,
    pub skip_spellbook: bool,
    pub dropdown_index: usize,
    pub dropdown_open: bool,
    pub message: Option<(String, bool)>,
    pub has_unsaved: bool,
}

impl Default for AddSpellForm {
    fn default() -> Self {
        Self {
            field: AddSpellField::default(),
            name: String::new(),
            command: String::new(),
            lore: String::new(),
            school: String::new(),
            tags: String::new(),
            spellbook_index: None,
            skip_spellbook: false,
            dropdown_index: 0,
            dropdown_open: false,
            message: None,
            has_unsaved: false,
        }
    }
}

impl AddSpellForm {
    pub fn clear(&mut self) {
        self.name.clear();
        self.command.clear();
        self.lore.clear();
        self.school.clear();
        self.tags.clear();
        self.spellbook_index = None;
        self.skip_spellbook = false;
        self.dropdown_open = false;
        self.message = None;
        self.has_unsaved = false;
        self.field = AddSpellField::default();
    }

    pub fn is_typing(&self) -> bool {
        matches!(
            self.field,
            AddSpellField::Name
                | AddSpellField::Command
                | AddSpellField::Lore
                | AddSpellField::School
                | AddSpellField::Tags
        )
    }
}
