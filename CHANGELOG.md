# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepaloggelog.com/en/1.0.0/).

## [Unreleased]

### Completed
- **v2.0.0 Rewrite 100% Complete** - All core features implemented and tested
- Removed deprecated Screen enum, fully migrated to Mode/Overlay system
- TUI streaming execution with real-time output and 10k line cap
- Simple mode now writes recents.toml before exec() (critical fix)
- AddSpellbook form with full UI and event handling
- Loading states for all archivist operations
- Context-aware footer hints for all modes
- 100 unit tests passing

### Fixed
- Background jobs now persist correctly after app restart (missing `.spellbook` directory creation)

## [2.0.0] - 2026-03-22

### Added

#### Architecture
- Complete v2 refactor with clean separation of concerns
- Mode-based navigation (BrowseSpellbooks, BrowseSpells, AddSpell, EditSpell)
- Overlay system for modals (OutputModal, ConfirmDialog, Help)
- Atomic file writes for data integrity
- UUID-based spell IDs for stable references

#### Execution Modes
- **Simple mode** (`s`): Copy to clipboard, exit TUI
- **TUI mode** (`Ctrl+r`): Execute in modal with streamed output
- **Background mode** (`Ctrl+b`): Run as detached job, track in sidebar

#### Jobs Sidebar
- Toggle with `:jobs` command
- Real-time status icons: ⟳ (running), ✓ (completed), ✗ (failed), ⊘ (cancelled)
- View output with `Enter` or `v`
- Kill running jobs with `k`, cancel queued with `c`
- Tab key cycles focus between main and sidebar

#### Virtual Spellbooks
- **Favorites**: Dynamic collection of spells marked as favorite (`f` key)
- **Recent**: Last 100 used spells from `recents.toml`
- Both appear at top of spellbook list

#### Search & Filtering
- `/` key activates search mode
- Real-time filtering for spellbooks (by name) and spells (by name/lore/school/glyphs)
- Esc clears and deactivates search

#### Import/Export
- `:export [file]` - Export full codex to TOML
- `:export <spellbook>` - Export single spellbook
- `:import <file>` - Import with auto-merge (renames conflicts)

#### Focus Management
- `FocusTarget` enum tracks focused component (Main/JobsSidebar)
- Visual indicators (bright borders when focused)
- Tab key cycles focus when sidebar is open

#### Commands
- `:n` / `:new` - Add new spell
- `:nb` / `:new book` - Add new spellbook
- `:b` / `:browse` - Browse spellbooks
- `:s` / `:spells` - Browse spells in selected spellbook
- `:c` / `:cards` - Card view
- `:p` / `:spines` - Spine view
- `:l` / `:list` - List view
- `:t` / `:theme` - Cycle themes
- `:j` / `:jobs` - Toggle jobs sidebar
- `:?` / `:help` - Show keybind reference
- `:export` / `:import` - import/export commands

#### Spellbook Management
- Delete spellbook with `Shift+D`
- Confirmation dialog before deletion
- Prevents deletion of virtual spellbooks (Favorites/Recent)

#### Error Handling & Validation
- Working directory fallback to `$HOME` when invalid
- Graceful clipboard degradation when tool unavailable
- Invalid TOML recovery (creates empty codex)
- Startup validation warnings (broken refs, duplicates, orphans)
- Job spawn failure handling with user feedback

### Changed

#### Navigation
- SearchOverlay as primary navigation hub
- Row-based card/spine navigation with wrapping
- Spell list navigation with Up/Down

#### Persistence
- Spells referenced by UUID instead of name
- Spellbooks use `spell_ids` array instead of `spells` name array
- Automatic V1 → V2 migration on first run
- Backup created before migration

#### UI Improvements
- Context-aware footer hints
- Improved confirm dialogs with typed confirmation
- Help overlay showing all keybinds

### Fixed

- Ctrl+C properly quits application
- Ctrl+Z passes through for terminal job control
- Tests pass with serial test fixtures
- Raw string literal syntax errors resolved

## [0.1.0] - Initial Release

### Added
- Spellbook list (home screen)
- Spell list with details panel
- Search overlay with global search
- Add Spell form with dropdown
- 10 built-in themes
- Theme cycling with `t` key
- D-Bus notifications on copy
- Vim-style navigation (j/k/h/l)
- CLI argument `--add` for direct add screen
- TOML data persistence
- Theme persistence in theme.toml
