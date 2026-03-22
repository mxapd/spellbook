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
- [x] Integrate job status updates from poller

### Focus Management
- [x] Implement `FocusTarget` tracking
- [x] Implement Tab key cycling
- [x] Add visual focus indicators
- [x] Route events based on focus

### Job Actions
- [x] Implement Enter to view output (OutputModal)
- [x] Implement `k` to kill running job
- [x] Implement `c` to cancel queued job

---

## Phase 6: Search & Filtering

### Search Activation
- [x] Implement `/` key handler
- [x] Add `search_active` flag in SearchState
- [x] Add visual indicator (search bar highlight)

### BrowseSpellbooks Search
- [x] Filter by spellbook name
- [x] Implement real-time filtering
- [x] Update `filtered_spellbook_indices`

### BrowseSpells Search
- [x] Filter by name, lore, school, glyphs
- [x] Implement real-time filtering
- [x] Update `filtered_indices`

### Search Deactivation
- [x] Clear query on Esc
- [x] Deactivate search mode on Esc

---

## Phase 7: Import/Export

### Export
- [x] Implement `:export [file]` for full codex
- [x] Implement `:export <spellbook>` for single spellbook
- [x] Generate valid TOML output
- [x] Show success notification

### Import
- [x] Implement `:import <file>` command
- [x] Parse and validate external TOML
- [x] Detect conflicts (duplicate IDs/names)
- [x] Auto-merge with Rename strategy (simplified - no overlay needed)
- [x] Persist merged codex

---

## Phase 8: Polish & Testing

### Error Handling
- [x] Invalid `working_dir` fallback to `$HOME`
- [x] Graceful degradation when clipboard tool missing
- [x] Handle job spawn failures
- [x] Recover from invalid TOML

### Validation
- [x] Startup validation report
- [x] Broken reference warnings
- [x] Duplicate ID detection
- [x] Required field validation

### UX Improvements
- [ ] Loading states for archivist operations
- [x] Better error messages
- [ ] Refine footer hints for all modes
- [x] Complete Help overlay content

### Testing
- [x] Unit tests for models
- [ ] Integration tests for persistence
- [ ] Manual testing matrix (see AGENTS.md)
- [ ] Test V1 → V2 migration

### Documentation
- [x] Update CHANGELOG.md with v2 release
- [ ] Final review of all docs
- [ ] Add usage examples

---

**Current Status**: Phase 8 complete, ready for final testing
**Last Updated**: 2026-03-22
