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
- IDs should be stable (use UUIDs or persistent IDs, not sequential on load)
- If a command needs `sudo`, write it in the incantation directly — no separate elevation flag

### Spellbook

A named collection of spells.

```toml
[[spellbooks]]
name = "System"
cover = "System monitoring and management commands"
sigil = "∧"
spells = ["List Processes", "Kill Process", "Rebuild NixOS"]
```

| Field    | Type         | Required | Description                         |
| -------- | ------------ | -------- | ----------------------------------- |
| `name`   | String       | Yes      | Display name                        |
| `cover`  | String       | No       | Description / purpose               |
| `sigil`  | String       | No       | Emoji or symbol for visual identity |
| `spells` | Vec<String>  | No       | References to spell names           |
| `style`  | String       | No       | Spine style for spine view mode     |

### Codex

Root data structure. This is the in-memory representation of `codex.toml`.

```rust
pub struct Codex {
    pub spells: Vec<Spell>,
    pub spellbooks: Vec<Spellbook>,
}
```

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
}```

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
	    pub spellbook_index: usize,  // which spellbook we're browsing
	    pub selected_index: usize,
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
1. User presses s or r (if spell default is simple)
2. If confirm flag is set → show ConfirmDialog overlay
3. TUI shuts down completely (restore terminal)
4. Command is executed via the user's shell
5. App process exits — user is back at their shell

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

Key	Action
:	Open command palette
t	Cycle theme
v	Cycle view mode
q	Quit
Esc	Close overlay / go back
?	Show help overlay

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
	├── persistence/
	│   ├── mod.rs
	│   ├── codex_store.rs       # Load/save codex.toml (full serialization)
	│   ├── config_store.rs      # Load/save config.toml
	│   ├── theme_store.rs       # Load/save theme.toml
	│   ├── job_store.rs         # Load/save jobs.toml and job output files
	│   └── recent_store.rs      # Load/save recents.toml
	│
	├── executor/
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
- persistence/: Each store handles one file. Full serialize/deserialize — no line-by-line editing. All stores are stateless functions that take a path and return/accept data.
- executor/: Each execution mode is its own module. job_manager.rs handles the lifecycle of background jobs (start, poll, notify, kill).
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
