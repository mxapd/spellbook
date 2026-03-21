use crate::executor::Placeholder;
use crate::models::Spell;

#[derive(PartialEq, Clone, Copy)]
pub enum InputPhase {
    Password,
    Arguments,
}

pub struct InputPopupState {
    pub phase: InputPhase,
    pub command: String,
    pub spell: Option<Spell>,
    pub resolved_command: String,
    pub placeholders: Vec<PlaceholderInput>,
    pub password: String,
    pub show_password: bool,
    pub run_background: bool,
}

pub struct PlaceholderInput {
    pub placeholder: Placeholder,
    pub value: String,
}

impl InputPopupState {
    pub fn new(
        spell: Spell,
        placeholders: Vec<Placeholder>,
        command: String,
        run_background: bool,
    ) -> Self {
        let placeholder_inputs: Vec<PlaceholderInput> = placeholders
            .into_iter()
            .map(|p| PlaceholderInput {
                placeholder: p,
                value: String::new(),
            })
            .collect();

        Self {
            phase: InputPhase::Arguments,
            command: command.clone(),
            spell: Some(spell),
            resolved_command: command,
            placeholders: placeholder_inputs,
            password: String::new(),
            show_password: false,
            run_background,
        }
    }

    pub fn with_password(
        spell: Spell,
        placeholders: Vec<Placeholder>,
        command: String,
        run_background: bool,
    ) -> Self {
        let mut state = Self::new(spell, placeholders, command, run_background);
        state.phase = InputPhase::Password;
        state
    }

    pub fn substitute(&self) -> String {
        let values: Vec<(String, String)> = self
            .placeholders
            .iter()
            .map(|p| (p.placeholder.name.clone(), p.value.clone()))
            .collect();
        crate::executor::substitute_placeholders(&self.command, &values)
    }

    pub fn validate(&self) -> bool {
        match self.phase {
            InputPhase::Password => !self.password.is_empty(),
            InputPhase::Arguments => self.placeholders.iter().all(|p| !p.value.is_empty()),
        }
    }
}
