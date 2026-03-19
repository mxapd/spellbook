# Spellbook Developer Notes

## Design Preferences

- **No emojis** - prefer ASCII or text characters
- Cool ASCII symbols are welcome (e.g., *, >, ::, #, etc.)

## Code Style

- Follow existing code conventions in the codebase
- Use theme colors (from `state.theme`) for all UI styling
- Keep render functions clean and readable

## Features

- Theme cycling with `t` key, persisted to `theme.toml`
- View mode cycling with `v` key (cards/spines), persisted to `theme.toml`
- `--add` CLI flag opens Add Spell screen directly
- SearchOverlay is the primary screen with modes: BrowseSpellbooks, BrowseSpells, AddSpell, AddSpellbook
- Command bar with `:` prefix for quick actions (`:n`, `:b`, `:s`, `:c`, `:p`, `:a`, `:t`, `:?`)
- Row-based navigation: Left/Right wrap within row, Up/Down wrap within column
- Ctrl+C quits, Ctrl+Z passes through (let terminal handle job control)
- No sigil sizing - Ratatui does not support font size changes

## Known Limitations

- **Ctrl+Z**: Crossterm raw mode captures Ctrl+Z before terminal can handle job control. This is a fundamental limitation - let terminal deal with it naturally by returning false.
- **Font sizing**: Ratatui does not support font size changes.

## SearchOverlay Modes

The SearchOverlay has four modes controlled by `SearchMode` enum:

```rust
pub enum SearchMode {
    BrowseSpellbooks, // Default - show cards/spines
    BrowseSpells,      // Show spells in selected spellbook
    AddSpell,          // Add new spell form
    AddSpellbook,      // Add new spellbook form
}
```

## Navigation in BrowseSpellbooks

Row-based navigation uses `search_items_per_row`:
- Left/Right wrap within the same row
- Up/Down wrap within the same column
- Enter opens spellbook in BrowseSpells mode

## View Modes

Three view modes for spellbook display:
- `ViewMode::Cards`: Large card view
- `ViewMode::Spines`: Compact spine view
- Both modes are responsive (cards when they fit, spines otherwise)

## Testing

When the linker issue is resolved, test the app with:
```bash
cargo run
```

Test key features:
1. Browse spellbooks with arrow keys
2. Enter to open a spellbook
3. Navigate spells with Up/Down
4. Enter to copy a spell
5. Type `:` to open command bar
6. Use `:n` to add a new spell
7. Cycle themes with `t`
