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

### SearchOverlay Modes

Within SearchOverlay, there are several modes defined by `SearchMode`:

```rust
pub enum SearchMode {
    BrowseSpellbooks, // Default - show cards/spines of spellbooks
    BrowseSpells,      // Show spells in selected spellbook
    AddSpell,          // Add new spell form
    AddSpellbook,      // Add new spellbook form
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

[ / ] from any screen → SearchOverlay (BrowseSpellbooks)
[ --add ] CLI arg    → AddSpell

In SearchOverlay:
  [Enter] on spellbook → BrowseSpells mode
  [:n] command        → AddSpell mode
  [:b] command        → BrowseSpellbooks mode
  [Esc]               → Return to previous screen
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

Primary navigation hub that consolidates browsing, searching, and adding spells.

**Layout:**
- Search input bar (with `:` command prefix support)
- Main area: Spellbooks displayed as cards or spines, OR spell list in BrowseSpells mode
- Footer: Context-aware keyboard hints

**State:**
- `search_mode`: Current mode (BrowseSpellbooks, BrowseSpells, AddSpell, AddSpellbook)
- `search_items_per_row`: Items per row (used for navigation)
- `search_showing_spellbooks`: True when browsing spellbooks
- `search_query`: Current search text
- `filtered_indices`: Matching spell indices

**Modes:**

#### BrowseSpellbooks (default)
- Displays all spellbooks as cards or spines based on view mode
- Row-based navigation: Left/Right wrap within row, Up/Down wrap within column
- Enter opens the selected spellbook
- `:` opens command bar

#### BrowseSpells
- Shows spells from the selected spellbook
- Simple list navigation with Up/Down
- Left returns to BrowseSpellbooks mode
- Enter copies the selected spell

#### AddSpell
- Form for adding new spells
- Same layout as AddSpell screen

#### AddSpellbook
- Form for adding new spellbooks

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

## View Modes

The SearchOverlay supports two view modes. Both are responsive and adapt to terminal width.

### Cards (`:c`)
Large card view displaying spellbook sigils, names, and descriptions.

### Spines (`:p`)
Compact spine (book spine) view showing just the sigils, ideal for many spellbooks.

Cycle view modes with `v` key or use commands (`:c`, `:p`).

## Theming

All screens support theming with the following color slots:
- `bg`: Background color
- `fg`: Foreground/text color
- `accent`: Accent/highlight color
- `muted`: Muted/secondary text color
- `selection`: Selection highlight color
- `border`: Border color

Available themes (cycle with `t` or `:t`):
- default, default-light, dracula, gruvbox-dark, gruvbox-light, nord, catppuccin, one-dark, solarized-dark, solarized-light

Theme preference is persisted in `theme.toml`.

## View Mode Persistence

The current view mode preference is persisted in `theme.toml` alongside the theme selection.
