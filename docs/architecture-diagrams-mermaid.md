# Spellbook Architecture Diagrams

> Visual reference for the Spellbook codebase. These diagrams use [Mermaid](https://mermaid.js.org/) syntax and render natively in GitHub, GitLab, and most modern Markdown viewers.

---

## 1. Module Dependency Graph

```mermaid
flowchart TB
    subgraph Binary
        main[src/main.rs]
    end

    cli[src/cli.rs<br/>CLI args]
    logging[src/logging.rs<br/>log macros]
    clipboard[src/clipboard.rs<br/>copy + ExecutionResult]

    main --> cli
    main --> logging
    main --> archivist
    main --> state
    main --> ui
    main --> invoker

    state[src/state.rs<br/>State + CRUD]
    state --> archivist
    state --> models

    archivist[src/archivist/archivist.rs<br/>Archivist]
    archivist --> models
    archivist --> validation

    validation[src/validation.rs<br/>validate_codex]
    validation --> models

    models[src/models/<br/>Spell, Spellbook, Codex, Job, Theme]

    invoker[src/invoker/mod.rs<br/>JobManager + execution]
    invoker -.->|notify-send| dbus[D-Bus notifications]

    ui[src/ui/<br/>render, events, components]
    ui --> state
    ui --> models
    ui --> clipboard
    ui --> invoker

    style main fill:#4A148C,stroke:#000,color:#fff
    style ui fill:#00796B,stroke:#000,color:#fff
    style invoker fill:#00796B,stroke:#000,color:#fff
    style state fill:#00796B,stroke:#000,color:#fff
```

---

## 2. Data Model

```mermaid
classDiagram
    direction LR

    class Codex {
        +Vec~Spell~ spells
        +Vec~Spellbook~ spellbooks
    }

    class Spell {
        +String id
        +String name
        +String incantation
        +String lore
        +String school
        +Vec~String~ glyphs
        +bool confirm
        +RunMode run_mode
        +String working_dir
        +bool favorite
    }

    class Spellbook {
        +String name
        +String cover
        +String sigil
        +Vec~String~ spell_ids
        +Option~SpineStyle~ style
        +Option~Color~ color
    }

    class SpellbookRef {
        <<enumeration>>
        Virtual(VirtualKind)
        Codex(usize)
    }

    class VirtualKind {
        <<enumeration>>
        Favorites
        Recent
    }

    class RunMode {
        <<enumeration>>
        Simple
        Tui
        Background
    }

    class Job {
        +u64 id
        +String spell_name
        +String command
        +JobStatus status
        +Option~u32~ pid
        +Option~i32~ exit_code
        +DateTime~Utc~ started_at
        +Option~DateTime~ completed_at
        +PathBuf output_file
        +PathBuf error_file
    }

    class JobStatus {
        <<enumeration>>
        Queued
        Running
        Completed
        Failed
        Cancelled
    }

    class RecentEntry {
        +String spell_id
        +String spell_name
        +DateTime~Utc~ timestamp
        +RecentAction action
    }

    class RecentAction {
        <<enumeration>>
        Run
        Copy
    }

    class Theme {
        <<enumeration>>
        DarkDefault, LightDefault, Dracula, ...
    }

    class RatatuiColors {
        +Color bg
        +Color fg
        +Color accent
        +Color muted
        +Color selection
        +Color border
    }

    class UserSettings {
        +ViewMode view_mode
    }

    class ViewMode {
        <<enumeration>>
        List
        Cards
        Spines
    }

    Codex "1" *-- "*" Spell
    Codex "1" *-- "*" Spellbook
    Spellbook "*" ..> "*" Spell : references by id
    SpellbookRef ..> VirtualKind
    JobManager "1" *-- "*" Job
    RecentEntry ..> RecentAction
    Theme --> RatatuiColors : colors()
    UserSettings --> ViewMode
```

---

## 3. UI State Hierarchy

```mermaid
flowchart TB
    UiState[UiState]

    UiState --> Mode[Mode]
    UiState --> Overlays[overlays: Vec~Overlay~]
    UiState --> Focus[focus: FocusTarget]
    UiState --> Sidebar[jobs_sidebar_open]
    UiState --> Streaming[streaming_modal]
    UiState --> AddSpell[add_spell: AddSpellForm]
    UiState --> AddSpellbook[add_spellbook: AddSpellbookForm]

    Mode --> BrowseSpellbooks[BrowseSpellbooks]
    Mode --> BrowseSpells[BrowseSpells]
    Mode --> AddSpellMode[AddSpell]
    Mode --> EditSpell[EditSpell]
    Mode --> AddSpellbookMode[AddSpellbook]

    BrowseSpellbooks --> BrowseState
    BrowseSpells --> BrowseState

    BrowseState --> Idle[Idle]
    BrowseState --> Searching[Searching<br/>query + filtered_indices]
    BrowseState --> Viewing[Viewing<br/>spellbook_index + spell_list_state]

    AddSpellMode --> FormState
    EditSpell --> FormState
    AddSpellbookMode --> FormState

    FormState --> Idle2[Idle]
    FormState --> Editing[Editing<br/>FormField]

    Overlays --> OutputModal[OutputModal]
    Overlays --> ConfirmDialog[ConfirmDialog]
    Overlays --> CommandPalette[CommandPalette]
    Overlays --> Help[Help]
    Overlays --> InputPopup[InputPopup]
    Overlays --> SpellDetails[SpellDetails]

    Focus --> Main[Main]
    Focus --> JobsSidebar[JobsSidebar]

    style UiState fill:#4A148C,stroke:#000,color:#fff
    style Mode fill:#00796B,stroke:#000,color:#fff
    style Overlays fill:#00796B,stroke:#000,color:#fff
```

---

## 4. Event Handling Priority

```mermaid
flowchart TD
    A[Key event received] --> B[handle_event]

    B --> C{Active overlay?}
    C -->|Yes| D[Route to top overlay handler]
    D --> E{Event consumed?}
    E -->|Yes| Z[Done]
    E -->|No| F

    C -->|No| F{Sidebar focused?}
    F -->|Yes| G[jobs::handle_jobs_key]
    G --> H{Consumed?}
    H -->|Yes| Z
    H -->|No| I

    F -->|No| I{Global keybind?}
    I -->|Yes| J[Ctrl+C / q / t / v / ? / Tab]
    J --> K{Consumed?}
    K -->|Yes| Z
    K -->|No| L

    I -->|No| L[Route to current Mode handler]
    L --> M[BrowseSpellbooks / BrowseSpells / AddSpell / EditSpell / AddSpellbook]
    M --> Z

    style A fill:#4A148C,stroke:#000,color:#fff
    style Z fill:#4A148C,stroke:#000,color:#fff
```

---

## 5. Spell Execution Flow

```mermaid
flowchart TD
    A[User invokes spell<br/>r / s / Ctrl+r / Ctrl+b] --> B{confirm = true?}
    B -->|Yes| C[Push ConfirmDialog overlay]
    C --> D[User confirms]
    B -->|No| E[Determine run mode]
    D --> E

    E --> F{RunMode}
    F -->|Simple| G[State.add_recent]
    G --> H[Archivist.save_recents]
    H --> I[Leave alternate screen]
    I --> J[execvp $SHELL -c]
    J --> K[Process replaced<br/>user back at shell]

    F -->|Tui| L[invoker::stream_command]
    L --> M[Spawn child with pipes]
    M --> N[Threads read stdout/stderr]
    N --> O[mpsc channel to UI]
    O --> P[OutputModal streams output]
    P --> Q{Controls}
    Q -->|Ctrl+C| R[Kill process]
    Q -->|Ctrl+B| S[Promote to background job]
    Q -->|Esc| T[Close modal]

    F -->|Background| U[invoker::start_spell]
    U --> V[Create Job entry]
    U --> W[Spawn nohup detached process]
    U --> X[Save jobs.toml]
    V --> Y[Job appears in sidebar]
    W --> Zz[Poller monitors PID]
    Zz --> AA[notify-send on finish]

    style A fill:#4A148C,stroke:#000,color:#fff
    style K fill:#00796B,stroke:#000,color:#fff
    style T fill:#00796B,stroke:#000,color:#fff
    style AA fill:#00796B,stroke:#000,color:#fff
```

---

## 6. Job Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Queued : start_spell()
    Queued --> Running : spawn_detached()
    Running --> Completed : ps shows process gone
    Running --> Failed : ps shows process gone
    Running --> Cancelled : kill/cancel_job()
    Completed --> [*] : cleanup_completed()
    Failed --> [*] : cleanup_completed()
    Cancelled --> [*] : cleanup_completed()

    note right of Running
        Background thread polls every 1s
        via `ps -p $PID`
    end note
```

---

## 7. Application Startup Sequence

```mermaid
sequenceDiagram
    actor User
    participant main as main.rs
    participant cli as cli.rs
    participant log as logging.rs
    participant arch as Archivist
    participant state as State
    participant inv as JobManager
    participant ui as UiState

    User->>main: cargo run [--add]
    main->>cli: parse args
    main->>log: init_logging()
    main->>arch: load codex.toml
    alt missing / invalid
        arch-->>main: create empty codex
    end
    main->>arch: load config.toml
    main->>arch: load user settings
    main->>arch: load recents.toml
    main->>state: State::new(codex)
    main->>inv: init_job_manager(launch_dir)
    inv->>inv: spawn polling thread
    main->>ui: UiState::new(mode)
    loop Event loop
        main->>ui: render(frame)
        main->>main: poll crossterm 100ms
        main->>ui: handle_event(key)
    end
```

---

## 8. Persistence Layer

```mermaid
flowchart LR
    subgraph LocalDir["Working directory"]
        codex[codex.toml]
        theme[config.toml]
    end

    subgraph SpellbookDir["~/.spellbook/"]
        jobs[jobs.toml]
        recents[recents.toml]
        out[job_*.out]
        err[job_*.err]
        log[spellbook.log]
    end

    Archivist -->|load/save| codex
    Archivist -->|load/save| theme
    Archivist -->|load/save| jobs
    Archivist -->|load/save| recents
    JobManager -->|write| out
    JobManager -->|write| err
    logging -->|append| log

    style codex fill:#4A148C,stroke:#000,color:#fff
    style jobs fill:#00796B,stroke:#000,color:#fff
    style recents fill:#00796B,stroke:#000,color:#fff
```

---

## 9. Virtual Spellbooks

```mermaid
flowchart TD
    A[All spells in codex.spells] --> B{Generate virtual spellbooks}

    B --> C[Favorites]
    C --> D[Filter spells where favorite = true]

    B --> E[Recent]
    E --> F[Load ~/.spellbook/recents.toml]
    F --> G[Map spell_id to Spell]
    G --> H[Sort by timestamp desc]
    H --> I[Limit 100]

    B --> J[Codex spellbooks]

    K[Rendered spellbook list] --> C
    K --> E
    K --> J

    style K fill:#4A148C,stroke:#000,color:#fff
```

---

## 10. Rendering Pipeline

```mermaid
flowchart TD
    A[terminal.draw] --> B[ui::render]
    B --> C{jobs_sidebar_open?}
    C -->|Yes| D[split layout horizontally]
    D --> E[render_mode in main area]
    D --> F[render jobs panel in sidebar]
    C -->|No| E
    E --> G{match Mode}
    G -->|Browse*| H[search_overlay::render_in_area]
    G -->|Add/Edit Spell| I[add_spell::render_in_area]
    G -->|Add Spellbook| J[add_spellbook_form::render_in_area]
    E --> K[render overlays on top]
    K --> L{match Overlay}
    L -->|OutputModal| M[streaming_modal / output popup]
    L -->|ConfirmDialog| N[confirm popup]
    L -->|Help| O[help overlay]
    L -->|InputPopup| P[input popup]
    L -->|SpellDetails| Q[spell details popup]
    B --> R[render loading indicator if active]

    style A fill:#4A148C,stroke:#000,color:#fff
    style B fill:#00796B,stroke:#000,color:#fff
```

---

## How to render these locally

If you want PNG/SVG exports, install the Mermaid CLI and run:

```bash
# Using npx
npx @mermaid-js/mermaid-cli -i docs/architecture-diagrams-mermaid.md -o out.svg

# Or with bun
bunx @mermaid-js/mermaid-cli -i docs/architecture-diagrams-mermaid.md -o out.svg
```

For a single diagram, extract the diagram block into a `.mmd` file and run:

```bash
mmdc -i module-graph.mmd -o module-graph.png
```

---

## Key architectural takeaways

1. **Single-mode navigation**: One `Mode` is active at a time; state is nested inside mode variants.
2. **Overlay stack**: Multiple overlays can be pushed; the topmost one gets first chance at input.
3. **Jobs sidebar is not an overlay**: It's a persistent panel with its own `FocusTarget`.
4. **Event priority**: Overlay → Sidebar (if focused) → Global keys → Mode handler.
5. **Atomic persistence**: All TOML writes use `write_to_temp + fs::rename`.
6. **UUID references**: Spells are referenced by ID, not name, enabling safe renames.
7. **Execution modes**: Simple uses `exec()` to replace the process; TUI streams via mpsc; Background uses `nohup` and a polling thread.
8. **Static job manager**: `OnceLock<JobManager>` provides global access after `init_job_manager()` in `main`.
