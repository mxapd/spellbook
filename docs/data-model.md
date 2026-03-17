# Data Model

## Codex

Root data structure containing all spells and spellbooks.

## Spell

A single command snippet.

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Display name (e.g., "List Processes") |
| `incantation` | String | The actual command(s) to run |
| `lore` | String | Description/usage notes |
| `school` | String | Category (e.g., "System", "Network") |
| `glyphs` | Vec<String> | Search tags for filtering |

**Note:** The `id` field is generated automatically on load (1, 2, 3...).

### Example (TOML)

```toml
[[spells]]
name = "List Processes"
incantation = "ps aux"
lore = "Shows all running processes."
school = "System"
glyphs = ["process", "running", "ps", "list"]
```

## Spellbook

A named collection of spells.

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Display name |
| `cover` | String | Description/purpose |
| `sigil` | String | Emoji or symbol for visual flair |
| `spells` | Vec<String> | References to spell names |

### Example (TOML)

```toml
[[spellbooks]]
name = "System"
cover = "System monitoring commands."
sigil = "💻"
spells = ["List Processes", "Kill Process"]
```

## Notes

- Spell references in spellbooks are by **name**, not by ID
- IDs are generated automatically when the file is loaded
- Data is stored in `codex.toml` (TOML format)
- Theme preference stored in `theme.toml` (`selected_theme` index 0-9)

## Theme Configuration

Stored in `theme.toml`:

```toml
# Theme index: 0=default, 1=default-light, 2=dracula, etc.
selected_theme = 0
```

Each theme defines 16-color ANSI indices:
- bg, fg, accent, muted, selection, border
