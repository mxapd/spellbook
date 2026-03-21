# UI Screens (v2)

## Overview

Spellbook v2 uses a **single-mode navigation system** with overlays, not multiple top-level screens. The application always has one active `Mode` and zero or more `Overlay`s rendered on top.

---

## Mode Enum

The primary navigation state.

```rust
pub enum Mode {
    BrowseSpellbooks,   // Home - card/spine view of spellbooks
    BrowseSpells,       // Spells inside a selected spellbook
    AddSpell,           // Form to add a new spell
    EditSpell,          // Form to edit an existing spell
    AddSpellbook,       // Form to add a new spellbook
}
```

### Mode Transitions

```
BrowseSpellbooks
    │
    ├─ Enter on spellbook ──> BrowseSpells
    │                              │
    ├─ :n command ──────────> AddSpell
    │                              │
    └─ :nb command ─────────> AddSpellbook
                                   │
                         Esc <─────┘
```

---

## Overlay Enum

Overlays render on top of the current mode.

```rust
pub enum Overlay {
    OutputModal,        // Scrollable command output viewer
    ConfirmDialog,      // "Are you sure?" confirmation
    CommandPalette,     // : command input with filtered list
    Help,               // ? keybind reference
}
```

### Overlay Behavior

- Multiple overlays can be active (stacked)
- Overlays receive events first (highest priority)
- Esc closes the topmost overlay
- Overlays render centered on top of current mode

---

## Jobs Sidebar

**Important**: The jobs sidebar is **NOT** an overlay or mode. It's a **toggleable panel** that coexists with any mode.

- Toggle with `:jobs` command
- Appears on right side of screen
- Visible across all modes when toggled on
- Has its own focus state (`FocusTarget::JobsSidebar`)
- Does not block interaction with main content
- Tab key cycles focus between main content and sidebar

---

## Layout

### Standard Layout (No Sidebar)

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│                                                     │
│             Main Content Area                       │
│       (mode-dependent: cards, list, form)           │
│                                                     │
│                                                     │
│                                                     │
├─────────────────────────────────────────────────────┤
│  Footer: context-aware hints                        │
└─────────────────────────────────────────────────────┘
```

### Layout with Jobs Sidebar

```
┌─────────────────────────────────────┬───────────────┐
│                                     │ Jobs Sidebar  │
│                                     │               │
│      Main Content Area              │  ⟳ Rebuild    │
│  (mode-dependent: cards, list,      │  ✓ Scan       │
│   form, etc.)                       │  ✗ Deploy     │
│                                     │               │
│                                     │               │
├─────────────────────────────────────┴───────────────┤
│  Footer: context-aware hints                        │
└─────────────────────────────────────────────────────┘
```

### Layout with Overlay

```
┌─────────────────────────────────────┬───────────────┐
│          ┌─────────────────┐        │ Jobs Sidebar  │
│          │  Output Modal   │        │               │
│          │  (scrollable)   │        │  ⟳ Rebuild    │
│          │                 │        │  ✓ Scan       │
│          │  > output here  │        │  ✗ Deploy     │
│          │                 │        │               │
│          └─────────────────┘        │               │
├─────────────────────────────────────┴───────────────┤
│  Footer: context-aware hints                        │
└─────────────────────────────────────────────────────┘
```

---

## Mode Details

### 1. BrowseSpellbooks

**Purpose**: Home screen showing all spellbooks (virtual + codex).

**Layout**:
- Main area: Spellbook cards or spines (based on view mode)
- Footer: Keyboard hints

**State** (`SpellbookBrowserState`):
```rust
pub struct SpellbookBrowserState {
    pub selected_index: usize,
    pub items_per_row: usize,
    pub search_active: bool,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
}
```

**Navigation**:
- Row-based navigation: ← → wrap within row, ↑ ↓ wrap within column
- Enter: Open selected spellbook (enter BrowseSpells mode)
- `/`: Activate search mode
- `:`: Open command palette

**Virtual Spellbooks**:
- Favorites appears first (if any favorites exist)
- Recent appears second (if any recents exist)
- Codex spellbooks follow

**View Modes**:
- **Cards**: Large card view with sigils, names, descriptions
- **Spines**: Compact book spine view
- **Auto**: Responsive (cards on wide terminals, spines on narrow)

---

### 2. BrowseSpells

**Purpose**: Show spells from a selected spellbook.

**Layout**:
- Main area: List of spells with details panel
- Top: Spell list (name, school, glyphs)
- Bottom: Selected spell details (incantation, lore)
- Footer: Keyboard hints

**State** (`SpellBrowserState`):
```rust
pub struct SpellBrowserState {
    pub spellbook: SpellbookRef,  // Which spellbook (virtual or codex)
    pub selected_index: usize,
    pub search_active: bool,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
}
```

**Actions**:
- Enter: Copy incantation to clipboard
- `r`: Run with spell's default mode
- `s`: Force simple run
- `Ctrl+r`: Force TUI run
- `Ctrl+b`: Force background run
- `e`: Edit selected spell
- `d`: Delete selected spell (with confirmation)
- `f`: Toggle favorite
- `/`: Activate search
- Esc or ←: Back to BrowseSpellbooks

**Search**:
- Filters by spell name, lore, school, and glyphs
- Real-time filtering as you type
- Esc clears search and deactivates search mode

---

### 3. AddSpell

**Purpose**: Form to add a new spell to the codex.

**Layout**:
- Main area: Form with fields
- Footer: Keyboard hints

**State** (`SpellFormState`):
```rust
pub struct SpellFormState {
    pub active_field: SpellFormField,
    pub name: String,
    pub incantation: String,
    pub lore: String,
    pub school: String,
    pub glyphs: String,          // comma-separated
    pub run_mode: RunMode,
    pub confirm: bool,
    pub working_dir: String,
    pub spellbook_index: Option<usize>,
    pub dropdown_open: bool,
    pub dropdown_index: usize,
    pub editing_spell_id: Option<SpellId>, // None for AddSpell
    pub dirty: bool,
}
```

**Fields**:
| Field | Icon | Description |
|-------|------|-------------|
| Name | `*` | Spell name |
| Incantation | `>` | CLI command |
| Lore | `::` | Description |
| School | `^` | Category |
| Glyphs | `#` | Tags (comma-separated) |
| Run Mode | `⚡` | Default execution mode (simple/tui/background) |
| Confirm | `?` | Require confirmation (checkbox) |
| Working Dir | `/` | Working directory (optional) |
| Spellbook | `>` | Dropdown to select spellbook |

**Navigation**:
- Tab / Arrow keys: Move between fields
- Enter on spellbook field: Toggle dropdown
- Enter on last field / Ctrl+S: Save spell
- Esc: Cancel (shows confirmation if dirty)

**Dropdown**:
- Lists all spellbooks + "Skip - just create spell"
- Arrow keys navigate
- Enter confirms selection

---

### 4. EditSpell

**Purpose**: Form to edit an existing spell.

**Layout**: Identical to AddSpell

**State**: Same `SpellFormState`, but with `editing_spell_id` set

**Behavior**:
- Form pre-populated with existing spell data
- Save updates the existing spell
- Esc shows "Discard changes?" if dirty

---

### 5. AddSpellbook

**Purpose**: Form to add a new spellbook.

**Layout**:
- Main area: Form with fields
- Footer: Keyboard hints

**State** (`SpellbookFormState`):
```rust
pub struct SpellbookFormState {
    pub active_field: SpellbookFormField,
    pub name: String,
    pub cover: String,
    pub sigil: String,
    pub style: SpineStyle,
    pub dirty: bool,
}
```

**Fields**:
| Field | Icon | Description |
|-------|------|-------------|
| Name | `*` | Spellbook name |
| Cover | `::` | Description |
| Sigil | `@` | Symbol/emoji |
| Style | `~` | Spine style (dropdown) |

**Navigation**:
- Tab / Arrow keys: Move between fields
- Enter on style field: Toggle dropdown
- Enter on last field / Ctrl+S: Save spellbook
- Esc: Cancel (shows confirmation if dirty)

---

## Overlays

### OutputModal

**Purpose**: Display command output (TUI runs and job output).

**State** (`OutputModalState`):
```rust
pub struct OutputModalState {
    pub content: Vec<String>,    // Output lines (cap: 10,000)
    pub scroll_offset: usize,
    pub is_streaming: bool,      // True while command running
    pub exit_code: Option<i32>,
    pub source: OutputSource,    // Job or TUI run
}
```

**Controls**:
- ↑ ↓: Scroll output
- Ctrl+b: Promote to background (if TUI run)
- Esc: Close modal

**Display**:
- Shows command output with syntax highlighting
- Auto-scrolls to bottom when streaming
- Shows exit code when complete
- Shows "Output truncated" warning if > 10,000 lines

---

### ConfirmDialog

**Purpose**: Show confirmation prompts.

**State** (`ConfirmDialogState`):
```rust
pub struct ConfirmDialogState {
    pub message: String,
    pub selected: bool,  // True = Yes, False = No
}
```

**Controls**:
- ← →: Toggle between Yes / No
- Enter: Confirm selection
- Esc: Cancel (equivalent to No)

**Use Cases**:
- `confirm = true` spells before execution
- Delete spell confirmation
- Discard unsaved changes

---

### CommandPalette

**Purpose**: Filterable command input (`:` prefix).

**State** (`CommandPaletteState`):
```rust
pub struct CommandPaletteState {
    pub input: String,
    pub filtered_commands: Vec<Command>,
    pub selected_index: usize,
}
```

**Controls**:
- Type: Filter commands
- ↑ ↓: Navigate filtered list
- Enter: Execute selected command
- Esc: Cancel and close

**Available Commands**:
- `:n` - New spell
- `:nb` - New spellbook
- `:b` - Browse spellbooks
- `:s` - Browse spells
- `:jobs` - Toggle jobs sidebar
- `:c` - Card view mode
- `:p` - Spine view mode
- `:a` - Auto view mode
- `:t` - Cycle theme
- `:?` - Show help
- `:import <file>` - Import spells
- `:export [file]` - Export codex

---

### Help

**Purpose**: Display keybind reference.

**Controls**:
- Esc / ?: Close help

**Sections**:
- Global keybinds
- Mode-specific keybinds
- Execution modes
- Command palette

---

## Focus Management

```rust
pub enum FocusTarget {
    Main,         // Main content has focus
    JobsSidebar,  // Jobs sidebar has focus
}
```

**Tab Key**: Cycles focus between Main and JobsSidebar (when sidebar is open)

**Visual Indicators**:
- Focused component has brighter border
- Unfocused component has muted border

**Event Routing**:
- Focused component receives key events
- Global keybinds (`:`, `t`, `v`, `q`) always work regardless of focus

---

## Footer Hints

Context-aware keyboard hints displayed in footer.

### BrowseSpellbooks
```
Enter: open | /: search | :: commands | t: theme | v: view | q: quit
```

### BrowseSpells
```
Enter: copy | r: run | e: edit | d: delete | f: fav | /: search | ←: back
```

### AddSpell / EditSpell
```
Tab: next field | Enter: save | Esc: cancel
```

### OutputModal
```
↑↓: scroll | Ctrl+b: background | Esc: close
```

### Jobs Sidebar (focused)
```
↑↓: navigate | Enter: view | k: kill | Esc: close | Tab: main
```

---

## Navigation Flow

```
         ┌──────────────────┐
         │ BrowseSpellbooks │  (Home)
         └────────┬─────────┘
                  │
        Enter on spellbook
                  │
                  ▼
         ┌───────────────┐
         │  BrowseSpells  │
         └───────┬───────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
Enter: copy    e: edit     d: delete
    │            │            │
    │            ▼            ▼
    │      ┌─────────┐  ┌───────────┐
    │      │EditSpell│  │ConfirmDialog│
    │      └─────────┘  └───────────┘
    │
    └──> Clipboard + Notification

From any mode:
  :n      → AddSpell
  :nb     → AddSpellbook
  :jobs   → Toggle jobs sidebar
  :       → CommandPalette overlay
  ?       → Help overlay
```

---

## View Modes

BrowseSpellbooks supports three view modes:

### Cards
- Large card view
- Shows sigil, name, cover, spell count
- Responsive grid layout
- Requires wider terminal

### Spines
- Compact book spine view
- Vertical spines with sigils
- Fits more spellbooks on screen
- Works on narrow terminals

### Auto
- Automatically switches based on terminal width
- Cards when width > 100 cols
- Spines when width ≤ 100 cols

**Toggle**: `v` key or commands (`:c`, `:p`, `:a`)
**Persistence**: Saved in `config.toml`
