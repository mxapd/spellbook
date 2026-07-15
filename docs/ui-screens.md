# UI Screens

## Overview

Spellbook uses a **single-mode navigation system** with overlays, not multiple top-level screens. The application always has one active `Mode` and zero or more `Overlay`s rendered on top.

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
    InputPopup,         // Parameter input for placeholders
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
- **List**: Simple vertical list

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
| Run Mode | `!!` | Default execution mode (simple/tui/background) |
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

### OutputModal (Streaming)

**Purpose**: Display real-time command output with streaming support.

**State** (`StreamingModalState`):
```rust
pub struct StreamingModalState {
    pub output: OutputModalState,
    pub streaming: Option<StreamingState>,
    pub auto_scroll: bool,
}

pub struct StreamingState {
    pub pid: Option<u32>,
    pub is_running: bool,
    pub command: String,
    pub spell_name: Option<String>,
    pub working_dir: Option<String>,
}
```

**Streaming Architecture**:
- Process spawned with piped stdout/stderr
- Background thread reads output lines
- mpsc channel sends lines to event loop
- UI polls channel every 100ms and updates display
- 10,000 line cap with automatic eviction (FIFO)

**Controls**:
- `↑/↓`: Manual scroll (disables auto-scroll)
- `s`: Toggle auto-scroll on/off
- `Ctrl+C`: Kill running process (sends SIGKILL)
- `Ctrl+B`: Promote to background (kills process, restarts via JobManager)
- `Esc`: Close modal (only when process finished)

**Display Features**:
- **Status indicator in title**:
  - `⟳` - Process running
  - `✓` - Completed successfully (exit code 0)
  - `✗` - Failed (non-zero exit code)
- **Color coding**:
  - Stderr lines: Red
  - System messages ([stderr], [Process killed]): Muted
  - Normal output: Default foreground
- **Auto-scroll**: Keeps view at bottom of output (toggle with `s`)
- **Truncation warning**: Shows `[!]` in footer when 10,000 line limit reached
- **Footer hints**: Context-aware (shows Ctrl+C/B while running, Esc when done)

**Promote to Background**:
When `Ctrl+B` is pressed on a running process:
1. Kill the current process
2. Start a new background job via JobManager
3. Job appears in jobs sidebar
4. Modal closes automatically

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
- `:l` - List view mode
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

### InputPopup

**Purpose**: Interactive parameter input for spells with placeholders.

**Description**: The InputPopup overlay enables parameterized spell execution. When a spell's incantation contains placeholder patterns like `<pid>`, `<port>`, or `<file>`, this popup collects user input for each parameter before execution.

**Placeholder Syntax**: Use `<name>` in spell incantations:
- `<pid>` - Process ID
- `<port>` - Port number
- `<file>` - File path
- `<directory>` or `<dir>` - Directory path
- `<service>` or `<svc>` - Service name
- `<message>` or `<msg>` - Message text
- Any custom `<name>` - User-defined parameter

**State** (`InputPopupState`):
```rust
pub struct InputPopupState {
    pub spell_id: String,
    pub spell_name: String,
    pub base_command: String,
    pub placeholders: Vec<Placeholder>,
    pub current_index: usize,
}

pub struct Placeholder {
    pub name: String,
    pub display_name: String,
    pub value: String,
}
```

**Controls**:
- Type: Enter value for current placeholder
- Tab / ↓: Move to next placeholder
- Shift+Tab / ↑: Move to previous placeholder
- Enter: Execute spell with substituted values (when all filled)
- Esc: Cancel execution
- Backspace: Delete character

**Workflow**:
1. User initiates spell execution (simple, TUI, or background)
2. System detects placeholders in incantation via regex `<([^>]+)>`
3. If placeholders found, InputPopup overlay is shown
4. User fills in values for each placeholder
5. Upon confirmation, placeholders are substituted: `<pid>` → `1234`
6. Modified command executes with user-provided values

**Example**:
```
Spell: "Kill Process"
Incantation: "kill -9 <pid>"

InputPopup shows:
  Command: kill -9 <pid>
  Process ID: _
  
User types: 1234
  Command: kill -9 1234
  Process ID: 1234

Result: "kill -9 1234" executes
```

**Note**: InputPopup is fully implemented but not yet integrated into the execution flow. To activate it, add placeholder detection before spell execution in the browse modes.

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

### List
- Simple vertical list of spellbooks
- Compact, works on narrow terminals

**Toggle**: `v` key or commands (`:c`, `:p`, `:l`)
**Persistence**: Saved in `config.toml`
