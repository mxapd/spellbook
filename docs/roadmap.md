# Roadmap

## Current Status

Core app is functional with themes, search, and add spell form. Vim-style h/l navigation still pending.

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

## Phase 3: Polish (Priority: Medium)

Improve UX and visual quality.

- [x] Error handling for missing/broken codex.toml
- [x] Data validation on load (spell references, duplicate names)
- [x] D-Bus notifications on copy
- [x] Theming system with 10 themes (default, default-light, dracula, gruvbox-dark, gruvbox-light, nord, catppuccin, one-dark, solarized-dark, solarized-light)
- [x] Theme cycling with `t` key
- [x] Theme persistence in theme.toml
- [x] Add Spell screen with form fields
- [x] Arrow key navigation in Add Spell form
- [x] Spellbook dropdown in Add Spell form
- [ ] Vim-style h/l navigation between screens
- [ ] Loading states

## Phase 4: Extended Features (Priority: Low)

Future enhancements.

- [x] Add new spells via UI (--add flag)
- [ ] Edit existing spells
- [ ] Delete spells
- [ ] Add/edit spellbooks
- [ ] Import/export functionality
- [ ] Command execution (run the copied command)
- [ ] Favorites or recent items
- [ ] Multiple data files

## Notes

- Data is stored in `codex.toml` (TOML format) - spell references are by name, not ID
- Theme preference stored in `theme.toml` (`selected_theme` index 0-9)
- Data validation is performed on load (checks for valid spell references, duplicate names, empty names)
- CLI argument `--add` opens directly to Add Spell screen
