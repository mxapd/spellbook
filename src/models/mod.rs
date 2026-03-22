mod codex;
mod job;
mod spell;
mod spellbook;
mod theme;

pub use codex::Codex;
pub use job::{Job, JobId, JobManager, JobStatus, JobsData, RecentAction, RecentEntry};
pub use spell::{RunMode, Spell, SpellId};
pub use spellbook::{Spellbook, SpineStyle};
pub use spellbook_ref::{FocusTarget, SpellbookRef, VirtualKind};
pub use theme::{RatatuiColors, Theme, ThemeConfig, UserSettings, ViewMode};

mod spellbook_ref {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum VirtualKind {
        Favorites,
        Recent,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum SpellbookRef {
        Virtual(VirtualKind),
        Codex(usize),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum FocusTarget {
        Main,
        JobsSidebar,
    }
}
