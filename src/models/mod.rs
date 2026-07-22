mod codex;
mod job;
mod spell;
mod spellbook;
mod theme;

pub use codex::Codex;
pub use job::{RecentAction, RecentEntry};
pub use spell::{RunMode, Spell};
pub use spellbook::{Spellbook, SpineStyle};
pub use theme::{RatatuiColors, ThemeConfig, UserSettings, ViewMode};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FocusTarget {
    Main,
    JobsSidebar,
}
