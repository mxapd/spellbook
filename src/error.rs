use thiserror::Error;

#[derive(Debug, Error)]

pub enum SaveError {
    #[error("failed to write `{path}`: {cause}")]
    Write { path: String, cause: String },

    #[error("failed to serialize: {cause}")]
    Serialize { cause: String },
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("failed to read `{path}`: {cause}")]
    Read { path: String, cause: String },

    #[error("failed to parse `{path}`: {cause}")]
    Parse { path: String, cause: String },

    #[error("failed to validate: {0}")]
    Validation(#[from] ValidationError),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("duplicate spell name `{name}`")]
    DuplicateSpellName { name: String },

    #[error("duplicate spellbook name `{name}`")]
    DuplicateSpellbookName { name: String },

    #[error("spellbook name cannot be empty")]
    EmptySpellbookName,

    #[error("spell name cannot be empty")]
    EmptySpellName,

    #[error("spellbook '{spellbook}' references non-existent spell id: {spell_id}")]
    BrokenReference { spellbook: String, spell_id: String },
}
