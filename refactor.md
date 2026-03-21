# Spellbook v2 — Architecture Redesign Specification

## Overview

Spellbook is a TUI application for managing and executing CLI command snippets.
It uses a fantasy/magic theme where commands are "spells" organized into
"spellbooks", stored in a "codex".

This document is the complete specification for a redesign of the application.
The goal is to clean up the architecture, simplify the state model, and
support all planned features in a maintainable way.

## Technology Stack

- **Language**: Rust
- **TUI Framework**: ratatui
- **Terminal I/O**: crossterm
- **Serialization**: serde + toml
- **Clipboard**: wl-copy / xclip / xsel (system tools)
- **Notifications**: notify-send (D-Bus)

---

## Data Model

### Spell

A single command snippet with metadata.

```toml
[[spells]]
id = "550e8400-e29b-41d4-a716-446655440000"
name = "Rebuild NixOS"
incantation = "sudo nixos-rebuild switch"
lore = "Rebuild and switch to new system configuration"
school = "NixOS"
glyphs = ["nix", "rebuild", "system"]
confirm = true
run_mode = "background"
working_dir = "/etc/nixos"
favorite = false
```

| Field         | Type         | Required | Default    | Description                                |
| ------------- | ------------ | -------- | ---------- | ------------------------------------------ |
| `id`          | String       | Yes      | —          | UUID (e.g. "550e8400-e29b-41d4-a716-446655440000") |
| `name`        | String       | Yes      | —          | Display name                               |
| `incantation` | String       | Yes      | —          | The actual command(s) to run               |
| `lore`        | String       | No       | `""`       | Description / usage notes                  |
| `school`      | String       | No       | `""`       | Category (e.g. "System", "Network")        |
| `glyphs`      | Vec<String>  | No       | `[]`       | Search tags                                |
| `confirm`     | bool         | No       | `false`    | Require confirmation before any execution  |
| `run_mode`    | String       | No       | `"simple"` | Default execution mode: simple/tui/background |
| `working_dir` | String       | No       | `""`       | Working directory for execution            |
| `favorite`    | bool         | No       | `false`    | Whether the spell is favorited             |

**Notes:**
- IDs are UUIDs generated with `uuid::Uuid::new_v4()` when spell is created
- IDs must be stable and persist across application restarts
- If a command needs `sudo`, write it in the incantation directly — no separate elevation flag

### Spellbook

A named collection of spells.

```toml
[[spellbooks]]
name = "System"
cover = "System monitoring and management commands"
sigil = "∧"
spells = [
    "550e8400-e29b-41d4-a716-446655440000",
    "660e9401-f30c-52e5-b827-557766551111",
    "770f0512-g41d-63f6-c938-668877662222"
]
```

| Field    | Type         | Required | Description                         |
| -------- | ------------ | -------- | ----------------------------------- |
| `name`   | String       | Yes      | Display name                        |
| `cover`  | String       | No       | Description / purpose               |
| `sigil`  | String       | No       | Emoji or symbol for visual identity |
| `spells` | Vec<String>  | No       | References to spell IDs (UUIDs)     |
| `style`  | String       | No       | Spine style for spine view mode     |

### Codex

Root data structure. This is the in-memory representation of `codex.toml`.

```rust
pub struct Codex {
    pub spells: Vec<Spell>,
    pub spellbooks: Vec<Spellbook>,
}
```

### SpellbookRef

Used to reference spellbooks, distinguishing between virtual and codex spellbooks.

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

**Notes:**
- Virtual spellbooks (Favorites, Recent) are generated dynamically at runtime
- Codex spellbooks are stored in `codex.toml`
- This enum provides type safety when navigating between spellbook types

### Job

Represents a running or completed background command.

```rust
pub struct Job {
    pub id: u64,
    pub spell_name: String,
    pub command: String,
    pub status: JobStatus,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub started_at: DateTime,
    pub completed_at: Option<DateTime>,
    pub output_file: PathBuf,
    pub error_file: PathBuf,
}

pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

---

## Persistence

### File Layout

| File                          | Purpose                              |
| ----------------------------- | ------------------------------------ |
| `codex.toml`                  | Spells and spellbooks                |
| `config.toml`                 | User settings (view mode, defaults)  |
| `theme.toml`                  | Theme selection only                 |
| `~/.spellbook/jobs.toml`      | Job registry                         |
| `~/.spellbook/job_<id>.out`   | Job stdout                           |
| `~/.spellbook/job_<id>.err`   | Job stderr                           |
| `~/.spellbook/recents.toml`   | Recently used spells                 |

### config.toml

```toml
view_mode = "auto"          # auto / cards / spines
default_run_mode = "simple"  # simple / tui / background
```

### theme.toml

```toml
selected_theme = "dracula"
```

### recents.toml

```toml
[[recents]]
spell_name = "List Processes"
timestamp = 2026-03-21T10:30:00Z
action = "run"  # "run" or "copy"
```

### Persistence Rules

- TOML is preferred over JSON for all files
- When saving spells (add/edit/delete), serialize and write the full codex — do NOT do line-by-line text manipulation
- All config files should be human-readable and hand-editable
- Job output files are plain text

---

## Screen Architecture

There is ONE primary screen with multiple modes. No separate Screen enum with
multiple top-level screens — everything lives within the main view.

### Modes

```rust
pub enum Mode {
    BrowseSpellbooks,    // Home — card/spine view of spellbooks
    BrowseSpells,        // Spells inside a selected spellbook
    AddSpell,            // Form to add a new spell
    EditSpell,           // Form to edit an existing spell
    AddSpellbook,        // Form to add a new spellbook
}
```

### Overlays

Overlays render on top of the current mode. Multiple can be active (e.g. jobs
sidebar + confirm dialog).

```rust
pub enum Overlay {
    OutputModal,         // Scrollable command output viewer
    ConfirmDialog,       // "Are you sure?" before execution
    CommandPalette,      // `:` command input with filtered list
    Help,                // `:?` keybind reference
}
```

### Jobs Sidebar

The jobs sidebar is NOT an overlay or a mode. It's a **toggleable panel** that
sits on the right side of the screen and coexists with any mode.

- Toggle with `:jobs`
- Shows compact job list with status icons
- Navigate with keybinds to select a job
- Enter on a job opens the OutputModal overlay with that job's output
- Visible across all modes when toggled on

### Layout
┌─────────────────────────────────────────────┬──────────────┐
│                                             │  Jobs Sidebar │
│           Main Content Area                 │  (optional)   │
│   (mode-dependent: cards, spell list,       │               │
│    form, etc.)                              │  ⟳ Rebuild    │
│                                             │  ✓ Scan       │
│                                             │  ✗ Deploy     │
│                                             │               │
├─────────────────────────────────────────────┴──────────────┤
│  Footer: context-aware hints                                │
└─────────────────────────────────────────────────────────────┘

When an overlay is active, it renders centered on top of everything:

┌─────────────────────────────────────────────┬──────────────┐
│              ┌─────────────────┐            │  Jobs Sidebar │
│              │  Output Modal   │            │               │
│              │  (scrollable)   │            │               │
│              │                 │            │               │
│              │  > output here  │            │               │
│              │                 │            │               │
│              └─────────────────┘            │               │
├─────────────────────────────────────────────┴──────────────┤
│  Footer: context-aware hints                                │
└─────────────────────────────────────────────────────────────┘

---

## State Architecture

### AppState

Top-level application state. This is the single source of truth.

```rust
pub struct AppState {
    // Data
    pub codex: Codex,
    pub jobs: JobManager,
    pub recents: Vec<RecentEntry>,

    // UI state
    pub mode: Mode,
    pub overlays: Vec<Overlay>,
    pub jobs_sidebar_open: bool,
    pub focus: FocusTarget,
    pub theme: Theme,
    pub config: Config,

    // Mode-specific state (encapsulated)
    pub spellbook_browser: SpellbookBrowserState,
    pub spell_browser: SpellBrowserState,
    pub spell_form: SpellFormState,
    pub spellbook_form: SpellbookFormState,
    pub output_modal: OutputModalState,
    pub command_palette: CommandPaletteState,
    pub confirm_dialog: ConfirmDialogState,
    pub jobs_sidebar: JobsSidebarState,
}

pub enum FocusTarget {
    Main,         // Focus is on main content area
    JobsSidebar,  // Focus is on jobs sidebar
}
```

Key principle: Each UI component owns its state in a dedicated struct.

No flat add_spell_name, add_spell_command, etc. fields on a god-state object.

```rust
SpellbookBrowserState

	pub struct SpellbookBrowserState {
	    pub selected_index: usize,
	    pub items_per_row: usize,
	    pub search_query: String,
	    pub filtered_indices: Vec<usize>,
	}
```

### SpellBrowserState
```rust
	pub struct SpellBrowserState {
	    pub spellbook: SpellbookRef,  // which spellbook we're browsing (virtual or codex)
	    pub selected_index: usize,
	    pub search_active: bool,
	    pub search_query: String,
	    pub filtered_indices: Vec<usize>,
	}
```

### SpellFormState
Used for BOTH add and edit. Populated with existing data when editing.
```rust
	pub struct SpellFormState {
	    pub active_field: SpellFormField,
	    pub name: String,
	    pub incantation: String,
	    pub lore: String,
	    pub school: String,
	    pub glyphs: String,          // comma-separated input
	    pub run_mode: RunMode,
	    pub confirm: bool,
	    pub working_dir: String,
	    pub spellbook_index: Option<usize>,
	    pub dropdown_open: bool,
	    pub dropdown_index: usize,
	    pub editing_spell_id: Option<SpellId>, // None = adding, Some = editing
	    pub dirty: bool,             // tracks if form has unsaved changes
	}
```

### OutputModalState
```rust
	pub struct OutputModalState {
	    pub content: Vec<String>,    // output lines
	    pub scroll_offset: usize,
	    pub is_streaming: bool,      // true while command is still running
	    pub exit_code: Option<i32>,
	    pub source: OutputSource,    // which job or TUI run this is from
	}
```

---

### Execution System
Three Execution Modes
```rust
	pub enum RunMode {
	    Simple,      // Exit TUI, run in terminal. App is done.
	    Tui,         // Run command, show output in modal overlay.
	    Background,  // Detach, track in jobs sidebar.
	}
```

#### Execution Flow

##### Simple Mode
1. User presses `s` or `r` (if spell's default `run_mode` is `simple`)
2. If `confirm = true` → show ConfirmDialog overlay
3. **CRITICAL**: Write to `recents.toml` before executing (no chance after)
4. TUI shuts down completely (restore terminal state)
5. Command is executed via `$SHELL -c "<incantation>"` using `exec()` (replaces process)
6. User is back at their shell with command running/completed

**Implementation Notes**:
- Use `std::process::Command` with `.exec()` on Unix (process replacement)
- If `working_dir` is invalid, fall back to `$HOME` and log warning
- If `$SHELL` is not set, default to `/bin/sh`

##### TUI Mode

1. User presses Ctrl+r or r (if spell default is tui)
2. If confirm flag is set → show ConfirmDialog overlay
3. Command spawns as child process with captured stdout/stderr
4. OutputModal overlay opens, streams output in real-time
5. On completion, shows exit code
6. User can press a keybind to promote to background if taking too long
7. Esc to dismiss modal

##### Background Mode

1. User presses Ctrl+b or r (if spell default is background)
2. If confirm flag is set → show ConfirmDialog overlay
3. Command spawns as detached process (nohup)
4. Job added to JobManager and persisted to jobs.toml
5. stdout/stderr written to ~/.spellbook/job_<id>.out/err
6. Jobs sidebar shows progress
7. D-Bus notification on completion/failure

#### TUI Mode Streaming Architecture

When running commands in TUI mode, output streaming works as follows:

1. **Child Process Spawn**: Command spawns with `stdout` and `stderr` piped
2. **Background Thread**: Dedicated thread reads from pipes line-by-line using `BufReader`
3. **Message Channel**: Lines sent via `tokio::sync::mpsc` channel to main event loop
4. **Event Loop Polling**: Main loop polls channel each tick (~16ms / 60fps)
5. **Content Buffer**: Lines appended to `OutputModalState::content` (capped at 10,000 lines to prevent memory issues)
6. **Real-time Display**: Modal shows output with auto-scroll to bottom
7. **Promotion**: On Ctrl+b (promote to background):
   - All captured lines written to job output file
   - Process becomes detached (re-parented to init)
   - Job entry created in JobManager
   - Modal closes, job appears in sidebar

#### Unsaved Changes Handling

When `Esc` is pressed in `AddSpell`, `EditSpell`, or `AddSpellbook` modes:

- If `dirty` flag is `true` → show `ConfirmDialog` overlay: "Discard unsaved changes?"
  - If confirmed → clear form state, set `dirty = false`, return to previous mode
  - If cancelled → stay in form, keep `dirty = true`
- If `dirty` flag is `false` → immediately return to previous mode
- Successful save → set `dirty = false`

**When to set `dirty = true`**:
- Any character typed in any form field
- Toggle checkboxes (confirm, favorite, etc.)
- Change dropdown selection (run_mode, spellbook)
- Any modification to form state from default/loaded values

**When to set `dirty = false`**:
- Successful save operation
- User confirms discard via ConfirmDialog
- Form is freshly initialized (Add mode) or loaded (Edit mode)

Promotion: TUI → Background


While viewing a TUI run in the output modal:


- Press a keybind (e.g. Ctrl+b) to promote
- The running process is detached
- A Job entry is created in JobManager
- Output modal closes
- Job appears in jobs sidebar


---

### Keybind Map

#### Global (all modes)

| Key | Action |
|-----|--------|
| `/` | Activate search mode (in Browse modes) |
| `Tab` | Cycle focus (Main ↔ Jobs Sidebar when sidebar open) |
| `:` | Open command palette |
| `t` | Cycle theme |
| `v` | Cycle view mode |
| `q` | Quit |
| `Esc` | Close overlay / deactivate search / go back |
| `?` | Show help overlay |

#### BrowseSpellbooks Mode

Key	Action
←→↑↓	Navigate (row-aware wrapping)
Enter	Open selected spellbook
Type	Filter spellbooks by name

#### BrowseSpells Mode

Key	Action
↑↓	Navigate spell list
Enter	Copy incantation to clipboard
r	Run with spell's default mode
s	Force simple run
Ctrl+r	Force TUI run
Ctrl+b	Force background run
e	Edit selected spell
d	Delete selected spell
f	Toggle favorite
Esc / ←	Back to spellbooks
Type	Filter spells

#### Output Modal

Key	Action
↑↓	Scroll output
Ctrl+b	Promote to background
Esc	Close modal

#### Jobs Sidebar

Key	Action
↑↓	Navigate job list
Enter	View job output in modal
k	Kill selected running job
Esc	Close sidebar

#### Command Palette

Command	Action
:n	New spell (open AddSpell form)
:nb	New spellbook
:b	Browse spellbooks
:s	Browse spells
:jobs	Toggle jobs sidebar
:c	Card view mode
:p	Spine view mode
:a	Auto view mode
:t	Cycle theme
:?	Show help
:import	Import spells from file
:export	Export codex to file

---

### Features

1. Browse Spellbooks (Home)
    - App opens to spellbook cards/spines view
    - View modes: cards, spines, auto (responsive based on terminal width)
    - Row-aware navigation with wrapping
    - Type to filter spellbooks by name
    - Enter to open a spellbook

2. Browse Spells
    - View spells inside a selected spellbook
    - See spell details (incantation, lore, school, glyphs)
    - Type to filter spells within the current spellbook
    - Copy, run, edit, delete, or favorite spells

3. Search
    - Inline search — typing immediately filters in the current context
    - In BrowseSpellbooks: filters spellbooks by name
    - In BrowseSpells: filters spells by name, lore, and glyphs
    - No separate search mode or key to activate — just start typing

4. Copy to Clipboard
    - Enter copies the spell incantation to clipboard
    - Uses system clipboard tools: wl-copy (Wayland), xclip (X11), xsel (X11)
    - D-Bus notification on successful copy
    - Records the action in recents

5. Spell Execution
    - Three modes: Simple, TUI, Background
    - Each spell has a default run_mode (defaults to simple if omitted)
    - Runtime keybind overrides: r (default), s (simple), Ctrl+r (tui), Ctrl+b (background)
    - Spells with confirm = true show a confirmation dialog before any execution
    - TUI runs can be promoted to background
    - Records the action in recents

6. Jobs Sidebar
    - Toggled with :jobs
    - Right-side panel visible alongside any mode
    - Shows jobs with status icons: ⟳ Running, ✓ Completed, ✗ Failed, ⊘ Cancelled
    - Navigate and select jobs
    - Enter opens output modal for the selected job
    - Kill running jobs with k
    - Persisted to ~/.spellbook/jobs.toml
    - Background polling thread checks process status
    - D-Bus notification on job completion/failure

7. Output Modal
    - Shared viewer for TUI run output and background job output
    - Scrollable with arrow keys
    - Streams output in real-time for active runs
    - Shows exit code on completion
    - Promote active TUI run to background with Ctrl+b
    - Dismiss with Esc

8. Add Spell
    - Form with fields: name, incantation, lore, school, glyphs, run_mode, confirm, working_dir, spellbook
    - Spellbook dropdown selector
    - Accessible via :n command or --add CLI flag
    - Saves to codex.toml (full serialization, not line editing)

9. Edit Spell
    - Opens the same form as Add Spell, pre-populated with existing data
    - Accessible via e key on a selected spell
    - Updates codex.toml on save

10. Delete Spell
    - Confirmation dialog before deleting
    - Removes spell from codex.toml
    - Removes spell references from all spellbooks
    - Accessible via d key on a selected spell

11. Add Spellbook
    - Form with fields: name, cover, sigil, style
    - Accessible via :nb command

12. Favorites
    - Toggle favorite on a spell with f key
    - favorite field persisted in codex.toml
    - A virtual "Favorites" spellbook appears in BrowseSpellbooks if any favorites exist
    - This spellbook is not stored in codex.toml — it's generated dynamically from favorited spells

13. Recent Items
    - Track recently used spells (copy or run actions) with timestamps
    - Persisted in ~/.spellbook/recents.toml
    - A virtual "Recent" spellbook appears in BrowseSpellbooks
    - Shows most recent N items, sorted by timestamp
    - This spellbook is not stored in codex.toml — it's generated dynamically
    
14. Import / Export
    - Export: Export entire codex or a single spellbook to a .toml file
    	- Uses the same codex.toml format so exported files are directly usable
    	- Command: :export (exports full codex) or :export <spellbook_name>
    
    - Import: Load spells and spellbooks from an external .toml file
    	- Merges into existing codex
    	- Handles duplicates (skip, overwrite, or rename — prompt user)
    	- Command: :import <path>

15. Theming
    - 10 built-in themes: default, default-light, dracula, gruvbox-dark, gruvbox-light, nord, catppuccin, one-dark, solarized-dark, solarized-light
    - Cycle with t key or :t command
    - Persisted in theme.toml
    - Each theme defines: bg, fg, accent, muted, selection, border colors
    
16. View Modes
    - Cards: large card view with sigils and descriptions
    - Spines: compact book spine view
    - Auto: responsive — cards on wide terminals, spines on narrow
    - Cycle with v key or commands :c, :p, :a
    - Persisted in config.toml
    
17. Command Palette
    - Triggered with :
    - Filterable list of available commands
    - Arrow keys to navigate, Enter to execute
    - Esc to cancel
    
18. CLI Arguments
    - --add: Opens directly to Add Spell form
    - Default: Opens to BrowseSpellbooks


---

### Module Structure
	
    src/
	├── main.rs                  # Entry point, CLI args, terminal setup
	├── app.rs                   # AppState, main event loop, top-level dispatch
	├── config.rs                # Config struct, load/save config.toml
	│
	├── models/
	│   ├── mod.rs
	│   ├── spell.rs             # Spell struct, SpellId, RunMode
	│   ├── spellbook.rs         # Spellbook struct, SpineStyle
	│   ├── codex.rs             # Codex struct (spells + spellbooks)
	│   └── job.rs               # Job struct, JobStatus
	│
	├── archivist/
	│   ├── mod.rs
	│   ├── codex_store.rs       # Load/save codex.toml (full serialization)
	│   ├── config_store.rs      # Load/save config.toml
	│   ├── theme_store.rs       # Load/save theme.toml
	│   ├── job_store.rs         # Load/save jobs.toml and job output files
	│   └── recent_store.rs      # Load/save recents.toml
	│
	├── invoker/
	│   ├── mod.rs
	│   ├── simple.rs            # Simple mode: exit TUI, exec command
	│   ├── tui_runner.rs        # TUI mode: spawn with captured output
	│   ├── background.rs        # Background mode: detach with nohup
	│   └── job_manager.rs       # Job lifecycle, polling, notifications
	│
	├── theme/
	│   ├── mod.rs
	│   └── themes.rs            # Theme definitions, RatatuiColors
	│
	├── ui/
	│   ├── mod.rs               # Mode enum, Overlay enum
	│   ├── render.rs            # Top-level render dispatcher
	│   ├── events.rs            # Top-level event dispatcher
	│   ├── components/
	│   │   ├── mod.rs
	│   │   ├── spellbook_browser.rs  # Card/spine rendering + state
	│   │   ├── spell_browser.rs      # Spell list rendering + state
	│   │   ├── spell_form.rs         # Add/edit spell form + state
	│   │   ├── spellbook_form.rs     # Add spellbook form + state
	│   │   ├── output_modal.rs       # Output viewer overlay + state
	│   │   ├── confirm_dialog.rs     # Confirmation overlay + state
	│   │   ├── command_palette.rs    # Command palette overlay + state
	│   │   ├── jobs_sidebar.rs       # Jobs panel + state
	│   │   ├── help.rs               # Help overlay
	│   │   └── footer.rs             # Context-aware footer hints
	│   └── widgets/
	│       ├── mod.rs
	│       ├── card.rs               # Spellbook card widget
	│       ├── spine.rs              # Spellbook spine widget
	│       └── search_input.rs       # Inline search input widget
	│
	├── clipboard.rs             # Clipboard operations
	├── notifications.rs         # D-Bus notification helpers
	├── validation.rs            # Codex validation (references, duplicates)
	├── logging.rs               # Logging setup
	└── cli.rs                   # CLI argument parsing

Module Responsibilities
- main.rs: Parse CLI args, initialize terminal, create AppState, run event loop, restore terminal on exit.
- app.rs: Owns AppState. Dispatches events to the correct handler based on active mode/overlay. Coordinates between UI, executor, and persistence.
- models/: Pure data structures with serde derives. No logic beyond basic methods (e.g. Spell::requires_confirmation()).
- archivist/: Each store handles one file. Full serialize/deserialize — no line-by-line editing. All stores are stateless functions that take a path and return/accept data.
- invoker/: Each execution mode is its own module. job_manager.rs handles the lifecycle of background jobs (start, poll, notify, kill).
- ui/render.rs: Dispatches to the correct component renderer based on current mode. Renders overlays on top. Renders jobs sidebar if open.
- ui/events.rs: Dispatches key events based on active overlay (overlays take priority) → then mode → then global keybinds.
- ui/components/: Each component is self-contained with its own state struct, render function, and event handler function.
- ui/widgets/: Reusable ratatui widgets (card, spine, search input).


---

### Event Handling Priority

When a key event arrives, it should be handled in this order:

1. Active overlay (topmost first if multiple) — ConfirmDialog, OutputModal, CommandPalette, Help
2. Jobs sidebar (if focused)
3. Current mode — BrowseSpellbooks, BrowseSpells, AddSpell, EditSpell, AddSpellbook
4. Global keybinds — :, t, v, q, ?

If a handler consumes the event, stop propagation.


---

### Virtual Spellbooks

Favorites and Recents appear as spellbooks in BrowseSpellbooks but are not
stored in codex.toml. They are generated dynamically:

- Favorites: Contains all spells with favorite = true. Only visible if at least one favorite exists.
- Recent: Contains recently used spells from recents.toml, ordered by most recent. Only visible if recents exist.

These virtual spellbooks should appear at the top of the spellbook list,
before user-defined spellbooks. They should be visually distinguishable
(e.g. different border style or muted accent).
Virtual spellbooks cannot be edited, deleted, or exported directly.

---

### Validation Rules

On load, validate the codex:

- No duplicate spell names
- No empty spell names
- No empty incantations
- Spellbook spell references must point to existing spell names
- Warn (don't crash) on invalid references — skip them and log a warning
- run_mode must be one of: simple, tui, background (default to simple if invalid)

---

### Error Handling

- Missing codex.toml → create a default empty one
- Missing config.toml → create with defaults
- Missing theme.toml → create with default theme
- Missing ~/.spellbook/ directory → create it
- Invalid TOML → show error message in TUI, don't crash
- Clipboard tool not found → show error in footer, don't crash
- Job process spawn failure → show error, mark job as Failed

---

### Implementation Notes

- Use stable spell IDs (UUIDs or content-based hashing) instead of sequential
IDs generated on load. This prevents issues with jobs referencing stale IDs.

- Persist codex by full serialization (toml::to_string + write). Do NOT
parse and edit lines manually.

- Each UI component should implement a consistent interface:
	- render(state, frame, area, theme) — draw the component
	- handle_event(state, event) -> EventResult — process input
	- EventResult indicates whether the event was consumed

- The event loop should run at a reasonable tick rate (e.g. 60fps / 16ms)
with an async channel for job status updates.

- Simple run mode must fully restore the terminal before executing the command
and then exit the process. The command should be executed via exec (replace
process) or std::process::Command with inherited stdio.


--- 

## Operational Policies

Define limits and behaviors for system resources:

### File Retention

- **recents.toml**: Keep last 100 entries, FIFO eviction when limit reached
- **jobs.toml**: Keep last 50 jobs, auto-purge completed/failed jobs on startup
- **Job output files**: Retained as long as job is in registry, purged when job is removed

### Resource Limits

- **OutputModalState**: Cap at 10,000 lines per job to prevent memory issues
  - When limit reached, truncate from beginning (keep most recent)
  - Display warning: "Output truncated (showing last 10,000 lines)"
- **Job concurrency**: Maximum 10 concurrent running jobs
  - Additional jobs enter `Queued` state
  - Automatically started when a running job completes

### File Operations

- **Atomic writes**: Use write-to-temp + atomic rename for all TOML files
  - Pattern: Write to `{file}.tmp`, then `fs::rename()` to `{file}`
  - Prevents corruption if process dies mid-write
- **working_dir validation**: If path is invalid at execution time:
  - Fall back to `$HOME`
  - Log warning to `spellbook.log`
  - Show warning in TUI footer

### Data Migrations

- **Serde compatibility**: Use `#[serde(default)]` for all new optional fields
  - Enables forward compatibility when adding fields
  - Old TOML files load successfully with defaults for missing fields
- **Spell ID generation**: 
  - New spells: Generate with `uuid::Uuid::new_v4()`, store as string
  - Legacy spells (no ID): Generate UUID from name hash on first load, persist to file
  - Migration: On first v2 load, assign UUIDs to all spells, rewrite `codex.toml`

### Logging

- **Log file**: `~/.spellbook/spellbook.log`
- **Rotation**: Keep last 5MB, rotate when exceeded
- **Levels**: ERROR, WARN, INFO, DEBUG (configurable via env var `SPELLBOOK_LOG`)


--- 


## RESOLVED CONSIDERATIONS

The following design decisions were made during specification review. These issues have been addressed in the spec above:

### 1. Spell ID Stability ✓
**Issue**: Spell references by name cause problems when spells are renamed.
**Resolution**: Added explicit `id` field (UUID) to Spell TOML format. Spellbooks reference spells by ID, not name.

### 2. Search vs Hotkey Conflicts ✓
**Issue**: "Just start typing" for search conflicts with single-key actions (e, d, f, r).
**Resolution**: Use `/` key to activate search mode explicitly, avoiding ambiguity.

### 3. Virtual Spellbook Indexing ✓
**Issue**: `SpellBrowserState::spellbook_index: usize` is ambiguous for virtual vs codex spellbooks.
**Resolution**: Introduced `SpellbookRef` enum with `Virtual(VirtualKind)` and `Codex(usize)` variants.

### 4. Focus Management ✓
**Issue**: No defined focus behavior when jobs sidebar is open.
**Resolution**: Added `FocusTarget` enum to AppState, Tab key cycles focus between Main and JobsSidebar.

### 5. Unsaved Changes Handling ✓
**Issue**: No confirmation when Esc pressed in forms with unsaved changes.
**Resolution**: Added `dirty` flag to `SpellFormState`, show ConfirmDialog when dirty and Esc pressed.

### 6. Simple Mode Execution ✓
**Issue**: Underspecified how simple mode executes commands.
**Resolution**: Use `$SHELL -c` with `exec()` replacement. Write `recents.toml` **before** exec (no after).

### 7. TUI Streaming Architecture ✓
**Issue**: No description of how stdout/stderr flows from child to UI.
**Resolution**: Added detailed architecture: background thread → mpsc channel → event loop polling.

### 8. Output Memory Limits ✓
**Issue**: `OutputModalState::content: Vec<String>` can grow unbounded.
**Resolution**: Cap at 10,000 lines with truncation from beginning when exceeded.

### 9. Recents & Jobs Retention ✓
**Issue**: No defined limits for `recents.toml` and `jobs.toml` growth.
**Resolution**: Keep last 100 recents (FIFO), last 50 jobs (auto-purge on startup).

### 10. Atomic File Writes ✓
**Issue**: No mention of atomic writes for TOML files (corruption risk).
**Resolution**: All persistence modules use write-to-temp + atomic rename pattern.

### Additional Clarifications

The following details were also specified:

- **Job polling**: Single background thread polls all jobs periodically, sends updates via mpsc channel
- **Job ID generation**: Monotonic counter stored in `jobs.toml` (`next_id` field)
- **Form navigation**: Tab and Arrow keys move between fields (specified in keybinds)
- **Scroll behavior**: Cursor-follows scrolling (selected item stays visible, scrolls when near edge)
- **Footer hints**: Context-aware (e.g., "Enter: copy | e: edit | d: delete | /: search")
- **Notifications**: Format is `"{spell_name} completed"` or `"{spell_name} failed (exit {code})"`
- **CLI pre-population**: `--add` opens blank form (no pre-population in v2 scope)
- **Import conflicts**: Show overlay with options: Skip / Overwrite / Rename (append number)
- **Undo**: Not in v2 scope (future enhancement)
