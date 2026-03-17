# Spellbook Architecture

## Overview

Spellbook is a TUI (Terminal User Interface) application for managing and quickly accessing CLI command snippets. It uses a simple theme where commands are "spells" organized into "spellbooks".

## Technology Stack

- **Language**: Rust
- **TUI Framework**: [ratatui](https://ratatui.rs/)
- **Terminal I/O**: crossterm
- **Serialization**: serde + toml

## Directory Structure

```
src/
├── main.rs           # Entry point, CLI args, event loop
├── state.rs          # Application state (Codex + theme)
├── clipboard.rs      # Clipboard operations with fallback support
├── models/           # Data structures
│   ├── mod.rs
│   ├── codex.rs      # Root data container
│   ├── spell.rs      # Spell/command definition
│   ├── spellbook.rs # Spellbook/collection
│   └── theme.rs     # Theme colors (10 themes)
├── ui/               # Rendering and events
│   ├── mod.rs        # Screen states, UiState, AddSpellField
│   ├── render.rs     # Main render dispatcher
│   ├── events.rs     # Key event handlers
│   ├── spellbook_list.rs
│   ├── spell_list.rs
│   ├── search_overlay.rs
│   └── add_spell.rs  # Add spell form with dropdown
└── persistence/
    ├── mod.rs
    └── archivist.rs  # TOML load/save

codex.toml            # Spell data
theme.toml            # Theme configuration
```

## Data Flow

1. **Load**: `main.rs` calls `Archivist::load("codex.toml")` to deserialize TOML into `Codex`
2. **Theme Load**: Theme is loaded from `theme.toml` with `Archivist::load_theme()` and `load_theme_index()`
3. **ID Generation**: Spells get auto-generated IDs (1, 2, 3...) on load
4. **Resolution**: Spellbook spell references are resolved from names to IDs
5. **State**: `Codex` + theme are wrapped in `State` and passed through the app
6. **UI State**: `UiState` tracks navigation state (selected items, current screen, search query, form fields)
7. **Render Loop**: Each frame, `render()` dispatches to the appropriate screen renderer
8. **Events**: `handle_event()` processes key presses and updates `UiState`
9. **Clipboard**: On copy, notification is sent via `notify-send`
10. **Save**: New spells are appended to `codex.toml` via `Archivist::append_spell()`

## Key Concepts

- **Codex**: The root data structure containing all spells and spellbooks
- **Spell**: A single command with metadata (name, incantation, lore, school, glyphs)
- **Spellbook**: A named collection of spells (referenced by name in TOML, resolved to IDs on load)
- **Screen**: Enum representing the current UI view (SpellbookList, SpellList, SearchOverlay, AddSpell)
- **UiState**: Tracks selection, navigation, search state, and form fields
- **Theme**: Color scheme with bg, fg, accent, muted, selection, border colors

## Theming

The app uses 16-color ANSI themes for terminal compatibility. Theme colors are stored in `RatatuiColors`:

```rust
pub struct RatatuiColors {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub muted: Color,
    pub selection: Color,
    pub border: Color,
}
```

**Available Themes** (cycle with `t` key):
- default (dark)
- default-light
- dracula
- gruvbox-dark
- gruvbox-light
- nord
- catppuccin
- one-dark
- solarized-dark
- solarized-light

Theme preference is persisted in `theme.toml` (`selected_theme` index).

## CLI Arguments

- `--add`: Opens directly to Add Spell screen instead of SpellbookList

## Clipboard

The app uses external clipboard tools for maximum compatibility:

1. `wl-copy` (Wayland)
2. `xclip` (X11)
3. `xsel` (X11)

On successful copy, a D-Bus notification is sent via `notify-send`.
