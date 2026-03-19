# Spellbook

A TUI application for managing and quickly accessing CLI command snippets.

## Features

- **Browse Spellbooks** - Organize commands into categorized collections
- **Global Search** - Search across all spells by name, lore, or tags
- **Quick Copy** - Copy any command to clipboard with Enter
- **D-Bus Notifications** - Get notified when commands are copied
- **Vim-style Navigation** - Use j/k or arrow keys to navigate
- **Themes** - 10 built-in themes, cycle with `t` key
- **View Modes** - Cards or spines view (responsive)
- **Command Bar** - Quick actions with `:` prefix commands
- **Add Spells** - Add new spells via UI (`--add` flag)

## Quick Start

```bash
# Run the app
cargo run

# Run with Add Spell screen open
cargo run -- --add
```

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `Ctrl+C` | Quit |
| `Esc` | Go back / Close search / Cancel |
| `t` | Cycle theme |
| `v` | Cycle view mode |
| `:` | Open command bar |

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Navigate |
| `←` / `→` / `h` / `l` | Move within row / column |
| `Enter` | Open / Copy / Save |
| `/` | Open search (from home screens) |

### Command Bar
Press `:` then type:
| Command | Action |
|---------|--------|
| `:n` | New spell |
| `:b` | Browse spellbooks |
| `:s` | Browse spells |
| `:c` | Card view |
| `:p` | Spine view |
| `:t` | Cycle theme |
| `:?` | Help |

### Add Spell Screen
| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `↑` / `↓` | Navigate fields |
| `Enter` | Save |
| `Esc` | Cancel |

## Requirements

- **wl-clipboard** - For Wayland clipboard support (or xclip/xsel for X11)
- **notify-send** - For copy notifications (part of libnotify)

On NixOS, add to your system packages:
```
wl-clipboard
libnotify
```

## Configuration

Theme and view mode preferences are persisted in `theme.toml`.

### Available Themes
- default (dark)
- default-light
- dracula
- gruvbox-dark
- gruvbox-light
- nord
- catppuccin
- one-dark
- solarized-dark
- solarized-light

### View Modes
- **Cards**: Large card view with sigils and descriptions
- **Spines**: Compact spine view

Both modes are responsive and adapt to terminal width.

## Project Structure

```
src/
├── main.rs           # Entry point, CLI args, event loop
├── clipboard.rs      # Clipboard operations with fallback support
├── state.rs          # Application state (Codex + settings)
├── models/           # Data structures
│   ├── codex.rs      # Root data container
│   ├── spell.rs      # Spell/command definition
│   ├── spellbook.rs  # Spellbook/collection
│   ├── theme.rs      # Theme colors
│   └── view_mode.rs  # View mode enum
├── ui/               # Rendering and events
│   ├── mod.rs        # Screen states, UiState, SearchMode
│   ├── render.rs     # Main render dispatcher
│   ├── events.rs     # Key event handlers
│   ├── spellbook_list.rs
│   ├── spell_list.rs
│   ├── search_overlay.rs  # Primary navigation screen
│   └── add_spell.rs  # Add spell form
└── persistence/
    └── archivist.rs  # TOML load/save

codex.toml            # Spell data
theme.toml            # Theme and view mode configuration
```

## Data Format

Spells are defined in `codex.toml`:

```toml
[[spells]]
name = "List Processes"
incantation = "ps aux"
lore = "Shows all running processes."
school = "System"
glyphs = ["process", "running", "ps"]

[[spellbooks]]
name = "System"
cover = "System monitoring commands."
sigil = "*"
spells = ["List Processes"]
```
