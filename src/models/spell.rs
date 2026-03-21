use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Spell {
    /// Internal ID generated on load (not in the file)
    #[serde(default)]
    pub id: u64,
    pub name: String,
    pub incantation: String,
    pub lore: String,
    pub school: String,
    pub glyphs: Vec<String>,
    #[serde(default)]
    pub elevated: bool,
    #[serde(default)]
    pub dangerous: bool,
    #[serde(default)]
    pub confirm: bool,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default)]
    pub background: bool,
}

impl Spell {
    pub fn requires_confirmation(&self) -> bool {
        self.confirm || self.elevated || self.dangerous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_confirmation_false_when_all_flags_clear() {
        let spell = Spell {
            name: "Minor Illusion".into(),
            confirm: false,
            elevated: false,
            dangerous: false,
            ..Default::default()
        };
        assert!(!spell.requires_confirmation());
    }

    #[test]
    fn test_requires_confirmation_true_when_confirm_flag_set() {
        let spell = Spell {
            name: "Any Spell".into(),
            confirm: true,
            elevated: false,
            dangerous: false,
            ..Default::default()
        };
        assert!(spell.requires_confirmation());
    }

    #[test]
    fn test_requires_confirmation_true_when_elevated() {
        let spell = Spell {
            name: "System Command".into(),
            elevated: true,
            ..Default::default()
        };
        assert!(spell.requires_confirmation());
    }

    #[test]
    fn test_requires_confirmation_true_when_dangerous() {
        let spell = Spell {
            name: "Extinction".into(),
            dangerous: true,
            ..Default::default()
        };
        assert!(spell.requires_confirmation());
    }

    #[test]
    fn test_requires_confirmation_true_when_multiple_flags_set() {
        let spell = Spell {
            name: "Dangerous Elevated".into(),
            elevated: true,
            dangerous: true,
            ..Default::default()
        };
        assert!(spell.requires_confirmation());
    }

    #[test]
    fn test_spell_default_values() {
        let spell = Spell::default();
        assert_eq!(spell.id, 0);
        assert!(spell.name.is_empty());
        assert!(spell.incantation.is_empty());
        assert!(spell.lore.is_empty());
        assert!(spell.school.is_empty());
        assert!(spell.glyphs.is_empty());
        assert!(!spell.elevated);
        assert!(!spell.dangerous);
        assert!(!spell.confirm);
        assert!(spell.working_dir.is_empty());
    }
}
