# UI Screens

## Screen States

The app has four main screens defined in `src/ui/mod.rs`:

```rust
pub enum Screen {
    SpellbookList,           // Home: list all spellbooks
    SpellList,               // Show spells in selected spellbook
    SearchOverlay { return_to: SearchContext },  // Search modal
    AddSpell,                // Add new spell form
}
```

## Navigation Flow

```
┌──────────────────┐
│  SpellbookList   │  ← Enter to select
│  (Home)          │
└────────┬─────────┘
         │ esc to go back
         ▼
┌──────────────────┐
│    SpellList     │  ← Enter to copy spell to clipboard
│  (Details below)  │
└──────────────────┘

[ / ] from any screen → SearchOverlay
[ --add ] CLI arg    → AddSpell
```

## Screen Details

### 1. SpellbookList

Displays all spellbooks in a list.

**Layout:**
- Main area: List of spellbook names with Block::bordered
- Footer: Keyboard hints

**State:**
- `spellbook_list_state`: Selected spellbook index

### 2. SpellList

Displays spells within a selected spellbook, plus details panel.

**Layout:**
- Top (60%): List of spells in current spellbook with Block::bordered
- Bottom (40%): Details panel showing selected spell info (incantation, school, glyphs, lore)
- Footer: Keyboard hints

**State:**
- `spell_list_state`: Selected spell index
- `selected_spellbook`: Which spellbook is being viewed

### 3. SearchOverlay

Modal overlay for searching across all spells.

**Layout:**
- Search input field (3 lines)
- Filtered results list
- Condensed details of selected spell
- Footer: Keyboard hints

**State:**
- `search_query`: Current search text
- `filtered_indices`: Matching spell indices
- `return_to`: Which screen to return to after search

### 4. AddSpell

Form for adding a new spell to the codex.

**Layout:**
- Main area: Form with Block::bordered containing:
  - Name field
  - Command field
  - Lore field
  - School field
  - Tags field
  - Spellbook dropdown (shows when field is active)
- Footer: Keyboard hints

**Fields:**
| Field | Icon | Description |
|-------|------|-------------|
| Name | `*` | Spell name |
| Command | `>` | CLI command/incantation |
| Lore | `::` | Description or notes |
| School | `^` | Category |
| Tags | `#` | Comma-separated tags |
| Spellbook | `>` | Dropdown to select spellbook |

**Input Style:**
- Active field: highlighted with selection color background
- Inactive field: normal text with `[value]` brackets
- Empty field: `[...]` placeholder

**Dropdown:**
- Shows when Spellbook field is active
- Lists all spellbooks plus "Skip - just create spell"
- Arrow keys navigate, Enter confirms

**State:**
- `add_spell_field`: Current active field
- `add_spell_name`, `add_spell_command`, `add_spell_lore`, `add_spell_school`, `add_spell_tags`: Form values
- `add_spell_spellbook`: Selected spellbook index
- `add_spell_skip_spellbook`: Whether to skip adding to spellbook
- `add_spell_dropdown_index`: Current dropdown selection

## Theming

All screens support theming with the following color slots:
- `bg`: Background color
- `fg`: Foreground/text color
- `accent`: Accent/highlight color
- `muted`: Muted/secondary text color
- `selection`: Selection highlight color
- `border`: Border color

Available themes (cycle with `t`):
- default, default-light, dracula, gruvbox-dark, gruvbox-light, nord, catppuccin, one-dark, solarized-dark, solarized-light

Theme preference is persisted in `theme.toml`.
