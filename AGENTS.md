# Spellbook v2 Developer Notes

## Design Preferences

- **No emojis** - prefer ASCII or text characters (except in user-facing sigils)
- Cool ASCII symbols are welcome (e.g., *, >, ::, #, ∧, ⟳, ✓, ✗, etc.)
- More symbols are in symbols.md
- **Clean architecture** - component state encapsulation, no God objects
- **Explicit over implicit** - use enums like SpellbookRef instead of raw indices

## Development Environment

This project uses **NixOS**. All dependencies should be managed via:

```bash
nix develop      # Enter development shell with all dependencies
nix shell        # Run commands with dependencies available
```

Do not use traditional package managers (apt, brew, cargo install, etc.) for system dependencies.

## Code Style

- Follow existing code conventions in the codebase
- Use theme colors (from `state.theme`) for all UI styling
- Keep render functions clean and readable
- Each UI component should have its own state struct
- Prefer pattern matching over if/else chains
- Use `#[serde(default)]` for all new optional fields in data models

## Architecture (v2)

### Core Principles

1. **Single-mode navigation** - One active `Mode`, multiple `Overlay`s
2. **Component state encapsulation** - Each component owns its state
3. **Type-safe references** - Use `SpellbookRef` enum, not raw indices
4. **Atomic archivist** - Write-to-temp + rename pattern
5. **Event priority** - Overlay → Sidebar → Mode → Global

### Mode Enum

```rust
pub enum Mode {
    BrowseSpellbooks,   // Home - card/spine view
    BrowseSpells,       // Spells in selected spellbook
    AddSpell,           // Add spell form
    EditSpell,          // Edit spell form
    AddSpellbook,       // Add spellbook form
}
```

### Overlay Enum

```rust
pub enum Overlay {
    OutputModal,        // Command output viewer
    ConfirmDialog,      // Confirmation prompts
    CommandPalette,     // : command input
    Help,               // ? keybind reference
}
```

### Jobs Sidebar

**Important**: Jobs sidebar is NOT an overlay. It's a toggleable panel that coexists with any mode.

- Toggle with `:jobs`
- Renders on right side
- Has own focus state (`FocusTarget::JobsSidebar`)
- Tab key cycles focus between Main and Sidebar

## Key Features (v2)

### Execution Modes

Three execution modes for spells:

1. **Simple** (`s`) - Exit TUI, exec via `$SHELL -c`, user back at shell
2. **TUI** (`Ctrl+r`) - Capture output in modal, stream in real-time (10k line cap)
3. **Background** (`Ctrl+b`) - Detach as job, track in sidebar, notify on completion

### Virtual Spellbooks

- **Favorites**: Dynamic collection of spells with `favorite = true`
- **Recent**: Last 100 used spells from `recents.toml`
- Both appear at top of spellbook list
- Referenced via `SpellbookRef::Virtual(VirtualKind)`

### Focus Management

```rust
pub enum FocusTarget {
    Main,         // Main content has focus
    JobsSidebar,  // Sidebar has focus
}
```

- Tab key cycles focus when sidebar is open
- Visual indicators (bright vs muted borders)
- Focused component receives key events first

### Search Activation

- `/` key activates search mode (explicit, no conflicts)
- `search_active: bool` flag in browser states
- Esc deactivates and clears query
- Filters: spellbooks by name, spells by name/lore/school/glyphs

## Persistence

### File Locations

| File | Purpose |
|------|---------|
| `codex.toml` | Spells and spellbooks |
| `config.toml` | User settings (view mode, defaults) |
| `theme.toml` | Theme selection |
| `~/.spellbook/jobs.toml` | Job registry |
| `~/.spellbook/recents.toml` | Recently used spells |
| `~/.spellbook/spellbook.log` | Application logs |

### Atomic Writes

All TOML writes use atomic pattern:
```rust
write_to_temp("{file}.tmp")
fs::rename("{file}.tmp", "{file}")
```

### Retention Policies

- **Recents**: Keep last 100, FIFO eviction
- **Jobs**: Keep last 50, auto-purge on startup
- **Output lines**: Cap at 10,000 per job

## Data Model

### Spell

```rust
pub struct Spell {
    pub id: String,              // UUID
    pub name: String,
    pub incantation: String,
    pub lore: String,
    pub school: String,
    pub glyphs: Vec<String>,
    pub confirm: bool,
    pub run_mode: RunMode,
    pub working_dir: String,
    pub favorite: bool,
}
```

**Critical**: Use UUIDs for `id`, not sequential numbers. References are by ID, not name.

### SpellbookRef

```rust
pub enum SpellbookRef {
    Virtual(VirtualKind),
    Codex(usize),
}

pub enum VirtualKind {
    Favorites,
    Recent,
}
```

**Critical**: Always use SpellbookRef when referencing spellbooks, never raw indices.

## Known Limitations

- **Ctrl+Z**: Crossterm raw mode captures Ctrl+Z. Fundamental limitation - return false to pass through.
- **Font sizing**: Ratatui does not support font size changes.
- **Exec on Windows**: `exec()` is Unix-only. Use conditional compilation for Windows fallback.

## Navigation

### BrowseSpellbooks

- Row-based grid navigation
- Left/Right wrap within row
- Up/Down wrap within column
- Enter opens spellbook

### BrowseSpells

- List navigation
- Up/Down through spells
- Enter copies to clipboard
- `r`/`s`/`Ctrl+r`/`Ctrl+b` for execution modes
- `e` edit, `d` delete, `f` favorite

## Command Bar

`:` prefix for commands:

| Command | Action |
|---------|--------|
| `:n` | New spell |
| `:nb` | New spellbook |
| `:b` | Browse spellbooks |
| `:s` | Browse spells |
| `:jobs` | Toggle jobs sidebar |
| `:c` / `:p` / `:a` | Card / Spine / Auto view |
| `:t` | Cycle theme |
| `:?` | Help |
| `:import <file>` | Import spells |
| `:export [file]` | Export codex |

## Testing Checklist

When testing v2:

1. **Navigation**
   - [ ] Browse spellbooks with arrows
   - [ ] Enter to open spellbook
   - [ ] Navigate spells with Up/Down
   - [ ] Back with Esc or ←

2. **Execution**
   - [ ] Copy to clipboard (Enter)
   - [ ] Simple mode (`s`) - TUI exits
   - [ ] TUI mode (`Ctrl+r`) - output modal appears
   - [ ] Background mode (`Ctrl+b`) - job in sidebar
   - [ ] Confirmation dialog on `confirm = true` spells

3. **CRUD**
   - [ ] Add spell (`:n`)
   - [ ] Edit spell (`e`)
   - [ ] Delete spell (`d`) - confirmation shown
   - [ ] Toggle favorite (`f`)
   - [ ] Add spellbook (`:nb`)

4. **Virtual Spellbooks**
   - [ ] Favorites appears when favorites exist
   - [ ] Recent appears with recent activity
   - [ ] Both at top of list

5. **Jobs**
   - [ ] Jobs sidebar toggles (`:jobs`)
   - [ ] Running jobs show ⟳ icon
   - [ ] Completed show ✓, failed show ✗
   - [ ] Enter on job shows output
   - [ ] Kill with `k` works

6. **Search**
   - [ ] `/` activates search
   - [ ] Real-time filtering works
   - [ ] Esc clears and deactivates

7. **Focus**
   - [ ] Tab cycles between main and sidebar
   - [ ] Visual indicators correct
   - [ ] Events route to focused component

8. **Themes & Views**
   - [ ] `t` cycles themes
   - [ ] `v` cycles view modes
   - [ ] Preferences persist across restarts

9. **AddSpellbook Mode**
   - [ ] `:nb` opens AddSpellbook form
   - [ ] Tab/Arrow keys navigate fields
   - [ ] All fields (Name, Cover, Sigil) accept input
   - [ ] Ctrl+S or Enter on last field saves
   - [ ] Esc cancels (with confirmation if dirty)
   - [ ] Spellbook appears in list after save

10. **Streaming Output**
    - [ ] `Ctrl+r` shows streaming output modal
    - [ ] Output streams in real-time
    - [ ] `Ctrl+C` kills running process
    - [ ] `Ctrl+B` promotes to background job
    - [ ] Auto-scroll keeps view at bottom
    - [ ] Scroll up/down disables auto-scroll
    - [ ] `s` toggles auto-scroll on/off

11. **Loading States**
    - [ ] Spinner appears during spell save
    - [ ] Spinner appears during import/export
    - [ ] "Loading..." message shows operation

## Known Issues & Pending Fixes

The following issues require manual testing and potential fixes:

### High Priority
- **Simple Mode Recents**: Verify recents.toml is written BEFORE exec() replaces process
- **TUI Streaming Edge Cases**: Test with commands that produce massive output (>10k lines)
- **Job Promotion**: Verify Ctrl+B in streaming modal correctly moves job to background

### Medium Priority
- **Input Popup Integration**: InputPopup overlay is migrated but needs to be triggered when executing spells with placeholders (e.g., `<pid>`, `<port>`)
- **Focus Edge Cases**: Test rapid Tab switching between main and sidebar
- **Theme Persistence**: Verify theme changes survive app restart

### Low Priority
- **Windows Compatibility**: exec() is Unix-only; test fallback on Windows
- **Very Long Commands**: Test display truncation in forms
- **Unicode Handling**: Test spell names/commands with emojis/unicode

## Migration Notes

### V1 → V2

On first v2 run:
1. Detect missing `id` fields in spells
2. Generate UUIDs for all spells
3. Update spellbook references from names to IDs
4. Rewrite `codex.toml` with new format
5. Log migration in `spellbook.log`

Backup `codex.toml` before migration (automatic).

## Debug Tips

- Set `SPELLBOOK_LOG=debug` for verbose logging
- Check `~/.spellbook/spellbook.log` for errors
- Job output in `~/.spellbook/job_<id>.out/err`
- Use `RUST_BACKTRACE=1` for panics

## Architecture Refactor (In Progress)

**IMPORTANT: events.rs is frozen for refactor. Do not add new features to events.rs until refactor is complete.**

### The Problem
UiState has multiple overlapping state flags that don't know about each other:
- `mode: Mode` vs `search_mode: SearchMode` (two parallel enums!)
- `is_typing`, `search_active`, `showing_spellbooks` (all separate booleans)

This causes unpredictable behavior and 2,482 line event handlers.

### The Solution
Nest state inside Mode variants:
```rust
enum Mode {
    BrowseSpellbooks(BrowseState),
    BrowseSpells(BrowseState),
    AddSpell(FormState),
    // ...
}

enum BrowseState {
    Idle,
    Searching(String),  // query lives here
}
```

### Migration Steps
1. Freeze events.rs (no new features)
2. Add BrowseState to Mode, collapse search state
3. Split events.rs into browse_spells.rs, browse_spellbooks.rs, form.rs
4. Audit remaining UiState fields

See `docs/architecture-refactor.md` for full details.

## References

- [refactor.md](refactor.md) - Complete v2 specification (source of truth)
- [docs/architecture.md](docs/architecture.md) - System design
- [docs/architecture-diagram.md](docs/architecture-diagram.md) - Visual architecture diagrams (DEBUGGING AID)
- [docs/data-model.md](docs/data-model.md) - Data structures
- [docs/ui-screens.md](docs/ui-screens.md) - UI details
- [docs/roadmap.md](docs/roadmap.md) - Implementation phases
