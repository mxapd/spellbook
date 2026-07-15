# Keybindings

## Global Keybinds

Available in all modes and overlays.

| Key | Action |
|-----|--------|
| `/` | Activate search mode (in Browse modes) |
| `Tab` | Cycle focus (Main ↔ Jobs Sidebar when sidebar open) |
| `:` | Open command palette |
| `t` | Cycle to next theme |
| `v` | Cycle view mode (cards → spines → list) |
| `q` | Quit application |
| `Esc` | Close overlay / deactivate search / go back |
| `?` | Show help overlay |

---

## Mode-Specific Keybinds

### BrowseSpellbooks (Home)

| Key | Action |
|-----|--------|
| `↑` / `k` | Move to previous row |
| `↓` / `j` | Move to next row |
| `←` / `h` | Move left within row (wraps) |
| `→` / `l` | Move right within row (wraps) |
| `Enter` | Open selected spellbook |
| `/` | Activate search mode |
| `:` | Open command palette |
| `Type` | Filter spellbooks (when search active) |

### BrowseSpells

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up in spell list |
| `↓` / `j` | Move down in spell list |
| `Enter` | Copy incantation to clipboard |
| `r` | Run with spell's default mode |
| `s` | Force simple run (exit TUI) |
| `Ctrl+r` | Force TUI run (capture output) |
| `Ctrl+b` | Force background run (detached job) |
| `e` | Edit selected spell |
| `d` | Delete selected spell (with confirmation) |
| `f` | Toggle favorite |
| `/` | Activate search mode |
| `←` / `h` / `Esc` | Back to BrowseSpellbooks |
| `Type` | Filter spells (when search active) |

### AddSpell / EditSpell

| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `Shift+Tab` | Move to previous field |
| `↑` | Move to previous field |
| `↓` | Move to next field |
| `Enter` | Save spell (on last field or any field with Ctrl) |
| `Ctrl+S` | Save spell (from any field) |
| `Esc` | Cancel (shows confirmation if dirty) |
| `Type` | Add character to current field |
| `Backspace` | Delete last character |
| `Space` | Toggle checkbox fields (Confirm, Favorite) |

#### Spellbook Dropdown (when Spellbook field active)

| Key | Action |
|-----|--------|
| `↑` / `k` | Navigate up |
| `↓` / `j` | Navigate down |
| `Enter` | Confirm selection |
| `Esc` | Close dropdown |

### AddSpellbook

| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `Shift+Tab` | Move to previous field |
| `↑` | Move to previous field |
| `↓` | Move to next field |
| `Enter` | Save spellbook |
| `Ctrl+S` | Save spellbook (from any field) |
| `Esc` | Cancel (shows confirmation if dirty) |
| `Type` | Add character to current field |
| `Backspace` | Delete last character |

---

## Overlay Keybinds

### OutputModal (Streaming)

| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up (disables auto-scroll) |
| `↓` / `j` | Scroll down (disables auto-scroll) |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |
| `Home` | Jump to top |
| `End` | Jump to bottom |
| `s` | Toggle auto-scroll on/off |
| `Ctrl+c` | **Kill** running process (sends SIGKILL) |
| `Ctrl+b` | **Promote to background** (restarts as detached job) |
| `Esc` | Close modal (only when process finished) |

**Streaming Features**:
- **Real-time output**: Lines appear as they're produced
- **10,000 line cap**: Old lines discarded automatically (FIFO)
- **Auto-scroll**: View stays at bottom (toggle with `s`)
- **Color coding**: Stderr highlighted in red
- **Status indicator**: Title shows ⟳ (running), ✓ (success), ✗ (failure)
- **Footer hints**: Context-aware based on process state

### ConfirmDialog

| Key | Action |
|-----|--------|
| `←` / `h` | Select "No" |
| `→` / `l` | Select "Yes" |
| `Tab` | Toggle selection |
| `Enter` | Confirm selection |
| `Esc` | Cancel (equivalent to "No") |
| `y` | Quick confirm "Yes" |
| `n` | Quick confirm "No" |

### CommandPalette

| Key | Action |
|-----|--------|
| `Type` | Filter commands by name/alias |
| `↑` / `k` | Navigate up in filtered list |
| `↓` / `j` | Navigate down in filtered list |
| `Enter` | Execute selected command |
| `Esc` | Cancel and close |
| `Backspace` | Delete last character from input |

### Help

| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |
| `Esc` / `?` | Close help |

---

## Jobs Sidebar

When sidebar is open and focused (use Tab to focus):

| Key | Action |
|-----|--------|
| `↑` / `k` | Navigate up in job list |
| `↓` / `j` | Navigate down in job list |
| `Enter` | View job output in OutputModal |
| `k` | Kill selected running job |
| `c` | Cancel selected queued job |
| `Esc` | Close sidebar |
| `Tab` | Return focus to main content |

---

## Command Palette Commands

Type `:` to open command palette, then:

| Command | Aliases | Action |
|---------|---------|--------|
| `:n` | `:new`, `:spell` | New spell (open AddSpell form) |
| `:nb` | `:newbook`, `:spellbook` | New spellbook (open AddSpellbook form) |
| `:b` | `:browse`, `:books` | Browse spellbooks mode |
| `:s` | `:spells` | Browse spells mode |
| `:jobs` | `:j` | Toggle jobs sidebar |
| `:c` | `:cards` | Switch to cards view mode |
| `:p` | `:spines` | Switch to spines view mode |
| `:l` | `:list` | Switch to list view mode |
| `:t` | `:theme` | Cycle to next theme |
| `:?` | `:help`, `:h` | Show help overlay |
| `:import <file>` | | Import spells from file |
| `:export [file]` | | Export codex to file |
| `:q` | `:quit` | Quit application |

---

## Execution Modes

When viewing a spell in BrowseSpells:

| Key | Mode | Behavior |
|-----|------|----------|
| `r` | Default | Use spell's configured `run_mode` |
| `s` | Simple | Exit TUI, execute in terminal, user back at shell |
| `Ctrl+r` | TUI | Capture output in modal, stream in real-time |
| `Ctrl+b` | Background | Detach as job, track in sidebar, notify on completion |

### Mode Details

**Simple Mode**:
- TUI exits completely
- Command executes via `$SHELL -c`
- User returns to their shell
- Use for: quick commands (ls, ps, git status)

**TUI Mode**:
- Output captured and displayed in modal
- Streams in real-time (up to 10,000 lines)
- Can promote to background with Ctrl+b
- Use for: commands with output to review (grep, find, curl)

**Background Mode**:
- Process detaches (survives TUI close)
- Job appears in sidebar
- D-Bus notification on completion
- Use for: long-running commands (builds, deployments, downloads)

---

## Search Mode

In BrowseSpellbooks and BrowseSpells:

1. Press `/` to activate search
2. Type to filter items
3. `↑` / `↓` to navigate results
4. `Enter` to select
5. `Esc` to deactivate search and clear query

**Search Filters**:
- BrowseSpellbooks: Filters by spellbook name
- BrowseSpells: Filters by spell name, lore, school, and glyphs

---

## Vi-style Navigation

The app supports vim-inspired keybindings for navigation:

| Vi Key | Arrow Key | Action |
|--------|-----------|--------|
| `j` | `↓` | Move down / next |
| `k` | `↑` | Move up / previous |
| `h` | `←` | Move left / back |
| `l` | `→` | Move right / forward |

Both styles work in all contexts (lists, grids, overlays).

---

## Focus Cycling

When jobs sidebar is open:

```
Main Content ←─ Tab ─→ Jobs Sidebar
```

- **Tab**: Cycle focus forward (Main → Sidebar → Main)
- **Shift+Tab**: Cycle focus backward (Sidebar → Main → Sidebar)

**Visual Indicators**:
- Focused component has bright border
- Unfocused component has muted border

---

## Quick Reference Card

### Essential Keybinds

```
Navigation       Actions           Modes
──────────────   ───────────────   ─────────────
↑↓←→ / hjkl      Enter: open/copy  r: run default
Tab: focus       e: edit           s: simple run
Esc: back        d: delete         ^r: TUI run
                 f: favorite       ^b: background

Search & Help    Commands          System
──────────────   ───────────────   ─────────────
/: search        :: palette        t: theme
?: help          :n: new spell     v: view mode
                 :jobs: sidebar    q: quit
```
