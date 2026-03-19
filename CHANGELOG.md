# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added

#### SearchOverlay as Primary Screen
- SearchOverlay now serves as the primary navigation hub
- Spellbooks displayed as cards or spines directly in SearchOverlay
- Command palette with `:` prefix for quick actions

#### Command Palette
- Type `:` to show filterable command list
- Navigate with `↑`/`↓`, execute with `Enter`
- Commands defined with aliases for flexible matching (e.g., `:n`, `:new`, `:new spell` all work)
- `Esc` cancels and returns to normal mode

#### View Modes
- Two view modes: Cards, Spines (both responsive)
- View mode cycling with `v` key
- View mode persistence in theme.toml

#### BrowseSpells Mode
- Inline spell list when a spellbook is selected
- Up/Down navigation through spells
- Left returns to spellbook browsing
- Enter copies selected spell

#### Row-Based Navigation
- Left/Right wrap within the same row
- Up/Down wrap within the same column
- Unified `search_items_per_row` for consistent navigation

### Changed

#### UI Changes
- Footer hints are context-aware based on current mode
- Removed `t theme` from footer (now use `:t` command)
- Updated keybindings documentation

### Fixed

- Ctrl+C properly quits the application
- Ctrl+Z passes through to terminal for job control

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
