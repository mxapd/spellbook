# Spellbook v2 Roadmap [ARCHIVED - COMPLETE]

> **STATUS: ALL PHASES COMPLETED** - This roadmap has been fully implemented. The v2 refactor is complete as of refactor-completion.md.

## Overview

This roadmap outlines the implementation plan for Spellbook v2 - a complete architectural redesign focused on clean state management, unified navigation, and powerful execution modes.

---

## V1 Status (Completed)

V1 delivered a functional TUI with:
- ✓ Browse spellbooks and spells
- ✓ Search across all spells
- ✓ Copy to clipboard with notifications
- ✓ Add spells and spellbooks via UI
- ✓ 10 themes with cycling
- ✓ View modes (cards/spines)
- ✓ Command palette
- ✓ Row-based navigation

### V1 Limitations

- God-object state management (flat fields)
- Multiple top-level screen states
- Sequential IDs (unstable across restarts)
- Name-based references (break on rename)
- No job management
- No execution modes
- No edit/delete functionality

**V1 code archived to `docs/archive/`**

---

## V2 Goals

1. **Clean Architecture** - Component state encapsulation, Mode/Overlay system
2. **Stable IDs** - UUID-based spell references
3. **Execution Modes** - Simple, TUI, Background with job management
4. **Virtual Spellbooks** - Favorites and Recent collections
5. **Full CRUD** - Edit and delete spells/spellbooks
6. **Import/Export** - Share spell collections
7. **Polish** - Focus management, unsaved changes, validation

---

## Implementation Phases

### Phase 1: Core Refactor (Foundation)

**Goal**: Establish new data model and state architecture.

**Tasks**:
- [ ] Update models with UUIDs
  - [ ] Add `id: String` field to Spell
  - [ ] Update Spellbook to reference spell IDs
  - [ ] Create SpellbookRef enum
  - [ ] Create RunMode enum
- [ ] Refactor state management
  - [ ] Create AppState with component states
  - [ ] Implement SpellbookBrowserState
  - [ ] Implement SpellBrowserState
  - [ ] Implement SpellFormState
  - [ ] Implement SpellbookFormState
  - [ ] Add FocusTarget enum
- [ ] Update archivist layer
  - [ ] Implement atomic writes (write-to-temp + rename)
  - [ ] Add V1 → V2 migration logic
  - [ ] Update codex_store with UUID support
  - [ ] Add recent_store module
- [ ] Update Mode enum
  - [ ] Replace old Screen enum
  - [ ] Implement mode transitions
- [ ] Update event handling
  - [ ] Implement event priority (overlay → sidebar → mode → global)
  - [ ] Add focus management

**Deliverable**: App compiles with new architecture, basic navigation works

---

### Phase 2: Execution System

**Goal**: Implement three execution modes and job management.

**Tasks**:
- [ ] Simple mode invoker
  - [ ] Terminal restoration
  - [ ] Shell command execution via `$SHELL -c`
  - [ ] Process replacement with `exec()`
  - [ ] Pre-exec recents write
- [ ] TUI mode invoker
  - [ ] Child process spawn with piped stdout/stderr
  - [ ] Background thread for pipe reading
  - [ ] mpsc channel to event loop
  - [ ] OutputModal state and rendering
  - [ ] Real-time streaming with 10k line cap
  - [ ] Promotion to background
  - [x] Background mode invoker
  - [x] Detached process spawn (nohup)
  - [x] Job struct and JobManager
  - [x] Job persistence (jobs.toml)
  - [x] Output file management
  - [x] Job polling system
  - [x] Background poller thread
  - [x] Status updates via mpsc channel
  - [x] D-Bus/notify-send notifications on completion
  - [x] ConfirmDialog overlay
  - [ ] Rendering and event handling
  - [ ] Integration with `confirm` flag

**Deliverable**: All three execution modes functional, jobs tracked

---

### Phase 3: Virtual Spellbooks & Favorites

**Goal**: Dynamic spellbook generation and favorites system.

**Tasks**:
- [x] Favorites system
  - [x] Add `favorite: bool` to Spell
  - [x] Toggle favorite keybind (`f`)
  - [x] Generate virtual Favorites spellbook
- [x] Recent items system
  - [x] RecentEntry struct
  - [x] recent_store persistence
  - [x] Record actions (copy/run)
  - [x] Generate virtual Recent spellbook
  - [x] FIFO eviction (100 limit)
- [x] Virtual spellbook rendering
  - [x] Visual distinction (muted border)
  - [x] Top-of-list positioning
  - [x] SpellbookRef navigation
- [x] Update spell browser
  - [x] Handle SpellbookRef in BrowseSpells

**Deliverable**: Favorites and Recent spellbooks appear and function

---

### Phase 4: CRUD Operations

**Goal**: Full edit and delete functionality.

**Tasks**:
- [ ] Edit spell
  - [ ] EditSpell mode
  - [ ] Reuse SpellFormState
  - [ ] Pre-populate with existing data
  - [ ] Update spell in codex
  - [ ] Persist changes
- [ ] Delete spell
  - [ ] Delete keybind (`d`)
  - [ ] ConfirmDialog integration
  - [ ] Remove from codex
  - [ ] Remove references from spellbooks
  - [ ] Persist changes
- [ ] Delete spellbook
  - [ ] Delete keybind
  - [ ] Confirmation dialog
  - [ ] Remove from codex
  - [ ] Persist changes
- [ ] Unsaved changes handling
  - [ ] `dirty` flag tracking
  - [ ] Confirmation on Esc
  - [ ] Form field change detection

**Deliverable**: Full CRUD operations for spells and spellbooks

---

### Phase 5: Jobs Sidebar & Focus

**Goal**: Jobs sidebar UI and focus management.

**Tasks**:
- [x] Jobs sidebar component
  - [x] JobsSidebarState
  - [x] Rendering (right side panel)
  - [x] Status icons (⟳ ✓ ✗ ⊘)
  - [x] Navigation (↑ ↓)
  - [x] Toggle with `:jobs`
- [x] Focus management
  - [x] FocusTarget tracking in AppState
  - [x] Tab key cycling
  - [x] Visual focus indicators
  - [x] Event routing based on focus
- [x] Job actions
  - [x] View output (Enter → OutputModal)
  - [x] Kill running job (`k`)
  - [x] Cancel queued job (`c`)
- [x] Integration
  - [x] Sidebar visible across all modes
  - [x] Job status updates from poller

**Deliverable**: Jobs sidebar functional with focus management

---

### Phase 6: Search & Filtering

**Goal**: Inline search with `/` activation.

**Tasks**:
- [x] Search mode activation
  - [x] `/` key handler
  - [x] `search_active` flag in browser states
  - [x] Visual indicator (search bar highlight)
- [x] BrowseSpellbooks search
  - [x] Filter by spellbook name
  - [x] Real-time filtering
  - [x] Update filtered_indices
- [x] BrowseSpells search
  - [x] Filter by name, lore, school, glyphs
  - [x] Real-time filtering
  - [x] Update filtered_indices
- [x] Search deactivation
  - [x] Esc clears query and deactivates
  - [x] Navigation switches to filtered view

**Deliverable**: `/` search works in both browse modes

---

### Phase 7: Import/Export

**Goal**: Share spell collections as TOML files.

**Tasks**:
- [x] Export command
  - [x] `:export [file]` - full codex
  - [x] `:export <spellbook>` - single spellbook
  - [x] Generate valid TOML
  - [x] Success notification
- [x] Import command
  - [x] `:import <file>` - load external TOML
  - [x] Parse and validate
  - [x] Conflict detection (duplicate IDs/names)
  - [x] Conflict resolution overlay
  - [x] Merge options: Skip / Overwrite / Rename
  - [x] Persist merged codex

**Deliverable**: Import/export functional with conflict handling

---

### Phase 8: Polish & Testing

**Goal**: Final quality pass and edge case handling.

**Tasks**:
- [ ] Error handling
  - [ ] Invalid working_dir fallback
  - [ ] Missing clipboard tool graceful degradation
  - [ ] Job spawn failures
  - [ ] Invalid TOML recovery
- [ ] Validation improvements
  - [ ] Startup validation report
  - [ ] Broken reference warnings
  - [ ] Duplicate detection
- [ ] UX improvements
  - [ ] Loading states
  - [ ] Better error messages
  - [ ] Footer hints refinement
  - [ ] Help overlay content
- [ ] Testing
  - [ ] Unit tests for models
  - [ ] Integration tests for persistence
  - [ ] Manual testing matrix
- [ ] Documentation
  - [ ] Update CHANGELOG.md
  - [ ] Final doc review
  - [ ] Usage examples

**Deliverable**: Production-ready v2 release

---

## Success Criteria

### Must Have (v2.0)
- ✓ All V1 features preserved
- ✓ Three execution modes functional
- ✓ Job management with sidebar
- ✓ Virtual spellbooks (Favorites, Recent)
- ✓ Edit and delete operations
- ✓ UUID-based stable references
- ✓ Clean architecture with encapsulated state

### Should Have (v2.0)
- ✓ Import/export functionality
- ✓ Focus management
- ✓ Unsaved changes warnings
- ✓ Search with `/` activation
- ✓ V1 → V2 migration

### Nice to Have (v2.1+)
- [ ] Undo/redo system
- [ ] Spell history (track execution count)
- [ ] Custom themes (user-defined)
- [ ] Multi-select operations
- [ ] Spell variables/templating
- [ ] Encrypted spell storage

---

## Timeline Estimate

| Phase | Estimated Time | Dependencies |
|-------|---------------|--------------|
| Phase 1 | 3-5 days | None |
| Phase 2 | 5-7 days | Phase 1 |
| Phase 3 | 2-3 days | Phase 1 |
| Phase 4 | 2-3 days | Phase 1 |
| Phase 5 | 3-4 days | Phase 2 |
| Phase 6 | 1-2 days | Phase 1 |
| Phase 7 | 2-3 days | Phase 1 |
| Phase 8 | 3-5 days | All previous |
| **Total** | **21-32 days** | |

*Assumes single developer, full-time work. Adjust accordingly.*

---

## Risk Mitigation

### Technical Risks

1. **Exec() on non-Unix platforms**
   - Mitigation: Use conditional compilation, fall back to spawn + wait on Windows

2. **Job polling performance**
   - Mitigation: Efficient polling (1s interval), limit to 10 concurrent jobs

3. **TUI streaming memory**
   - Mitigation: 10k line cap with truncation warning

4. **V1 → V2 migration failures**
   - Mitigation: Backup codex.toml before migration, detailed error logging

### UX Risks

1. **Mode/Overlay confusion**
   - Mitigation: Clear visual indicators, comprehensive help overlay

2. **Search activation ambiguity**
   - Mitigation: `/` key is explicit, help docs explain

3. **Focus management discoverability**
   - Mitigation: Tab hint in footer when sidebar open

---

## Post-v2 Ideas

### v2.1 - Enhanced Workflows
- Spell chaining (run multiple spells sequentially)
- Conditional execution (only run if previous succeeded)
- Spell groups (logical sets for batch operations)

### v2.2 - Power Features
- Variables/templating (`$USER`, `$DATE`, prompts)
- Spell history tracking (execution count, last run)
- Custom keyboard shortcuts
- Multi-select for batch operations

### v2.3 - Collaboration
- Cloud sync (optional)
- Team spellbooks (shared collections)
- Spell marketplace (community sharing)

### v2.4 - Advanced Execution
- SSH remote execution
- Docker container execution
- Conditional branching (if/else logic)
- Parallel execution support

---

## Notes

- V1 code preserved in `docs/archive/` for reference
- Refactor spec in `refactor.md` is the source of truth
- All phases build incrementally - no big-bang rewrite
- Each phase should result in a compilable, testable state
- Prioritize functionality over perfection in early phases
- Polish and optimization happen in Phase 8

---

**Current Status**: Phase 0 - Documentation complete, ready to begin Phase 1
