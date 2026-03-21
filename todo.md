# Spellbook v2 Implementation Checklist

This is the active task list for implementing Spellbook v2. See [docs/roadmap.md](docs/roadmap.md) for detailed phase descriptions.

---

## Phase 1: Core Refactor ⏳

### Models & Data
- [ ] Add `id: String` (UUID) field to Spell struct
- [ ] Update Spellbook to use spell IDs instead of names
- [ ] Create `SpellbookRef` enum (Virtual | Codex)
- [ ] Create `VirtualKind` enum (Favorites | Recent)
- [ ] Create `RunMode` enum (Simple | Tui | Background)
- [ ] Create `FocusTarget` enum (Main | JobsSidebar)
- [ ] Create `RecentEntry` struct

### State Architecture
- [ ] Create new `AppState` struct with component states
- [ ] Implement `SpellbookBrowserState`
- [ ] Implement `SpellBrowserState` (with SpellbookRef)
- [ ] Implement `SpellFormState` (with dirty flag)
- [ ] Implement `SpellbookFormState` (with dirty flag)
- [ ] Implement `OutputModalState`
- [ ] Implement `ConfirmDialogState`
- [ ] Implement `CommandPaletteState`
- [ ] Implement `JobsSidebarState`
- [ ] Add `focus: FocusTarget` to AppState

### Mode & Overlay System
- [ ] Create `Mode` enum (replace old Screen enum)
- [ ] Create `Overlay` enum
- [ ] Implement mode transitions
- [ ] Implement overlay stacking
- [ ] Update render dispatcher for Mode/Overlay

### Persistence
- [ ] Implement atomic write pattern (write-to-temp + rename)
- [ ] Update `codex_store` for UUID support
- [ ] Implement V1 → V2 migration logic
- [ ] Create `recent_store` module
- [ ] Update all archivist modules with atomic writes

### Event Handling
- [ ] Implement event priority system (overlay → sidebar → mode → global)
- [ ] Add focus management to event dispatcher
- [ ] Update keybind handlers for new modes

---

## Phase 2: Execution System

### Simple Mode
- [ ] Implement terminal restoration
- [ ] Implement `$SHELL -c` execution
- [ ] Implement process replacement with `exec()`
- [ ] Write `recents.toml` before exec
- [ ] Handle `working_dir` fallback

### TUI Mode
- [ ] Spawn child process with piped stdout/stderr
- [ ] Create background thread for pipe reading
- [ ] Implement mpsc channel to event loop
- [ ] Implement `OutputModalState` with streaming
- [ ] Add 10k line cap with truncation warning
- [ ] Implement real-time display with auto-scroll
- [ ] Implement promotion to background (Ctrl+b)

### Background Mode
- [ ] Implement detached process spawn (nohup)
- [ ] Create `Job` struct
- [ ] Create `JobManager`
- [ ] Implement job persistence to `jobs.toml`
- [ ] Implement output file management
- [ ] Implement job ID generation (monotonic counter)

### Job System
- [ ] Create background poller thread
- [ ] Implement status updates via mpsc channel
- [ ] Integrate D-Bus notifications
- [ ] Implement 10 concurrent job limit
- [ ] Implement job retention (50 limit, auto-purge)

### ConfirmDialog
- [ ] Implement ConfirmDialog rendering
- [ ] Implement ConfirmDialog event handling
- [ ] Integrate with `confirm` flag in spells

---

## Phase 3: Virtual Spellbooks & Favorites

### Favorites
- [ ] Add `favorite: bool` to Spell
- [ ] Implement `f` keybind to toggle favorite
- [ ] Generate virtual Favorites spellbook
- [ ] Persist favorites to `codex.toml`

### Recent Items
- [ ] Create `RecentEntry` struct
- [ ] Implement `recent_store` persistence
- [ ] Record copy actions
- [ ] Record run actions
- [ ] Generate virtual Recent spellbook
- [ ] Implement FIFO eviction (100 limit)

### Virtual Spellbook Integration
- [ ] Render virtual spellbooks with visual distinction
- [ ] Position virtual spellbooks at top of list
- [ ] Update spell browser to handle `SpellbookRef`
- [ ] Handle navigation with SpellbookRef

---

## Phase 4: CRUD Operations

### Edit Spell
- [ ] Create EditSpell mode
- [ ] Reuse `SpellFormState`
- [ ] Pre-populate form with existing spell data
- [ ] Implement save logic (update spell in codex)
- [ ] Persist changes to `codex.toml`

### Delete Spell
- [ ] Implement `d` keybind
- [ ] Show ConfirmDialog before delete
- [ ] Remove spell from codex
- [ ] Remove spell references from all spellbooks
- [ ] Persist changes to `codex.toml`

### Delete Spellbook
- [ ] Implement delete keybind
- [ ] Show confirmation dialog
- [ ] Remove spellbook from codex
- [ ] Persist changes to `codex.toml`

### Unsaved Changes
- [ ] Implement `dirty` flag tracking
- [ ] Show ConfirmDialog on Esc when dirty
- [ ] Set dirty on any field change
- [ ] Clear dirty on save

---

## Phase 5: Jobs Sidebar & Focus

### Jobs Sidebar
- [ ] Implement `JobsSidebarState`
- [ ] Render sidebar on right side
- [ ] Display status icons (⟳ ✓ ✗ ⊘)
- [ ] Implement navigation (↑ ↓)
- [ ] Implement `:jobs` toggle command
- [ ] Integrate job status updates from poller

### Focus Management
- [ ] Implement `FocusTarget` tracking
- [ ] Implement Tab key cycling
- [ ] Add visual focus indicators
- [ ] Route events based on focus

### Job Actions
- [ ] Implement Enter to view output (OutputModal)
- [ ] Implement `k` to kill running job
- [ ] Implement `c` to cancel queued job

---

## Phase 6: Search & Filtering

### Search Activation
- [ ] Implement `/` key handler
- [ ] Add `search_active` flag to browser states
- [ ] Add visual indicator (search bar highlight)

### BrowseSpellbooks Search
- [ ] Filter by spellbook name
- [ ] Implement real-time filtering
- [ ] Update `filtered_indices`

### BrowseSpells Search
- [ ] Filter by name, lore, school, glyphs
- [ ] Implement real-time filtering
- [ ] Update `filtered_indices`

### Search Deactivation
- [ ] Clear query on Esc
- [ ] Deactivate search mode on Esc

---

## Phase 7: Import/Export

### Export
- [ ] Implement `:export [file]` for full codex
- [ ] Implement `:export <spellbook>` for single spellbook
- [ ] Generate valid TOML output
- [ ] Show success notification

### Import
- [ ] Implement `:import <file>` command
- [ ] Parse and validate external TOML
- [ ] Detect conflicts (duplicate IDs/names)
- [ ] Create conflict resolution overlay
- [ ] Implement merge options (Skip / Overwrite / Rename)
- [ ] Persist merged codex

---

## Phase 8: Polish & Testing

### Error Handling
- [ ] Invalid `working_dir` fallback to `$HOME`
- [ ] Graceful degradation when clipboard tool missing
- [ ] Handle job spawn failures
- [ ] Recover from invalid TOML

### Validation
- [ ] Startup validation report
- [ ] Broken reference warnings
- [ ] Duplicate ID detection
- [ ] Required field validation

### UX Improvements
- [ ] Loading states for archivist operations
- [ ] Better error messages
- [ ] Refine footer hints for all modes
- [ ] Complete Help overlay content

### Testing
- [ ] Unit tests for models
- [ ] Integration tests for persistence
- [ ] Manual testing matrix (see AGENTS.md)
- [ ] Test V1 → V2 migration

### Documentation
- [ ] Update CHANGELOG.md with v2 release
- [ ] Final review of all docs
- [ ] Add usage examples

---

## Future (v2.1+)

- [ ] Undo/redo system
- [ ] Spell execution count tracking
- [ ] Custom user-defined themes
- [ ] Multi-select operations
- [ ] Spell variables/templating
- [ ] Encrypted spell storage

---

**Current Status**: Documentation complete, ready to begin Phase 1
**Last Updated**: 2026-03-21
