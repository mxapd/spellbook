# Spellbook Architecture

## Overview

Spellbook is a TUI (Terminal User Interface) application for managing and quickly accessing CLI command snippets. It uses a simple theme where commands are "spells" organized into "spellbooks".

## Technology Stack

- **Language**: Rust
- **TUI Framework**: [ratatui](https://ratatui.rs/)
- **Terminal I/O**: crossterm
- **Serialization**: serde + toml

## Data Storage Conventions

- **TOML is preferred** over JSON for all persistent data
- Only use JSON if required by external tools (e.g., LSP servers)
- Human-editable configs should always use TOML

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
│   ├── spellbook.rs  # Spellbook/collection
│   ├── theme.rs      # Theme colors (10 themes)
│   └── view_mode.rs  # View mode enum (cards/spines)
├── ui/               # Rendering and events
│   ├── mod.rs        # Screen states, UiState, SearchMode, AddSpellField
│   ├── render.rs     # Main render dispatcher
│   ├── events.rs     # Key event handlers
│   ├── spellbook_list.rs
│   ├── spell_list.rs
│   ├── search_overlay.rs  # Primary screen with modes
│   └── add_spell.rs  # Add spell form with dropdown
└── persistence/
    ├── mod.rs
    └── archivist.rs  # TOML load/save, settings persistence

codex.toml            # Spell data
theme.toml            # Theme and view mode configuration
```

## Data Flow

1. **Load**: `main.rs` calls `Archivist::load("codex.toml")` to deserialize TOML into `Codex`
2. **Settings Load**: Theme and view mode loaded from `theme.toml` via `Archivist::load_theme()` and `Archivist::load_user_settings()`
3. **ID Generation**: Spells get auto-generated IDs (1, 2, 3...) on load
4. **Resolution**: Spellbook spell references are resolved from names to IDs
5. **State**: `Codex` + settings wrapped in `State` and passed through the app
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
- **SearchMode**: Mode within SearchOverlay (BrowseSpellbooks, BrowseSpells, AddSpell, AddSpellbook)
- **UiState**: Tracks selection, navigation, search state, and form fields
- **Theme**: Color scheme with bg, fg, accent, muted, selection, border colors
- **ViewMode**: Display mode for spellbook browser (cards/spines)

## Screen Architecture

### SearchOverlay (Primary Screen)

SearchOverlay is the main navigation hub with multiple modes:

1. **BrowseSpellbooks**: Default mode showing spellbooks as cards or spines
2. **BrowseSpells**: Shows spells from selected spellbook
3. **AddSpell**: Form to add new spells
4. **AddSpellbook**: Form to add new spellbooks

Navigation within SearchOverlay:
- Row-based navigation in BrowseSpellbooks: Left/Right wrap within row, Up/Down wrap within column
- Command palette with `:` prefix - shows filterable command list
- Context-aware footer hints

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

**Available Themes** (cycle with `t` key or `:t` command):
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

## View Modes

**Available View Modes** (cycle with `v` key or use commands):
- auto (`:a`): Responsive - cards on wide screens, spines on narrow
- cards (`:c`): Large card view with sigils and descriptions
- spines (`:p`): Compact spine (book spine) view

View mode preference is persisted in `theme.toml` (`selected_view_mode`).

## CLI Arguments

- `--add`: Opens directly to Add Spell screen instead of SpellbookList

## Clipboard

The app uses external clipboard tools for maximum compatibility:

1. `wl-copy` (Wayland)
2. `xclip` (X11)
3. `xsel` (X11)

On successful copy, a D-Bus notification is sent via `notify-send`.

## Command Palette

The SearchOverlay supports a command palette (triggered with `:`):

1. Type `:` to enter command mode
2. A filtered list of matching commands appears
3. Use `↑`/`↓` to navigate, `Enter` to execute
4. `Esc` to cancel

| Command | Action |
|---------|--------|
| `:n` | New spell - open Add Spell form |
| `:b` | Browse spellbooks mode |
| `:s` | Browse spells mode |
| `:c` | Card view mode |
| `:p` | Spine view mode |
| `:t` | Cycle theme |
| `:?` | Show help |

Commands are defined in `src/ui/events.rs` with aliases for flexible matching.

## Job Execution System

Spellbook supports detached job execution for long-running commands.

### Overview

Jobs run in the background, detached from the TUI:
- The TUI can be closed while jobs are running
- Jobs continue executing independently
- Notifications are sent via D-Bus when jobs complete
- Job state is persisted to `~/.spellbook/jobs.toml`

### Components

```
src/
├── executor.rs          # JobManager, job spawning, notifications
├── ui/jobs.rs           # JobsPanel UI component
└── ui/confirm.rs        # Confirmation dialog for elevated/dangerous commands
~/.spellbook/            # Created on first run
  jobs.toml              # Job registry
  job_001.out           # stdout
  job_001.err           # stderr
```

### Job Lifecycle

```
Queued → Running → Completed
                   ↘ Failed
                   ↘ Cancelled
```

### Job States

| State | Description |
|-------|-------------|
| `Queued` | Waiting to run (respects 10-job limit) |
| `Running` | Currently executing |
| `Completed` | Exited with code 0 |
| `Failed` | Exited with non-zero code |
| `Cancelled` | Killed by user |

### JobManager

Handles job lifecycle:
- Spawns detached child processes
- Tracks job state via PID polling
- Persists job registry to TOML
- Sends D-Bus notifications on completion
- Enforces 10 concurrent job limit

### Commands

| Command | Action |
|---------|--------|
| `:jobs` | Open Jobs panel |
| `:kill <id>` | Kill a running job |
| `:cancel <id>` | Cancel a queued job |

### Notifications

D-Bus notifications via `notify-send`:
- Success: `"<spell_name> completed"`
- Failure: `"<spell_name> failed (exit <code>)"`
