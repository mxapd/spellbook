# Spellbook

A powerful TUI application for managing and executing CLI command snippets with style.

## Features

### Core

- **Browse Spellbooks** - Organize commands into categorized collections with visual themes
- **Global Search** - Filter spells by name, description, tags, or category
- **Quick Copy** - Copy any command to clipboard with Enter
- **Three Execution Modes** - Simple, TUI, and Background for different use cases
- **Background Jobs** - Run long commands as detached jobs with progress tracking
- **D-Bus Notifications** - Get notified when jobs complete or commands are copied

### Organization

- **Favorites** - Mark frequently used spells for quick access
- **Recent Items** - Virtual spellbook of recently used commands
- **Virtual Spellbooks** - Dynamic collections that update automatically
- **Tags & Categories** - Flexible organization with schools and glyphs

### UI/UX

- **10 Built-in Themes** - Beautiful color schemes from Dracula to Solarized
- **View Modes** - Cards or spines view mode
- **Command Palette** - Quick actions with `:` prefix
- **Vi-style Navigation** - Use j/k or arrow keys
- **Jobs Sidebar** - Monitor running jobs without leaving the TUI
- **Real-time Output** - Stream command output in modal overlays

### Management

- **Add/Edit/Delete** - Full CRUD operations for spells and spellbooks
- **Import/Export** - Share spell collections as TOML files
- **Validation** - Automatic checks for broken references and duplicates
- **Persistence** - All preferences saved automatically

---

## Quick Start

```bash
# Run the app
cargo run

# Add a new spell directly
cargo run -- --add
```

---

## Installation

### Requirements

- **wl-clipboard** (Wayland) or **xclip/xsel** (X11) - For clipboard support
- **libnotify** (notify-send) - For D-Bus notifications
- **nohup** - For background job execution (usually pre-installed)

On NixOS (using flake.nix):
```bash
nix develop  # Enters dev shell with all dependencies
```

On other systems:
```bash
# Debian/Ubuntu
sudo apt install wl-clipboard libnotify-bin

# Arch
sudo pacman -S wl-clipboard libnotify

# Fedora
sudo dnf install wl-clipboard libnotify
```

---

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `/` | Activate search |
| `Tab` | Cycle focus (main / jobs sidebar) |
| `:` | Open command palette |
| `t` | Cycle theme |
| `v` | Cycle view mode |
| `q` | Quit |
| `?` | Help |

### Browse & Navigate
| Key | Action |
|-----|--------|
| `↑↓` or `jk` | Navigate |
| `←→` or `hl` | Move left/right |
| `Enter` | Open / Copy |
| `Esc` | Go back |

### Spell Actions
| Key | Action |
|-----|--------|
| `r` | Run (default mode) |
| `s` | Run in Simple mode |
| `Ctrl+r` | Run in TUI mode |
| `Ctrl+b` | Run in Background mode |
| `e` | Edit spell |
| `d` | Delete spell |
| `f` | Toggle favorite |

### Command Palette
Type `:` then:
| Command | Action |
|---------|--------|
| `:n` | New spell |
| `:nb` | New spellbook |
| `:jobs` | Toggle jobs sidebar |
| `:import <file>` | Import spells |
| `:export [file]` | Export codex |
| `:c` / `:p` / `:l` | Card / Spine / List view |

---

## Execution Modes

Spellbook supports three execution modes for different use cases:

### Simple Mode (`s`)
Exit TUI and run command in your shell. Perfect for quick commands.
```
Use for: ls, ps, git status, cd
```

### TUI Mode (`Ctrl+r`)
Capture output in a modal overlay with real-time streaming.
```
Use for: grep, find, curl, cat
```

### Background Mode (`Ctrl+b`)
Detach command as a job, track in sidebar, get notified on completion.
```
Use for: cargo build, nixos-rebuild, long downloads
```

Each spell has a default mode, but you can override it at execution time.

---

## Configuration

### File Locations

| File | Purpose |
|------|---------|
| `codex.toml` | Your spells and spellbooks |
| `theme.toml` | User preferences (view mode, theme) |
| `theme.toml` | Selected theme |
| `~/.spellbook/jobs.toml` | Background job registry |
| `~/.spellbook/recents.toml` | Recently used spells |

### Themes

10 built-in themes:
- default (dark) | default-light
- dracula
- gruvbox-dark | gruvbox-light
- nord
- catppuccin
- one-dark
- solarized-dark | solarized-light

Cycle with `t` key or `:t` command.

### View Modes

- **Cards**: Large card view with sigils and descriptions
- **Spines**: Compact book spine view
- **List**: Simple vertical list

Cycle with `v` key or commands (`:c`, `:p`, `:l`).

---

## Project Structure

```
src/
├── main.rs              # Entry point
├── cli.rs               # CLI argument parsing
├── state.rs             # Application state
├── models/              # Data structures
│   ├── spell.rs
│   ├── spellbook.rs
│   ├── codex.rs
│   ├── job.rs
│   └── theme.rs
├── archivist/           # TOML load/save
├── invoker/             # Execution modes & job manager
├── ui/                  # Rendering & events
│   ├── mod.rs
│   ├── render.rs
│   ├── events.rs
│   ├── browse_spellbooks.rs
│   ├── browse_spells.rs
│   ├── search_overlay.rs
│   ├── form.rs
│   ├── add_spell_form.rs
│   ├── add_spellbook_form.rs
│   ├── spellbook_browser.rs
│   ├── spell_list.rs
│   ├── streaming_modal.rs
│   ├── jobs.rs
│   ├── confirm.rs
│   ├── input.rs
│   ├── help.rs
│   └── quick_add_spell.rs
├── clipboard.rs         # Clipboard operations
├── editor.rs            # External editor integration
├── logging.rs           # Logging setup
├── validation.rs        # Data validation
└── test_utils.rs        # Test helpers

docs/                    # Documentation
├── architecture.md      # System design
├── architecture-diagrams-mermaid.md # Architecture diagrams
├── data-model.md        # Data structures
├── ui-screens.md        # UI details
└── keybindings.md       # Complete keybind reference
```

---

## Data Format

Spells are defined in `codex.toml`:

```toml
[[spells]]
id = "550e8400-e29b-41d4-a716-446655440000"
name = "List Processes"
incantation = "ps aux"
lore = "Shows all running processes."
school = "System"
glyphs = ["process", "running", "ps"]
run_mode = "simple"
favorite = false

[[spellbooks]]
name = "System"
cover = "System monitoring commands."
sigil = "∧"
spells = [
    "550e8400-e29b-41d4-a716-446655440000"
]
```

Each spell has:
- **Unique ID** (UUID) for stable references
- **Name** and **incantation** (the command)
- **Lore** (description) and **school** (category)
- **Glyphs** (tags for search)
- **run_mode** (default execution mode)
- **confirm** flag (require confirmation)
- **working_dir** (optional working directory)

---

## Jobs System

Background jobs run independently of the TUI:

- **Detached execution** - Jobs survive TUI close
- **Output capture** - stdout/stderr saved to files
- **Status tracking** - Monitor via jobs sidebar
- **Notifications** - D-Bus alerts on completion/failure
- **Limits** - Max 10 concurrent jobs, 50 total retained

Toggle sidebar with `:jobs`, navigate with arrows, press Enter to view output.

---

## Development

Built with:
- **Rust** - Systems programming language
- **ratatui** - Terminal UI framework
- **crossterm** - Terminal I/O
- **serde + toml** - Serialization

Run tests:
```bash
cargo test
```

Build release:
```bash
cargo build --release
```

---

## Documentation

- [Architecture](docs/architecture.md) - System design and data flow
- [Data Model](docs/data-model.md) - Spell, Spellbook, Job structures
- [UI Screens](docs/ui-screens.md) - Mode/Overlay system details
- [Keybindings](docs/keybindings.md) - Complete keybind reference
- [Architecture Diagrams](docs/architecture-diagrams-mermaid.md) - Visual architecture diagrams

---

## License

MIT License - See LICENSE file for details.

---

## Credits

Created with care by the Spellbook team.

Powered by [ratatui](https://ratatui.rs/) and the Rust ecosystem.
