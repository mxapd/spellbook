# Spellbook Developer Notes

## Design Preferences

- **No emojis** - prefer ASCII or text characters
- Cool ASCII symbols are welcome (e.g., ✦, ⚡, ⌬, ▲, ▼, etc.)

## Code Style

- Follow existing code conventions in the codebase
- Use theme colors (from `state.theme`) for all UI styling
- Keep render functions clean and readable

## Features

- Theme cycling with `t` key, persisted to `theme.toml`
- `--add` CLI flag opens Add Spell screen directly
