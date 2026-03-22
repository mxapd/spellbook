#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddSpellField {
    Name,
    Command,
    Lore,
    School,
    Tags,
    WorkingDir,
    RunMode,
    Confirm,
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
    pub run_mode: crate::models::RunMode,
    pub confirm: bool,
    pub working_dir: String,
    pub spellbook_index: Option<usize>,
    pub skip_spellbook: bool,
    pub dropdown_index: usize,
    pub dropdown_open: bool,
    pub message: Option<(String, bool)>,
    pub has_unsaved: bool,
    pub editing_spell_id: Option<String>,
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
            run_mode: crate::models::RunMode::Simple,
            confirm: false,
            working_dir: String::new(),
            spellbook_index: None,
            skip_spellbook: false,
            dropdown_index: 0,
            dropdown_open: false,
            message: None,
            has_unsaved: false,
            editing_spell_id: None,
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
        self.run_mode = crate::models::RunMode::Simple;
        self.confirm = false;
        self.working_dir.clear();
        self.spellbook_index = None;
        self.skip_spellbook = false;
        self.dropdown_open = false;
        self.message = None;
        self.has_unsaved = false;
        self.editing_spell_id = None;
        self.field = AddSpellField::default();
    }

    pub fn is_editing(&self) -> bool {
        self.editing_spell_id.is_some()
    }

    pub fn start_edit(&mut self, spell: &crate::models::Spell, spellbook_index: Option<usize>) {
        self.clear();
        self.editing_spell_id = Some(spell.id.clone());
        self.name = spell.name.clone();
        self.command = spell.incantation.clone();
        self.lore = spell.lore.clone();
        self.school = spell.school.clone();
        self.tags = spell.glyphs.join(", ");
        self.run_mode = spell.run_mode;
        self.confirm = spell.confirm;
        self.working_dir = spell.working_dir.clone();
        self.spellbook_index = spellbook_index;
    }

    pub fn is_typing(&self) -> bool {
        matches!(
            self.field,
            AddSpellField::Name
                | AddSpellField::Command
                | AddSpellField::Lore
                | AddSpellField::School
                | AddSpellField::Tags
                | AddSpellField::RunMode
                | AddSpellField::Confirm
                | AddSpellField::Spellbook
        )
    }
}
