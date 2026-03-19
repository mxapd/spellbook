# Roadmap

## Current Status

Core app is functional with themes, search, command bar, and multiple SearchOverlay modes. View modes (cards/spines) are implemented and persistent.

## Phase 1: Core Navigation (Priority: High)

Make the basic flow functional: navigate spellbooks → view spells → copy command.

- [x] Implement spell list navigation (k/j, arrows)
- [x] Show spell details in bottom panel (incantation, lore, school)
- [x] Copy incantation to clipboard with Enter key
- [x] Add back navigation (Esc from spell list)

## Phase 2: Search (Priority: High)

Add global search functionality.

- [x] Implement search overlay UI
- [x] Search across spell names, lore, and glyphs
- [x] Navigate search results with k/j
- [x] Copy from search results with Enter
- [x] Close search with Esc

## Phase 3: SearchOverlay as Primary Screen (Priority: High)

Consolidate workflow to SearchOverlay with inline browsing.

- [x] SearchOverlay displays spellbooks as primary screen
- [x] Enter opens spellbook in BrowseSpells mode
- [x] Command bar with `:` prefix for commands
- [x] Add spell list navigation in BrowseSpells mode
- [x] Row-based navigation (Left/Right wrap within row, Up/Down wrap within column)
- [x] Context-aware footer hints

## Phase 4: View Modes (Priority: Medium)

Display spellbooks in different visual styles.

- [x] Card view mode (`:c`)
- [x] Spine view mode (`:p`)
- [x] View mode cycling with `v` key
- [x] View mode persistence in theme.toml

## Phase 5: Polish (Priority: Medium)

Improve UX and visual quality.

- [x] Error handling for missing/broken codex.toml
- [x] Data validation on load (spell references, duplicate names)
- [x] D-Bus notifications on copy
- [x] Theming system with 10 themes
- [x] Theme cycling with `t` key (and `:t` command)
- [x] Theme persistence in theme.toml
- [x] Add Spell screen with form fields
- [x] Arrow key navigation in Add Spell form
- [x] Spellbook dropdown in Add Spell form
- [x] Loading states (for data persistence)
- [x] View mode persistence

## Phase 6: Extended Features (Priority: Low)

Future enhancements.

- [x] Add new spells via UI (`--add` flag)
- [x] Add new spellbooks via UI
- [ ] Edit existing spells
- [ ] Delete spells
- [ ] Import/export functionality
- [ ] Command execution (run the copied command)
- [ ] Favorites or recent items
- [ ] Multiple data files

## Notes

- Data is stored in `codex.toml` (TOML format) - spell references are by name, not ID
- Theme and view mode preferences stored in `theme.toml`
- Data validation is performed on load (checks for valid spell references, duplicate names, empty names)
- CLI argument `--add` opens directly to Add Spell screen
- SearchOverlay is the primary navigation screen (not the old SpellbookList/SpellList flow)
- Row-based navigation: Left/Right wrap within row, Up/Down wrap within column
