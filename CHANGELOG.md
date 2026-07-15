# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.0.0] - 2026-07-15

### Added

- TUI for managing CLI command snippets organized as spells in spellbooks.
- Three execution modes for spells:
  - **Simple** (`s`): exit TUI and run the command in the shell.
  - **TUI** (`Ctrl+r`): capture output in a streaming modal overlay.
  - **Background** (`Ctrl+b`): run as a detached job tracked in the sidebar.
- Spellbook browser with Cards and Spines view modes.
- Global search activated with `/`.
- Command palette with `:` prefix for quick actions.
- Virtual spellbooks: Favorites and Recent.
- Full CRUD operations for spells and spellbooks.
- Spell metadata: UUID-based IDs, name, incantation, lore, school, glyphs, run mode, confirmation, working directory, and favorite flag.
- 10 built-in color themes with persistence in `theme.toml`.
- Import/export of the codex as TOML.
- Background jobs sidebar with status tracking.
- D-Bus desktop notifications on job completion.
- Atomic TOML writes for data integrity.
- Vi-style keybindings (`hjkl`) and arrow-key navigation.
