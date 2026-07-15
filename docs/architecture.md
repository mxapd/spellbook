# Spellbook Architecture

## Overview

Spellbook is a TUI (Terminal User Interface) application for managing and executing CLI command snippets. It uses a fantasy/magic theme where commands are "spells" organized into "spellbooks", stored in a "codex".

This document describes the architecture, which is focused on:
- Clean state management with encapsulated component state
- Unified Mode/Overlay navigation system
- Three execution modes (Simple, TUI, Background)
- Background job management with notifications
- Virtual spellbooks (Favorites, Recent)

## Technology Stack

- **Language**: Rust
- **TUI Framework**: [ratatui](https://ratatui.rs/)
- **Terminal I/O**: crossterm
- **Serialization**: serde + toml
- **Clipboard**: wl-copy / xclip / xsel (system tools)
- **Notifications**: notify-send (D-Bus)
- **Job Management**: nohup (detached process execution)

## Data Storage Conventions

- **TOML is preferred** over JSON for all persistent data
- Human-editable configs should always use TOML
- All file writes use atomic write-to-temp + rename pattern

---

## Core Architecture Principles

### 1. Single-Mode Navigation

Spellbook uses a single primary mode with overlays, not multiple top-level screens:

```rust
pub enum Mode {
    BrowseSpellbooks,   // Home - card/spine view
    BrowseSpells,       // Spells in selected spellbook
    AddSpell,           // Form to add spell
    EditSpell,          // Form to edit spell
    AddSpellbook,       // Form to add spellbook
}

pub enum Overlay {
    OutputModal,        // Scrollable command output
    ConfirmDialog,      // Confirmation prompts
    CommandPalette,     // : command input
    Help,               // ? keybind reference
}
```

### 2. Elm Architecture (Mode with Nested State)

Spellbook implements the Elm Architecture pattern with nested state:

```rust
/// Mode is the single source of truth for application state
pub enum Mode {
    BrowseSpellbooks(BrowseState),  // Home - card/spine view
    BrowseSpells(BrowseState),      // Spells in selected spellbook
    AddSpell(FormState),            // Form to add spell
    EditSpell(FormState),           // Form to edit spell
    AddSpellbook(FormState),        // Form to add spellbook
}

/// Browse state shared across browse modes
pub enum BrowseState {
    Idle,
    Searching { 
        query: String, 
        filtered_indices: Vec<usize>, 
        selected_index: usize 
    },
}

/// Form state for add/edit modes
pub struct FormState {
    pub selected_field: usize,
    pub field_values: Vec<String>,
}

pub enum Overlay {
    OutputModal,        // Scrollable command output
    ConfirmDialog,      // Confirmation prompts
    CommandPalette,     // : command input
    Help,               // ? keybind reference
    InputPopup,         // Parameter input for placeholders
}
```

No parallel state machines - derive all state from Mode.

### 3. Jobs Sidebar (Not an Overlay)

The jobs sidebar is a **toggleable panel**, not an overlay:
- Sits on right side of screen
- Visible across all modes when toggled on
- Has its own focus state (`FocusTarget::JobsSidebar`)
- Does not block interaction with main content

---

## Module Structure

**Note:** The actual implementation uses consolidated modules rather than the granular structure originally planned.

```
src/
├── main.rs                  # Entry point, CLI args, terminal setup
├── state.rs                 # State struct, recents/jobs management
├── cli.rs                   # CLI argument parsing
├── clipboard.rs             # Clipboard operations
├── validation.rs            # Codex validation on load
├── logging.rs               # Logging setup
├── lib.rs                   # Library exports
├── test_utils.rs            # Test helpers
│
├── models/
│   ├── mod.rs               # Re-exports, FocusTarget, SpellbookRef
│   ├── spell.rs             # Spell, RunMode
│   ├── spellbook.rs         # Spellbook, SpineStyle
│   ├── codex.rs             # Codex struct
│   ├── job.rs               # Job, JobManager, RecentEntry (persistence types)
│   └── theme.rs             # Theme, UserSettings, ViewMode
│
├── archivist/
│   ├── mod.rs               # Re-exports
│   └── archivist.rs         # All persistence logic (consolidated)
│
├── invoker/
│   └── mod.rs               # All execution logic (consolidated)
│
└── ui/
    ├── mod.rs               # Mode, BrowseState, FormState, UiState
    ├── render.rs            # Top-level render dispatcher
    ├── events.rs            # Event router (Elm Architecture)
    ├── browse_spellbooks.rs # BrowseSpellbooks mode handler
    ├── browse_spells.rs     # BrowseSpells mode handler
    ├── form.rs              # AddSpell/EditSpell/AddSpellbook form handler
    ├── spellbook_browser.rs # Spellbook browser state
    ├── spell_list.rs        # Spell list rendering
    ├── add_spell_form.rs    # AddSpell form state
    ├── add_spellbook_form.rs # AddSpellbook form state
    ├── streaming_modal.rs   # Streaming output modal
    ├── jobs.rs              # Jobs sidebar
    ├── confirm.rs           # Confirm dialog
    ├── search_overlay.rs    # Search overlay rendering
    ├── input.rs             # InputPopup overlay
    └── help.rs              # Help overlay
```

---

## Data Flow

### Startup

1. **Parse CLI args** → Determine initial mode (`--add` opens AddSpell)
2. **Initialize logging** → `~/.spellbook/spellbook.log`
3. **Load codex** → `archivist::codex_store::load("codex.toml")`
4. **Load settings** → `Archivist::load_user_settings("config.toml")`
5. **Load theme** → `Archivist::load_theme("config.toml")`
6. **Load jobs** → `archivist::job_store::load()` from `~/.spellbook/jobs.toml`
7. **Load recents** → `archivist::recent_store::load()` from `~/.spellbook/recents.toml`
8. **Validate codex** → Check references, duplicates, required fields
9. **Create AppState** → Initialize all component states
10. **Start job poller** → Background thread monitors running jobs
11. **Enter event loop** → 60fps tick rate with crossterm events

### Event Loop

```
┌─────────────────────────────────────┐
│  Crossterm Event (Key, Mouse, etc) │
└──────────────┬──────────────────────┘
               │
               ▼
     ┌─────────────────────┐
     │   events::dispatch  │
     └──────────┬──────────┘
                │
         ┌──────┴──────┐
         │             │
    Overlay?      No   │
         │             ▼
        Yes    ┌───────────────┐
         │     │  Mode handler │
         │     └───────────────┘
         ▼
  ┌──────────────┐
  │Overlay handler│
  └──────────────┘
         │
         ▼
  ┌──────────────┐
  │Update AppState│
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐
  │    Render    │
  └──────────────┘
```

---

## Execution System

### Three Execution Modes

```rust
pub enum RunMode {
    Simple,      // Exit TUI, run in terminal
    Tui,         // Capture output in modal
    Background,  // Detach, track in jobs
}
```

### Simple Mode Flow

1. User presses `s` or `r` (if spell default is simple)
2. If `confirm = true` → show ConfirmDialog
3. **Write `recents.toml`** (critical - no chance after)
4. Shut down TUI (restore terminal)
5. Execute via `$SHELL -c "<incantation>"` using `exec()` (replaces process)
6. User is back at shell

### TUI Mode Flow

1. User presses `Ctrl+r` or `r` (if spell default is tui)
2. If `confirm = true` → show ConfirmDialog
3. Spawn child process with stdout/stderr piped
4. Open OutputModal overlay, stream output in real-time
5. **Streaming architecture**:
   - Process spawns with `stdout` and `stderr` captured via pipes
   - Background thread reads lines from both pipes concurrently
   - Lines sent via mpsc channel to main event loop
   - Event loop polls channel every tick (100ms timeout)
   - Lines appended to `OutputModalState::content` (cap: 10,000 lines)
   - Auto-scroll keeps view at bottom (toggle with `s` key)
6. **Live controls** while process runs:
   - `Ctrl+C`: Kill running process (sends SIGKILL)
   - `Ctrl+B`: Promote to background (restarts via JobManager)
7. On completion, show exit code (✓ for 0, ✗ for non-zero)
8. **Display features**:
   - Status indicator in title: ⟳ (running), ✓ (success), ✗ (failure)
   - Stderr highlighted in red
   - System messages ([stderr], [Process killed]) in muted color
   - Truncation warning when 10,000 line limit reached
   - Footer shows context-aware keyboard hints

### Background Mode Flow

1. User presses `Ctrl+b` or `r` (if spell default is background)
2. If `confirm = true` → show ConfirmDialog
3. Spawn detached process (nohup)
4. Create Job entry in JobManager
5. Persist to `~/.spellbook/jobs.toml`
6. Write stdout/stderr to `~/.spellbook/job_<id>.out/err`
7. Job appears in jobs sidebar
8. Background poller monitors process
9. D-Bus notification on completion/failure

---

## Virtual Spellbooks

Favorites and Recents are **virtual spellbooks** - generated dynamically, not stored in `codex.toml`:

### Favorites

- Contains all spells with `favorite = true`
- Only visible if at least one favorite exists
- Appears at top of spellbook list
- Cannot be edited or deleted directly

### Recent

- Contains recently used spells from `recents.toml`
- Sorted by most recent timestamp
- Only visible if recents exist
- Appears second in spellbook list (after Favorites)
- Limited to last 100 entries (FIFO)

### SpellbookRef Type

```rust
pub enum SpellbookRef {
    Virtual(VirtualKind),
    Codex(usize),
}

pub enum VirtualKind {
    Favorites,
    Recent,
}
```

This provides type-safe references to spellbooks, avoiding index offset bugs.

---

## Theming

10 built-in themes with ANSI color support:

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

Each theme defines:
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

Theme preference persisted in `config.toml`.

---

## Job System

### Job Lifecycle

```
Queued → Running → Completed
                 ↘ Failed
                 ↘ Cancelled
```

### JobManager

Responsibilities:
- Spawn detached child processes (nohup)
- Track job state via PID polling
- Persist job registry to `~/.spellbook/jobs.toml`
- Send D-Bus notifications on completion/failure
- Enforce 10 concurrent job limit

### Background Polling

- Single background thread polls all jobs periodically (every 1 second)
- Sends status updates via mpsc channel to event loop
- Event loop updates `JobManager` state each tick
- Notifications sent via `notify-send` when job completes/fails

### Job Output

- stdout → `~/.spellbook/job_<id>.out`
- stderr → `~/.spellbook/job_<id>.err`
- Viewable in OutputModal by selecting job in sidebar

---

## Focus Management

When jobs sidebar is open, focus can be on main content or sidebar:

```rust
pub enum FocusTarget {
    Main,
    JobsSidebar,
}
```

- **Tab** key cycles focus: Main ↔ JobsSidebar
- Focus determines which component receives key events
- Visual indicator shows which component has focus

---

## Event Handling Priority

When a key event arrives:

1. **Active overlay** (topmost if multiple) - ConfirmDialog, OutputModal, CommandPalette, Help
2. **Jobs sidebar** (if focused)
3. **Current mode** - BrowseSpellbooks, BrowseSpells, AddSpell, EditSpell, AddSpellbook
4. **Global keybinds** - `/`, `:`, `t`, `v`, `q`, `?`, `Tab`

If a handler consumes the event, stop propagation.

---

## Persistence Strategy

### Atomic Writes

All TOML files written atomically:
```rust
// Pattern
write_to_temp("{file}.tmp")
fs::rename("{file}.tmp", "{file}")
```

Prevents corruption if process dies mid-write.

### File Locations

| File | Purpose |
|------|---------|
| `codex.toml` | Spells and spellbooks |
| `config.toml` | Theme selection and user settings (view mode) |
| `~/.spellbook/jobs.toml` | Job registry |
| `~/.spellbook/job_<id>.out` | Job stdout |
| `~/.spellbook/job_<id>.err` | Job stderr |
| `~/.spellbook/recents.toml` | Recently used spells |
| `~/.spellbook/spellbook.log` | Application logs |

### Retention Policies

- **Recents**: Keep last 100, FIFO eviction
- **Jobs**: Keep last 50, auto-purge on startup
- **Logs**: Rotate at 5MB

---

## Validation Rules

On load, validate codex:

- No duplicate spell IDs
- No duplicate spell names (warning only)
- No empty spell names or incantations
- Spellbook spell references must point to existing spell IDs
- Warn (don't crash) on invalid references - skip and log
- `run_mode` must be one of: simple, tui, background (default to simple if invalid)

---

## Error Handling

- Missing `codex.toml` → create default empty one
- Missing `config.toml` → create with default theme and settings
- Missing `~/.spellbook/` → create directory
- Invalid TOML → show error message in TUI, don't crash
- Clipboard tool not found → show error in footer
- Job process spawn failure → mark job as Failed, log error
- Invalid `working_dir` → fall back to `$HOME`, log warning

---

## Forward Compatibility

- Use `#[serde(default)]` for all new optional fields
- Old TOML files load successfully with defaults for missing fields

---

## CLI Arguments

- `--add`: Opens directly to AddSpell form instead of BrowseSpellbooks

Future:
- `--import <file>`: Import spells from file
- `--export <file>`: Export codex to file

---

## Logging

- **File**: `~/.spellbook/spellbook.log`
- **Rotation**: Keep last 5MB
- **Levels**: ERROR, WARN, INFO, DEBUG
- **Control**: Set via `SPELLBOOK_LOG` env var (e.g., `SPELLBOOK_LOG=debug`)

Example log entries:
```
[INFO] Loaded 42 spells from codex.toml
[WARN] Invalid spell reference in spellbook 'System': unknown ID 550e8400...
[ERROR] Failed to spawn job 5: working_dir /invalid/path does not exist
[DEBUG] Job 3 status check: Running (PID 12345)
```

---

## See Also

- [architecture-diagrams-mermaid.md](architecture-diagrams-mermaid.md) - Visual diagrams of module relationships, data flow, and component hierarchy (useful for debugging)
