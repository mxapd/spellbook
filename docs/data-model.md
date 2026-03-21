# Data Model

## Codex

Root data structure containing all spells and spellbooks.

## Spell

A single command snippet.

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Display name (e.g., "List Processes") |
| `incantation` | String | The actual command(s) to run |
| `lore` | String | Description/usage notes |
| `school` | String | Category (e.g., "System", "Network") |
| `glyphs` | Vec<String> | Search tags for filtering |
| `elevated` | bool | Requires privilege escalation (sudo) |
| `dangerous` | bool | Destructive operation, show warning |
| `confirm` | bool | Require user confirmation before running |
| `working_dir` | String | Directory to run command in (optional) |

**Note:** The `id` field is generated automatically on load (1, 2, 3...).
Metadata fields (`elevated`, `dangerous`, `confirm`, `working_dir`) are optional and default to false/empty.

### Example (TOML)

```toml
[[spells]]
name = "List Processes"
incantation = "ps aux"
lore = "Shows all running processes."
school = "System"
glyphs = ["process", "running", "ps", "list"]

[[spells]]
name = "Collect Garbage"
incantation = "sudo nix-collect-garbage -d"
lore = "Removes unused Nix store entries."
school = "NixOS"
glyphs = ["nix", "garbage", "cleanup"]
elevated = true
dangerous = false
confirm = true
```

## Spellbook

A named collection of spells.

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Display name |
| `cover` | String | Description/purpose |
| `sigil` | String | Emoji or symbol for visual flair |
| `spells` | Vec<String> | References to spell names |

### Example (TOML)

```toml
[[spellbooks]]
name = "System"
cover = "System monitoring commands."
sigil = "💻"
spells = ["List Processes", "Kill Process"]
```

## Notes

- Spell references in spellbooks are by **name**, not by ID
- IDs are generated automatically when the file is loaded
- Data is stored in `codex.toml` (TOML format)
- Theme preference stored in `theme.toml` (`selected_theme` index 0-9)

## Theme Configuration

Stored in `theme.toml`:

```toml
# Theme index: 0=default, 1=default-light, 2=dracula, etc.
selected_theme = 0
```

Each theme defines 16-color ANSI indices:
- bg, fg, accent, muted, selection, border

## Job

Represents a running or completed command.

| Field | Type | Description |
|-------|------|-------------|
| `id` | u64 | Auto-incremented job ID |
| `name` | String | Spell name or command preview |
| `command` | String | Full command that was executed |
| `status` | JobStatus | Current state |
| `pid` | Option<u32> | Process ID (if running) |
| `exit_code` | Option<i32> | Exit code (if completed) |
| `started_at` | DateTime | When the job started |
| `completed_at` | Option<DateTime> | When the job finished |
| `elevated` | bool | Whether privilege escalation was used |
| `output_file` | String | Path to stdout file |
| `error_file` | String | Path to stderr file |

### JobStatus Enum

```rust
enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

### Job Registry (TOML)

Jobs are persisted to `~/.spellbook/jobs.toml`:

```toml
[jobs]
next_id = 42

[[jobs.jobs]]
id = 1
name = "Collect Garbage"
command = "sudo nix-collect-garbage -d"
status = "Completed"
exit_code = 0
started_at = 2026-03-21T10:30:00Z
completed_at = 2026-03-21T10:32:15Z
elevated = true
output_file = "job_001.out"
error_file = "job_001.err"
```
