use crate::models::{Codex, RecentAction, Spell, Spellbook};

pub fn make_spell(name: &str) -> Spell {
    Spell {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        incantation: format!("echo 'Running {}'", name),
        lore: format!("Test lore for {}", name),
        school: "Testing".to_string(),
        glyphs: vec![],
        confirm: false,
        run_mode: crate::models::RunMode::Simple,
        working_dir: String::new(),
        favorite: false,
    }
}

pub fn make_spell_with_id(name: &str, id: &str) -> Spell {
    Spell {
        id: id.to_string(),
        name: name.to_string(),
        incantation: format!("echo 'Running {}'", name),
        lore: format!("Test lore for {}", name),
        school: "Testing".to_string(),
        glyphs: vec![],
        confirm: false,
        run_mode: crate::models::RunMode::Simple,
        working_dir: String::new(),
        favorite: false,
    }
}

pub fn make_favorite_spell(name: &str) -> Spell {
    Spell {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        incantation: format!("echo 'Running {}'", name),
        lore: String::new(),
        school: String::new(),
        glyphs: vec![],
        confirm: false,
        run_mode: crate::models::RunMode::Simple,
        working_dir: String::new(),
        favorite: true,
    }
}

pub fn make_spellbook(name: &str, spell_ids: Vec<String>) -> Spellbook {
    Spellbook {
        name: name.to_string(),
        cover: format!("{} cover", name),
        sigil: "*".to_string(),
        spell_ids,
        spells: vec![],
        style: None,
        color: None,
    }
}

pub fn make_codex() -> Codex {
    Codex {
        spells: vec![],
        spellbooks: vec![],
    }
}

pub fn make_codex_with_spells(spells: Vec<Spell>) -> Codex {
    Codex {
        spells,
        spellbooks: vec![],
    }
}

pub fn make_recent_entry(
    spell_id: &str,
    spell_name: &str,
    action: RecentAction,
) -> crate::models::RecentEntry {
    crate::models::RecentEntry::new(spell_id.to_string(), spell_name.to_string(), action)
}
