# Spellbook v2 Architecture Diagram

This document provides visual representations of the Spellbook v2 architecture to aid in debugging and understanding the codebase structure.

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              spellbook (binary)                             │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐     ┌─────────────────┐     ┌─────────────────┐
│     main      │────▶│      cli        │     │    logging      │
│   (entry)     │     │   (arg parse)   │     │   (log macros)  │
└───────┬───────┘     └─────────────────┘     └─────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                    State                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │    Codex     │  │    Config    │  │    Theme     │  │    Jobs      │  │
│  │  (spells)    │  │ (settings)   │  │  (colors)    │  │ (job mgr)    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  │
└──────────┬──────────────────────────────────────────────────────────────────┘
           │
     ┌─────┴─────┬─────────────┬─────────────┬─────────────┐
     │           │             │             │             │
     ▼           ▼             ▼             ▼             ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│ models  │ │archivist│ │ invoker │ │   ui    │ │validation│
│(structs)│ │(persist)│ │ (exec)  │ │(render) │ │ (check) │
└─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

## Core Struct Relationships

```
AppState (main state container)
│
├── Data Layer
│   ├── codex: Codex
│   │   ├── spells: Vec<Spell>
│   │   └── spellbooks: Vec<Spellbook>
│   ├── jobs: JobManager
│   │   └── jobs: HashMap<u64, Job>
│   └── recents: Vec<RecentEntry>
│
├── UI State
│   ├── mode: Mode
│   │   ├── BrowseSpellbooks
│   │   ├── BrowseSpells
│   │   ├── AddSpell
│   │   ├── EditSpell
│   │   └── AddSpellbook
│   ├── overlays: Vec<Overlay>
│   │   ├── OutputModal
│   │   ├── ConfirmDialog
│   │   ├── CommandPalette
│   │   └── Help
│   ├── focus: FocusTarget
│   │   ├── Main
│   │   └── JobsSidebar
│   └── Component States
│       ├── spellbook_browser: SpellbookBrowserState
│       ├── streaming_modal: StreamingModalState
│       ├── add_spell: AddSpellForm
│       ├── add_spellbook: AddSpellbookForm
│       └── confirm_dialog: Option<ConfirmDialogState>
│
└── Configuration
    ├── user_settings: UserSettings
    └── theme: RatatuiColors
```

## UI Component Hierarchy

```
UiState (root)
│
├── Mode Layer (one active at a time)
│   ├── BrowseSpellbooks ────▶ SpellbookBrowserState
│   │   └── Renders: Card grid or Spine list
│   ├── BrowseSpells ────────▶ (uses spell_list)
│   │   └── Renders: Spell list with details
│   ├── AddSpell/EditSpell ──▶ AddSpellForm
│   │   └── Renders: Form fields (Name, Command, etc.)
│   └── AddSpellbook ────────▶ AddSpellbookForm
│       └── Renders: Form fields (Name, Cover, Sigil)
│
├── Overlay Stack (0+ active, rendered on top)
│   ├── OutputModal ─────────▶ StreamingModalState
│   │   ├── Running: Shows ⟳ with Ctrl+C/B controls
│   │   └── Finished: Shows ✓/✗ with scroll controls
│   ├── ConfirmDialog ───────▶ ConfirmDialogState
│   │   └── Typed confirmation for destructive actions
│   ├── CommandPalette ──────▶ CommandPaletteState
│   │   └── Filtered command list with : prefix
│   └── Help ────────────────▶ (static content)
│
├── Sidebar (toggleable, coexists with mode)
│   └── JobsSidebarState
│       └── Renders: Job list with status icons
│
└── Global UI Elements
    ├── Loading indicator (bottom-right spinner)
    └── Footer hints (context-aware)
```

## Event Flow Architecture

```
Key Event Received
│
▼
handle_event() [events.rs]
│
├──▶ Priority 1: Active Overlays
│   ├── OutputModal ────▶ streaming_modal::handle_key()
│   ├── ConfirmDialog ──▶ confirm dialog handler
│   ├── CommandPalette ─▶ command palette handler
│   └── Help ───────────▶ pop_overlay() on Esc
│
├──▶ Priority 2: Jobs Sidebar (if focused)
│   └── jobs::handle_jobs_key()
│
├──▶ Priority 3: Global Keybinds
│   ├── Ctrl+C ─────────▶ Quit
│   ├── Alt+R ──────────▶ Reload codex
│   ├── t ──────────────▶ Cycle theme
│   ├── v ──────────────▶ Cycle view mode
│   ├── ? ──────────────▶ Push Help overlay
│   └── Tab ────────────▶ Cycle focus (if sidebar open)
│
└──▶ Priority 4: Mode Handler
    ├── BrowseSpellbooks ─▶ handle_search()
    ├── BrowseSpells ─────▶ handle_spell_list()
    ├── AddSpell ─────────▶ handle_add_spell()
    └── AddSpellbook ─────▶ handle_add_spellbook()
```

## Data Flow: Spell Execution

```
User presses 'r' (run spell)
│
▼
start_spell_execution()
│
├── Spell has confirm=true?
│   ├── YES ──▶ Push ConfirmDialog overlay ──▶ Wait for user
│   └── NO ───▶ Continue
│
└── Match run_mode
    │
    ├── Simple ────────▶ execute_simple_mode()
    │   ├── 1. Add to recents (in memory)
    │   ├── 2. CRITICAL: Save recents.toml
    │   ├── 3. Disable raw mode
    │   └── 4. exec() - process replaced, never returns
    │
    ├── TUI ───────────▶ streaming_modal::start_tui_execution()
    │   ├── Spawn process with pipes
    │   ├── Background threads read stdout/stderr
    │   ├── mpsc channel sends lines to UI
    │   ├── Event loop polls channel every 100ms
    │   └── Render real-time output
    │       ├── Ctrl+C: kill process
    │       ├── Ctrl+B: promote to background job
    │       └── Esc: close when finished
    │
    └── Background ────▶ invoker::start_spell()
        ├── Spawn detached process (nohup)
        ├── Create Job entry
        ├── Write stdout/stderr to job files
        └── Update jobs sidebar
```

## Module Responsibilities

```
┌────────────────────────────────────────────────────────────┐
│ src/main.rs                                                 │
│ • Entry point, CLI parsing                                  │
│ • Terminal setup (raw mode, keyboard enhancements)          │
│ • Main event loop (poll → handle → render)                  │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│ src/state.rs                                                │
│ • AppState - central state container                        │
│ • State management (codex, jobs, recents, config)           │
│ • CRUD operations (add_spell, delete_spell, etc.)           │
│ • Theme and settings persistence                            │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│ src/models/                                                 │
│ • Core data structures (Spell, Spellbook, Codex)            │
│ • Enums (RunMode, FocusTarget, SpellbookRef)                │
│ • Theme definitions (RatatuiColors)                         │
└────────────────────────────────────────────────────────────┐
┌────────────────────────────────────────────────────────────┐
│ src/archivist/                                              │
│ • Persistence layer                                         │
│ • Load/save codex.toml with atomic writes                   │
│ • Recent/jobs storage                                       │
│ • V1 → V2 migration logic                                   │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│ src/invoker/                                                │
│ • Process execution (Simple, TUI, Background)               │
│ • Job management (spawn, poll, kill)                        │
│ • Stream output handling (threads, channels)                │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│ src/ui/                                                     │
│ • Rendering (ratatui widgets)                               │
│ • Event handling (key dispatch)                             │
│ • Component states (forms, browsers, modals)                │
│ • Mode/Overlay management                                   │
└────────────────────────────────────────────────────────────┘
```

## State Mutation Patterns

```
┌─────────────────────────────────────────────────────────────┐
│ IMMUTABLE REFERENCES (read-only)                            │
│ • state.codex.spells.iter()                                 │
│ • state.theme (for rendering)                               │
│ • ui.view_mode                                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ MUTABLE REFERENCES (modify then redraw)                     │
│ • ui.mode = Mode::BrowseSpells                              │
│ • ui.push_overlay(Overlay::Help)                           │
│ • state.add_recent() → ui.request_redraw()                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ PERSISTENCE (async-like via archivist)                      │
│ • Archivist::save(&state.codex, path)                     │
│ • Archivist::save_recents(&state.recents)                  │
│ • Archivist::append_spellbook(path, ...)                  │
└─────────────────────────────────────────────────────────────┘
```

## Key Architectural Decisions

1. **Mode + Overlay System**: Single mode active, overlays stack on top
2. **Component State Encapsulation**: Each component owns its state struct
3. **Event Priority**: Overlay → Sidebar → Global → Mode
4. **Atomic Writes**: All persistence uses temp file + rename
5. **UUID References**: Spells referenced by ID, not name
6. **Virtual Spellbooks**: Favorites/Recent generated dynamically

## Debugging Tips Using This Diagram

1. **Lost key events?** Check event priority in `handle_event()`
2. **Rendering wrong?** Verify mode and active overlays
3. **State not persisting?** Check archivist calls
4. **Overlay not closing?** Verify `pop_overlay()` is called
5. **Focus issues?** Check `FocusTarget` and Tab handler
6. **Stream not updating?** Verify mpsc channel and polling

## File Organization Map

```
src/
├── main.rs              # Entry, event loop
├── cli.rs               # CLI arguments
├── state.rs             # AppState, CRUD
├── clipboard.rs         # System clipboard
├── logging.rs           # Log macros
├── validation.rs        # Codex validation
│
├── models/
│   ├── mod.rs           # Re-exports
│   ├── spell.rs         # Spell struct, RunMode
│   ├── spellbook.rs     # Spellbook struct
│   ├── codex.rs         # Codex container
│   ├── job.rs           # Job, JobStatus
│   └── theme.rs         # RatatuiColors, Theme
│
├── archivist/
│   ├── mod.rs           # Re-exports
│   └── archivist.rs     # All persistence logic
│
├── invoker/
│   ├── mod.rs           # Re-exports
│   └── mod.rs           # Job management, execution
│
└── ui/
    ├── mod.rs           # Mode, Overlay, UiState
    ├── render.rs        # Top-level render dispatcher
    ├── events.rs        # Event handling, priority
    ├── footer.rs        # Context-aware footer hints
    │
    ├── add_spell.rs     # Add/Edit spell form UI
    ├── add_spell_form.rs # Form state management
    ├── add_spellbook_form.rs # Spellbook form + UI
    │
    ├── confirm.rs       # ConfirmDialog rendering
    ├── help.rs          # Help overlay
    ├── jobs.rs          # Jobs sidebar
    ├── input.rs         # Input popup (legacy)
    │
    ├── search_overlay.rs # BrowseSpellbooks + Search
    ├── search_state.rs  # Search state management
    ├── spell_list.rs    # BrowseSpells list
    ├── spellbook_browser.rs # Spellbook card/spine browser
    └── streaming_modal.rs # TUI streaming output
```
