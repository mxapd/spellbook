use crate::invoker::Placeholder;
use crate::models::Spell;

#[derive(PartialEq, Clone, Copy)]
pub enum InputPhase {
    Arguments,
}

pub struct InputPopupState {
    pub phase: InputPhase,
    pub command: String,
    pub spell: Option<Spell>,
    pub resolved_command: String,
    pub placeholders: Vec<PlaceholderInput>,
    pub placeholder_index: usize,
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
            placeholder_index: 0,
            run_background,
        }
    }

    pub fn substitute(&self) -> String {
        let values: Vec<(String, String)> = self
            .placeholders
            .iter()
            .map(|p| (p.placeholder.name.clone(), p.value.clone()))
            .collect();
        crate::invoker::substitute_placeholders(&self.command, &values)
    }

    pub fn validate(&self) -> bool {
        self.placeholders.iter().all(|p| !p.value.is_empty())
    }
}
