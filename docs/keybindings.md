# Keybindings

## Global
| Key | Action |
|-----|--------|
| `q` | Quit application |
| `Esc` | Go back / Close search / Cancel |
| `t` | Cycle to next theme |

## Spellbook List (Home)
| Key | Action |
|-----|--------|
| `↓` / `k` | Move selection down |
| `↑` / `j` | Move selection up |
| `Enter` | Open selected spellbook |
| `/` | Open search overlay |

## Spell List (Inside a Spellbook)
| Key | Action |
|-----|--------|
| `↓` / `k` | Move selection down |
| `↑` / `j` | Move selection up |
| `Enter` | Copy incantation to clipboard |
| `Esc` | Return to spellbook list |
| `/` | Open search overlay |

## Search Overlay
| Key | Action |
|-----|--------|
| `↓` / `k` | Move selection down |
| `↑` / `j` | Move selection up |
| `Enter` | Copy selected spell to clipboard |
| `Esc` | Close search and return to previous screen |
| `Backspace` | Delete last character from search query |
| `Any letter` | Add character to search query |

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

## Vim-style Navigation

The app supports vim-inspired keybindings:
- `j` = down
- `k` = up
- `h` = back/left
- `l` = forward/right

## Planned / Not Yet Implemented

| Key | Screen | Action |
|-----|--------|--------|
| `h` | SpellList | Go to previous spellbook |
| `l` | SpellList | Go to next spellbook |
