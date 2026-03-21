# Data Model (v2)

## Codex

Root data structure containing all spells and spellbooks.

```rust
pub struct Codex {
    pub spells: Vec<Spell>,
    pub spellbooks: Vec<Spellbook>,
}
```

---

## Spell

A single command snippet with metadata.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | String | Yes | — | UUID (e.g., "550e8400-e29b-41d4-a716-446655440000") |
| `name` | String | Yes | — | Display name (e.g., "List Processes") |
| `incantation` | String | Yes | — | The actual command(s) to run |
| `lore` | String | No | `""` | Description/usage notes |
| `school` | String | No | `""` | Category (e.g., "System", "Network") |
| `glyphs` | Vec<String> | No | `[]` | Search tags for filtering |
| `confirm` | bool | No | `false` | Require confirmation before execution |
| `run_mode` | String | No | `"simple"` | Default execution mode: "simple", "tui", or "background" |
| `working_dir` | String | No | `""` | Directory to run command in (optional) |
| `favorite` | bool | No | `false` | Whether the spell is favorited |

### Notes

- **ID**: UUIDs generated with `uuid::Uuid::new_v4()` when spell is created
- **ID Stability**: IDs must persist across restarts and renames
- **run_mode**: Determines default execution behavior (can be overridden at runtime)
- **confirm**: If true, shows confirmation dialog before any execution
- **Elevation**: If command needs `sudo`, include it in the incantation directly

### Example (TOML)

```toml
[[spells]]
id = "550e8400-e29b-41d4-a716-446655440000"
name = "List Processes"
incantation = "ps aux"
lore = "Shows all running processes."
school = "System"
glyphs = ["process", "running", "ps", "list"]
run_mode = "simple"
favorite = false

[[spells]]
id = "660e9401-f30c-52e5-b827-557766551111"
name = "Collect Garbage"
incantation = "sudo nix-collect-garbage -d"
lore = "Removes unused Nix store entries."
school = "NixOS"
glyphs = ["nix", "garbage", "cleanup"]
confirm = true
run_mode = "background"
working_dir = "/etc/nixos"
favorite = true
```

---

## Spellbook

A named collection of spells.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Display name |
| `cover` | String | No | Description/purpose |
| `sigil` | String | No | Emoji or symbol for visual identity |
| `spells` | Vec<String> | No | References to spell IDs (UUIDs) |
| `style` | String | No | Spine style for spine view mode |

### Notes

- **Spell references**: Use spell IDs (UUIDs), not names
- **Name changes**: Renaming a spell does not break spellbook references
- **Invalid references**: Spells that don't exist are skipped with a warning

### Example (TOML)

```toml
[[spellbooks]]
name = "System"
cover = "System monitoring and management commands."
sigil = "∧"
spells = [
    "550e8400-e29b-41d4-a716-446655440000",
    "660e9401-f30c-52e5-b827-557766551111"
]
style = "geometric"

[[spellbooks]]
name = "Network"
cover = "Network diagnostics and tools."
sigil = "⚡"
spells = [
    "770f0512-g41d-63f6-c938-668877662222"
]
```

---

## SpellbookRef

Enum for referencing spellbooks (virtual vs codex).

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

### Purpose

- Provides type-safe references to spellbooks
- Distinguishes between virtual spellbooks (Favorites, Recent) and codex spellbooks
- Prevents index offset bugs when virtual spellbooks are present

### Usage

```rust
// Virtual spellbook reference
let fav_ref = SpellbookRef::Virtual(VirtualKind::Favorites);

// Codex spellbook reference (index into codex.spellbooks)
let system_ref = SpellbookRef::Codex(0);
```

---

## RunMode

Execution mode enum for spells.

```rust
pub enum RunMode {
    Simple,      // Exit TUI, run in terminal
    Tui,         // Capture output, show in modal
    Background,  // Detach, track as job
}
```

### Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `Simple` | Exit TUI, execute via shell, user back at terminal | Quick commands (ls, ps, git status) |
| `Tui` | Capture output in modal overlay, stream in real-time | Commands with output to review (grep, find) |
| `Background` | Detach as job, track in sidebar, notify on completion | Long-running commands (builds, deployments) |

### Runtime Override

Users can override spell default with keybinds:
- `s` - Force simple mode
- `Ctrl+r` - Force TUI mode
- `Ctrl+b` - Force background mode
- `r` - Use spell's default `run_mode`

---

## Job

Represents a running or completed background command.

| Field | Type | Description |
|-------|------|-------------|
| `id` | u64 | Auto-incremented job ID |
| `spell_name` | String | Name of spell that was executed |
| `command` | String | Full command that was executed |
| `status` | JobStatus | Current state (Queued, Running, Completed, Failed, Cancelled) |
| `pid` | Option<u32> | Process ID (if running) |
| `exit_code` | Option<i32> | Exit code (if completed) |
| `started_at` | DateTime<Utc> | When the job started |
| `completed_at` | Option<DateTime<Utc>> | When the job finished |
| `output_file` | PathBuf | Path to stdout file |
| `error_file` | PathBuf | Path to stderr file |

### JobStatus Enum

```rust
pub enum JobStatus {
    Queued,      // Waiting to run (10 job limit)
    Running,     // Currently executing
    Completed,   // Exited with code 0
    Failed,      // Exited with non-zero code
    Cancelled,   // Killed by user
}
```

### Job Registry (TOML)

Jobs are persisted to `~/.spellbook/jobs.toml`:

```toml
[jobs]
next_id = 42

[[jobs.jobs]]
id = 1
spell_name = "Collect Garbage"
command = "sudo nix-collect-garbage -d"
status = "Completed"
exit_code = 0
started_at = "2026-03-21T10:30:00Z"
completed_at = "2026-03-21T10:32:15Z"
output_file = "/home/user/.spellbook/job_001.out"
error_file = "/home/user/.spellbook/job_001.err"

[[jobs.jobs]]
id = 2
spell_name = "Build Project"
command = "cargo build --release"
status = "Running"
pid = 12345
started_at = "2026-03-21T10:35:00Z"
output_file = "/home/user/.spellbook/job_002.out"
error_file = "/home/user/.spellbook/job_002.err"
```

### Notes

- **Job ID**: Monotonic counter stored in `next_id` field
- **Concurrency**: Maximum 10 concurrent running jobs
- **Retention**: Keep last 50 jobs, auto-purge on startup
- **Output files**: Plain text, viewable in OutputModal

---

## RecentEntry

Represents a recently used spell.

```rust
pub struct RecentEntry {
    pub spell_name: String,
    pub spell_id: String,       // UUID
    pub timestamp: DateTime<Utc>,
    pub action: RecentAction,
}

pub enum RecentAction {
    Run,
    Copy,
}
```

### Recents Registry (TOML)

Stored in `~/.spellbook/recents.toml`:

```toml
[[recents]]
spell_name = "List Processes"
spell_id = "550e8400-e29b-41d4-a716-446655440000"
timestamp = "2026-03-21T10:30:00Z"
action = "run"

[[recents]]
spell_name = "Collect Garbage"
spell_id = "660e9401-f30c-52e5-b827-557766551111"
timestamp = "2026-03-21T09:15:00Z"
action = "copy"
```

### Notes

- **Retention**: Keep last 100 entries, FIFO eviction
- **Virtual Spellbook**: Recent spellbook is generated from this data
- **Sorting**: Most recent first

---

## Config

User configuration settings.

```toml
view_mode = "auto"          # auto / cards / spines
default_run_mode = "simple"  # simple / tui / background
```

### ViewMode

```rust
pub enum ViewMode {
    Auto,    // Responsive (cards on wide, spines on narrow)
    Cards,   // Large card view
    Spines,  // Compact spine view
}
```

---

## Theme

Theme selection stored in `theme.toml`:

```toml
selected_theme = "dracula"
```

### RatatuiColors

Each theme defines these colors:

```rust
pub struct RatatuiColors {
    pub bg: Color,          // Background
    pub fg: Color,          // Foreground/text
    pub accent: Color,      // Accent/highlight
    pub muted: Color,       // Secondary text
    pub selection: Color,   // Selection highlight
    pub border: Color,      // Border color
}
```

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

---

## Migration Notes

### V1 → V2 Migration

When loading a V1 `codex.toml`:

1. **Add IDs**: Generate UUIDs for spells without `id` field
2. **Update references**: Convert spellbook spell references from names to IDs
3. **Remove deprecated fields**: `elevated`, `dangerous`, `background`
4. **Rewrite file**: Save updated codex with new format

### Forward Compatibility

- All optional fields use `#[serde(default)]`
- Old TOML files load with defaults for missing fields
- New fields added in future versions won't break old readers
