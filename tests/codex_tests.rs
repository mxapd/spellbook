use spellbook::models::{Codex, RunMode, Spell, Spellbook};

fn spell(id: &str, name: &str) -> Spell {
    Spell {
        id: id.to_string(),
        name: name.to_string(),
        incantation: String::new(),
        lore: String::new(),
        school: String::new(),
        glyphs: Vec::new(),
        confirm: false,
        run_mode: RunMode::Simple,
        working_dir: String::new(),
        favorite: false,
    }
}

fn book(name: &str, ids: &[&str]) -> Spellbook {
    Spellbook {
        name: name.to_string(),
        cover: String::new(),
        sigil: String::new(),
        color: None,
        style: None,
        spell_ids: ids.iter().map(|s| s.to_string()).collect(),
        spells: Vec::new(),
    }
}

#[test]
fn unassigned_count_excludes_assigned_spells() {
    let codex = Codex {
        spells: vec![spell("a", "A"), spell("b", "B"), spell("c", "C")],
        spellbooks: vec![book("main", &["a", "b"])],
    };
    assert_eq!(codex.unassigned_count(), 1);
    assert_eq!(codex.unassigned_spells().len(), 1);
    assert_eq!(codex.unassigned_spells()[0].id, "c");
}

#[test]
fn unassigned_count_is_zero_when_all_assigned() {
    let codex = Codex {
        spells: vec![spell("a", "A")],
        spellbooks: vec![book("main", &["a"])],
    };
    assert_eq!(codex.unassigned_count(), 0);
}

#[test]
fn unassigned_count_is_all_when_no_spellbooks() {
    let codex = Codex {
        spells: vec![spell("a", "A"), spell("b", "B")],
        spellbooks: vec![],
    };
    assert_eq!(codex.unassigned_count(), 2);
}

#[test]
fn unassigned_spells_are_empty_when_spell_id_in_multiple_books() {
    // A spell assigned to multiple spellbooks should still not be "unassigned".
    let codex = Codex {
        spells: vec![spell("a", "A")],
        spellbooks: vec![book("b1", &["a"]), book("b2", &["a"])],
    };
    assert_eq!(codex.unassigned_count(), 0);
}
