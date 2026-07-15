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
    pub is_editing: bool,
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
            is_editing: false,
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
        self.is_editing = false;
        self.message = None;
        self.has_unsaved = false;
        self.editing_spell_id = None;
        self.field = AddSpellField::default();
    }

    pub fn is_editing(&self) -> bool {
        self.editing_spell_id.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.command.trim().is_empty()
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
        self.skip_spellbook = spellbook_index.is_none();
        // Dropdown index 0 is "None (unassigned)"; real spellbooks start at 1.
        self.dropdown_index = match spellbook_index {
            Some(idx) => idx + 1,
            None => 0,
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RunMode;

    #[test]
    fn test_add_spell_field_default() {
        assert_eq!(AddSpellField::default(), AddSpellField::Name);
    }

    #[test]
    fn test_add_spell_form_default() {
        let form = AddSpellForm::default();
        assert_eq!(form.field, AddSpellField::Name);
        assert!(form.name.is_empty());
        assert!(form.command.is_empty());
        assert!(form.lore.is_empty());
        assert!(form.school.is_empty());
        assert!(form.tags.is_empty());
        assert_eq!(form.run_mode, RunMode::Simple);
        assert!(!form.confirm);
        assert!(form.working_dir.is_empty());
        assert!(form.spellbook_index.is_none());
        assert!(!form.skip_spellbook);
        assert!(!form.dropdown_open);
        assert!(form.message.is_none());
        assert!(!form.has_unsaved);
        assert!(form.editing_spell_id.is_none());
    }

    #[test]
    fn test_add_spell_form_clear() {
        let mut form = AddSpellForm {
            field: AddSpellField::RunMode,
            name: "Test".to_string(),
            command: "echo test".to_string(),
            lore: "Lore".to_string(),
            school: "School".to_string(),
            tags: "tag1,tag2".to_string(),
            run_mode: RunMode::Tui,
            confirm: true,
            working_dir: "/tmp".to_string(),
            spellbook_index: Some(1),
            skip_spellbook: true,
            dropdown_index: 5,
            dropdown_open: true,
            message: Some(("Error".to_string(), true)),
            has_unsaved: true,
            editing_spell_id: Some("uuid".to_string()),
            is_editing: true,
        };

        form.clear();

        assert_eq!(form.field, AddSpellField::Name);
        assert!(form.name.is_empty());
        assert!(form.command.is_empty());
        assert!(form.lore.is_empty());
        assert!(form.school.is_empty());
        assert!(form.tags.is_empty());
        assert_eq!(form.run_mode, RunMode::Simple);
        assert!(!form.confirm);
        assert!(form.working_dir.is_empty());
        assert!(form.spellbook_index.is_none());
        assert!(!form.skip_spellbook);
        assert!(!form.dropdown_open);
        assert!(form.message.is_none());
        assert!(!form.has_unsaved);
        assert!(form.editing_spell_id.is_none());
    }

    #[test]
    fn test_add_spell_form_is_valid_empty_name() {
        let form = AddSpellForm::default();
        assert!(!form.is_valid());
    }

    #[test]
    fn test_add_spell_form_is_valid_with_name() {
        let form = AddSpellForm {
            name: "Test Spell".to_string(),
            command: "echo test".to_string(),
            ..Default::default()
        };
        assert!(form.is_valid());
    }

    #[test]
    fn test_add_spell_form_is_valid_empty_command() {
        let form = AddSpellForm {
            name: "Test Spell".to_string(),
            command: "   ".to_string(),
            ..Default::default()
        };
        assert!(!form.is_valid());
    }

    #[test]
    fn test_add_spell_form_is_valid_whitespace_name() {
        let form = AddSpellForm {
            name: "   ".to_string(),
            command: "echo test".to_string(),
            ..Default::default()
        };
        assert!(!form.is_valid());
    }

    #[test]
    fn test_add_spell_form_is_typing_name_field() {
        let form = AddSpellForm {
            field: AddSpellField::Name,
            ..Default::default()
        };
        assert!(form.is_typing());
    }

    #[test]
    fn test_add_spell_form_is_typing_working_dir_field() {
        let form = AddSpellForm {
            field: AddSpellField::WorkingDir,
            ..Default::default()
        };
        assert!(!form.is_typing());
    }
}
