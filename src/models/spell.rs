use serde::{Deserialize, Serialize};

pub type SpellId = String;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Spell {
    #[serde(default)]
    pub id: SpellId,
    pub name: String,
    #[serde(alias = "incantation")]
    pub command: String,
    #[serde(default)]
    #[serde(alias = "lore")]
    pub description: String,
    #[serde(default)]
    #[serde(alias = "school")]
    pub category: String,
    #[serde(default)]
    #[serde(alias = "glyphs")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub confirm: bool,
    #[serde(default)]
    pub run_mode: RunMode,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default)]
    pub favorite: bool,
}

impl Spell {
    pub fn requires_confirmation(&self) -> bool {
        self.confirm
    }

    pub fn generate_id(&mut self) {
        if self.id.is_empty() {
            self.id = uuid::Uuid::new_v4().to_string();
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    #[default]
    Simple,
    Tui,
    Background,
}

impl RunMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "simple" => RunMode::Simple,
            "tui" => RunMode::Tui,
            "background" => RunMode::Background,
            _ => RunMode::Simple,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RunMode::Simple => "simple",
            RunMode::Tui => "tui",
            RunMode::Background => "background",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === confirmation ===

    #[test]
    fn requires_confirmation_false_when_flag_clear() {
        let spell = Spell {
            name: "Minor Illusion".into(),
            confirm: false,
            ..Default::default()
        };
        assert!(!spell.requires_confirmation());
    }

    #[test]
    fn requires_confirmation_true_when_confirm_flag_set() {
        let spell = Spell {
            name: "Any Spell".into(),
            confirm: true,
            ..Default::default()
        };
        assert!(spell.requires_confirmation());
    }

    // === defaults & generation ===

    #[test]
    fn spell_default_values() {
        let spell = Spell::default();
        assert!(spell.id.is_empty());
        assert!(spell.name.is_empty());
        assert!(spell.command.is_empty());
        assert!(spell.description.is_empty());
        assert!(spell.category.is_empty());
        assert!(spell.tags.is_empty());
        assert!(!spell.confirm);
        assert!(spell.working_dir.is_empty());
        assert!(!spell.favorite);
        assert_eq!(spell.run_mode, RunMode::Simple);
    }

    #[test]
    fn run_mode_from_str() {
        assert_eq!(RunMode::from_str("simple"), RunMode::Simple);
        assert_eq!(RunMode::from_str("tui"), RunMode::Tui);
        assert_eq!(RunMode::from_str("background"), RunMode::Background);
        assert_eq!(RunMode::from_str("unknown"), RunMode::Simple);
    }

    #[test]
    fn run_mode_as_str() {
        assert_eq!(RunMode::Simple.as_str(), "simple");
        assert_eq!(RunMode::Tui.as_str(), "tui");
        assert_eq!(RunMode::Background.as_str(), "background");
    }

    #[test]
    fn spell_generate_id() {
        let mut spell = Spell::default();
        assert!(spell.id.is_empty());
        spell.generate_id();
        assert!(!spell.id.is_empty());
    }
}
