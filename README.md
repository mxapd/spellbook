# Spellbook

A TUI application for managing and quickly accessing CLI command snippets.

## Features

- **Browse Spellbooks** - Organize commands into categorized collections
- **Global Search** - Search across all spells by name, lore, or tags
- **Quick Copy** - Copy any command to clipboard with Enter
- **D-Bus Notifications** - Get notified when commands are copied
- **Vim-style Navigation** - Use j/k or arrow keys to navigate
- **Themes** - 10 built-in themes, cycle with `t` key
- **Add Spells** - Add new spells via UI (`--add` flag)

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `q` | Quit |
| `Esc` | Go back / Close search / Cancel |
| `t` | Cycle to next theme |

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Navigate |
| `Enter` | Open spellbook / Copy command / Save |
| `/` | Open search |

### Add Spell Screen
| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `↑` / `↓` | Navigate fields or dropdown |
| `Enter` | Save spell |
| `Esc` | Cancel |

## Requirements

- **wl-clipboard** - For Wayland clipboard support (or xclip/xsel for X11)
- **notify-send** - For copy notifications (part of libnotify)

On NixOS, add to your system packages:
```
wl-clipboard
libnotify
```

## Usage

```bash
# Run the app
cargo run

# Run with Add Spell screen open
cargo run -- --add

# The app reads from codex.toml in the current directory
```

## Theme Configuration

Theme preference is persisted in `theme.toml`. Available themes:
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

## Project Structure

```
src/
├── main.rs           # Entry point, CLI args, event loop
├── clipboard.rs      # Clipboard operations with fallback support
├── state.rs          # Application state (Codex + theme)
├── models/           # Data structures
│   ├── codex.rs      # Root data container
│   ├── spell.rs      # Spell/command definition
│   ├── spellbook.rs  # Spellbook/collection
│   └── theme.rs      # Theme colors
├── ui/               # Rendering and events
│   ├── mod.rs        # Screen states, UiState
│   ├── render.rs     # Main render dispatcher
│   ├── events.rs     # Key event handlers
│   ├── spellbook_list.rs
│   ├── spell_list.rs
│   ├── search_overlay.rs
│   └── add_spell.rs  # Add spell form
└── persistence/
    └── archivist.rs  # TOML load/save

codex.toml            # Spell data
theme.toml            # Theme configuration
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
