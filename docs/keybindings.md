# Keybindings

## Global
| Key | Action |
|-----|--------|
| `Ctrl+C` | Quit application |
| `Esc` | Go back / Close search / Cancel |
| `t` | Cycle to next theme |
| `v` | Cycle view mode (cards/spines) |
| `:` | Open command bar |

## Command Bar (SearchOverlay)

Press `:` to open the command bar, then type a command:

| Command | Action |
|---------|--------|
| `:n` | New spell - open Add Spell form |
| `:b` | Browse spellbooks |
| `:s` | Browse spells in selected spellbook |
| `:c` | Card view mode |
| `:p` | Spine (compact) view mode |
| `:a` | Auto view mode (responsive) |
| `:t` | Cycle theme |
| `:?` | Show help |

## SearchOverlay Modes

### BrowseSpellbooks (default)
| Key | Action |
|-----|--------|
| `↓` / `k` | Move to next row of spellbooks |
| `↑` / `j` | Move to previous row of spellbooks |
| `→` / `l` | Move right within row (wraps to left) |
| `←` / `h` | Move left within row (wraps to right) |
| `Enter` | Open selected spellbook (BrowseSpells mode) |
| `:` | Open command bar |
| `Esc` | Close search and return to previous screen |

### BrowseSpells
| Key | Action |
|-----|--------|
| `↓` / `k` | Move down through spell list |
| `↑` / `j` | Move up through spell list |
| `→` / `l` | Page down through spell list |
| `←` / `h` | Return to spellbook browsing |
| `Enter` | Copy selected spell to clipboard |
| `Esc` | Return to spellbook browsing |
| `:` | Open command bar |

### Search/Command Input
When typing after `:` (command mode):
| Key | Action |
|-----|--------|
| `Any letter` | Filter commands |
| `↑` / `↓` | Navigate filtered commands |
| `Enter` | Execute selected command |
| `Esc` | Cancel and clear |

When typing without `:` (search mode):
| Key | Action |
|-----|--------|
| `Any letter` | Search spells |
| `↑` / `↓` | Navigate results |
| `Enter` | Copy selected spell |
| `Esc` | Clear search |

### Available Commands
| Command | Action |
|---------|--------|
| `:n` | New spell |
| `:b` | Browse spellbooks |
| `:s` | Browse spells |
| `:c` | Card view |
| `:p` | Spine view |
| `:t` | Cycle theme |
| `:?` | Help |

## Spellbook List (Home)
| Key | Action |
|-----|--------|
| `↓` / `k` | Move selection down |
| `↑` / `j` | Move selection up |
| `Enter` | Copy incantation to clipboard |
| `Esc` | Return to spellbook list |
| `/` | Open search overlay |

## Add Spell Screen
| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `↓` / `↑` | Navigate fields (or dropdown when on Spellbook) |
| `Enter` | Save spell and return to spellbook list |
| `Esc` | Cancel and return to spellbook list |
| `Any letter` | Add character to current field |
| `Backspace` | Delete last character from current field |

### Add Spell Field Navigation
1. **Name** - Spell name
2. **Command** - The CLI command/incantation
3. **Lore** - Description or notes
4. **School** - Category (e.g., Tool, Utility, Git)
5. **Tags** - Comma-separated tags
6. **Spellbook** - Dropdown to select which spellbook to add to

### Spellbook Dropdown
When on the Spellbook field:
- `↓` / `↑` - Navigate through spellbook options
- `Enter` - Confirm selection and move to next field
- Options include all spellbooks plus "Skip - just create spell"

## View Modes

The app supports two view modes for the spellbook browser. Both modes are responsive and adapt to terminal width.

| Mode | Command | Description |
|------|---------|-------------|
| Cards | `:c` | Large card view with sigils and descriptions |
| Spines | `:p` | Compact spine (book spine) view |

Cycle view mode with `v` key or use commands (`:c`, `:p`).

## Vim-style Navigation

The app supports vim-inspired keybindings:
- `j` = down
- `k` = up
- `h` = back/left
- `l` = forward/right
