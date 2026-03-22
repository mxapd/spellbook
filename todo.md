# Spellbook v2 Implementation Checklist

This is the active task list for implementing Spellbook v2. See [docs/roadmap.md](docs/roadmap.md) for detailed phase descriptions.

---

## Phase 1: Core Refactor ✓

### Models & Data
- [x] Add `id: String` (UUID) field to Spell struct
- [x] Update Spellbook to use spell IDs instead of names
- [x] Create `SpellbookRef` enum (Virtual | Codex)
- [x] Create `VirtualKind` enum (Favorites | Recent)
- [x] Create `RunMode` enum (Simple | Tui | Background)
- [x] Create `FocusTarget` enum (Main | JobsSidebar)
- [x] Create `RecentEntry` struct

### State Architecture
- [x] Create new `AppState` struct with component states
- [x] Implement `SpellbookBrowserState`
- [x] Implement `SpellBrowserState` (with SpellbookRef)
- [x] Implement `SpellFormState` (with dirty flag)
- [x] Implement `SpellbookFormState` (with dirty flag)
- [x] Implement `OutputModalState`
- [x] Implement `ConfirmDialogState`
- [x] Implement `CommandPaletteState`
- [x] Implement `JobsSidebarState`
- [x] Add `focus: FocusTarget` to AppState

### Mode & Overlay System
- [x] Create `Mode` enum (replace old Screen enum)
- [x] Create `Overlay` enum
- [ ] Implement mode transitions (pending full integration)
- [ ] Implement overlay stacking (pending full integration)
- [ ] Update render dispatcher for Mode/Overlay (pending full integration)

### Persistence
- [x] Implement atomic write pattern (write-to-temp + rename)
- [x] Update `codex_store` for UUID support
- [x] Implement V1 → V2 migration logic
- [x] Create `recent_store` module
- [x] Update all archivist modules with atomic writes

### Event Handling
- [ ] Implement event priority system (overlay → sidebar → mode → global) - pending Mode/Overlay integration
- [ ] Add focus management to event dispatcher - pending Mode/Overlay integration
- [ ] Update keybind handlers for new modes - pending Mode/Overlay integration

---

## Phase 2: Execution System ✓

### Simple Mode
- [x] Implement terminal restoration
- [x] Implement `$SHELL -c` execution
- [x] Implement process replacement with `exec()`
- [ ] Write `recents.toml` before exec
- [x] Handle `working_dir` fallback

### TUI Mode
- [x] Spawn child process with piped stdout/stderr
- [x] Create background thread for pipe reading
- [x] Implement mpsc channel to event loop
- [ ] Implement `OutputModalState` with streaming
- [ ] Add 10k line cap with truncation warning
- [ ] Implement real-time display with auto-scroll
- [ ] Implement promotion to background (Ctrl+b)

### Background Mode
- [x] Implement detached process spawn (nohup)
- [x] Create `Job` struct
- [x] Create `JobManager`
- [x] Implement job persistence to `jobs.toml`
- [x] Implement output file management
- [x] Implement job ID generation (monotonic counter)

### Job System
- [x] Create background poller thread
- [x] Implement status updates via mpsc channel
- [ ] Integrate D-Bus notifications
- [x] Implement 10 concurrent job limit
- [x] Implement job retention (50 limit, auto-purge)

### ConfirmDialog
- [x] Implement ConfirmDialog rendering
- [x] Implement ConfirmDialog event handling
- [x] Integrate with `confirm` flag in spells

---

## Phase 3: Virtual Spellbooks & Favorites ✓

### Favorites
- [x] Add `favorite: bool` to Spell
- [x] Implement `f` keybind to toggle favorite
- [x] Generate virtual Favorites spellbook
- [x] Persist favorites to `codex.toml`

### Recent Items
- [x] Create `RecentEntry` struct
- [x] Implement `recent_store` persistence
- [x] Record copy actions
- [x] Record run actions
- [x] Generate virtual Recent spellbook
- [x] Implement FIFO eviction (100 limit)

### Virtual Spellbook Integration
- [x] Render virtual spellbooks with visual distinction
- [x] Position virtual spellbooks at top of list
- [x] Update spell browser to handle `SpellbookRef`
- [x] Handle navigation with SpellbookRef

---

## Phase 4: CRUD Operations ✓

### Edit Spell
- [x] Create EditSpell mode
- [x] Reuse `SpellFormState`
- [x] Pre-populate form with existing spell data
- [x] Implement save logic (update spell in codex)
- [x] Persist changes to `codex.toml`

### Delete Spell
- [x] Implement `d` keybind
- [x] Show ConfirmDialog before delete
- [x] Remove spell from codex
- [x] Remove spell references from all spellbooks
- [x] Persist changes to `codex.toml`

### Delete Spellbook
- [ ] Implement delete keybind
- [ ] Show confirmation dialog
- [ ] Remove spellbook from codex
- [ ] Persist changes to `codex.toml`

### Unsaved Changes
- [x] Implement `dirty` flag tracking
- [x] Show ConfirmDialog on Esc when dirty
- [x] Set dirty on any field change
- [x] Clear dirty on save

---

## Phase 5: Jobs Sidebar & Focus

### Jobs Sidebar
- [x] Implement `JobsSidebarState`
- [x] Render sidebar on right side
- [x] Display status icons (⟳ ✓ ✗ ⊘)
- [x] Implement navigation (↑ ↓)
- [x] Implement `:jobs` toggle command
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
- [x] Unit tests for models
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

---

**Current Status**: Phase 4 complete, Phase 5 (Jobs Sidebar) next
**Last Updated**: 2026-03-22
