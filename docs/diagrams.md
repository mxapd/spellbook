# Spellbook Architecture Diagrams

This document contains architecture diagrams in Mermaid format. View in any Markdown viewer that supports Mermaid (GitHub, VS Code with extension, etc.).

## Module Structure

```mermaid
graph TB
    subgraph "src/"
        main["main.rs"]
        lib["lib.rs"]
        
        subgraph "models/"
            spell["spell.rs"]
            spellbook["spellbook.rs"]
            theme["theme.rs"]
            codex["codex.rs"]
            mod_models["mod.rs"]
        end
        
        subgraph "ui/"
            events["events.rs"]
            render["render.rs"]
            search_overlay["search_overlay.rs"]
            jobs["jobs.rs"]
            confirm["confirm.rs"]
            add_spell["add_spell.rs"]
            spell_list["spell_list.rs"]
            mod_ui["mod.rs"]
        end
        
        subgraph "persistence/"
            archivist["archivist.rs"]
        end
        
        executor["executor.rs"]
        clipboard["clipboard.rs"]
        state["state.rs"]
        validation["validation.rs"]
        logging["logging.rs"]
        cli["cli.rs"]
    end
    
    main --> lib
    main --> state
    main --> ui
    main --> executor
    main --> persistence
    main --> cli
    main --> logging
    
    lib --> models
    lib --> ui
    lib --> persistence
    lib --> validation
    lib --> executor
    lib --> clipboard
    lib --> state
    lib --> logging
    lib --> cli
```

## Data Models

```mermaid
classDiagram
    class Codex {
        +Vec~Spell~ spells
        +Vec~Spellbook~ spellbooks
    }
    
    class Spell {
        +u64 id
        +String name
        +String incantation
        +String lore
        +String school
        +Vec~String~ glyphs
        +bool elevated
        +bool dangerous
        +bool confirm
        +bool background
        +String working_dir
        +requires_confirmation() bool
    }
    
    class Spellbook {
        +String name
        +String cover
        +String sigil
        +Vec~u64~ spell_ids
        +Vec~String~ spells
        +Option~SpineStyle~ style
    }
    
    class SpineStyle {
        +StarsAndDiamonds
        +Celestial
        +DotsAndTherefore
        +Alchemy
        +Geometric
        +Minimal
    }
    
    class Theme {
        +DarkDefault
        +LightDefault
        +Dracula
        +GruvboxDark
        +GruvboxLight
        +Nord
        +CatppuccinMocha
        +OneDark
        +SolarizedDark
        +SolarizedLight
    }
    
    class ViewMode {
        +List
        +Cards
        +Spines
    }
    
    Codex "1" --> "*" Spell : contains
    Codex "1" --> "*" Spellbook : contains
    Spellbook "1" --> "*" Spell : references by id
    Spellbook --> SpineStyle
    Theme --> RatatuiColors
    ViewMode ..> Theme : cycling
```

## Job System Architecture

```mermaid
flowchart LR
    subgraph "Spell Execution Flow"
        A["User selects spell<br/>Alt+Enter or Alt+Shift+Enter"] --> B{background flag?}
        B -->|false| C["Run sync<br/>show output popup"]
        B -->|true| D["Run detached as job"]
        
        D --> E["JobManager.start()"]
        E --> F["Spawn nohup process"]
        F --> G["Save to jobs.toml"]
        
        C --> H["Wait for completion"]
        H --> I["Display output"]
        
        I --> J{"User presses 'b'?"}
        J -->|yes| K["Kill process"]
        K --> L["Save background=true to codex.toml"]
        L --> D
        
        G --> M["Background polling thread"]
        M --> N{"Process done?"}
        N -->|yes| O["Send DBus notification"]
    end
```

## Job Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Queued
    Queued --> Running : start()
    Running --> Completed : process ends (exit 0)
    Running --> Failed : process ends (non-zero)
    Running --> Cancelled : kill()
    Cancelled --> [*]
    Completed --> [*]
    Failed --> [*]
    Queued --> Cancelled : cancel()
```

## UI Screens

```mermaid
flowchart TD
    subgraph "Screen Enum"
        Screen["Screen"]
        Screen --> SearchOverlay
        Screen --> OutputPopup
        Screen --> JobsPanel
        Screen --> ConfirmDialog
        Screen --> AddSpell
    end
    
    subgraph "SearchOverlay Mode"
        Mode["SearchMode"]
        Mode --> BrowseSpellbooks
        Mode --> BrowseSpells
        Mode --> AddSpell
        Mode --> AddSpellbook
    end
```

## Event Handling Flow

```mermaid
sequenceDiagram
    participant User
    participant Terminal
    participant Events
    participant Executor
    participant UI
    participant Jobs
    
    User->>Terminal: Key press
    Terminal->>Events: KeyEvent
    Events->>Events: handle_event()
    
    alt Alt+Enter
        Events->>Events: execute_search_result()
        Events->>Events: start_spell_execution()
        alt Sync execution
            Events->>Executor: execute_sync()
            Executor-->>Events: SyncExecutionResult
            Events->>UI: show_output_popup()
        else Background execution
            Events->>Executor: start_spell()
            Executor->>Jobs: Add job
            Jobs-->>Events: job_id
            Events->>UI: copy_feedback
        end
    end
    
    alt OutputPopup
        User->>Terminal: 's' or 'b'
        Terminal->>Events: KeyEvent
        Events->>Events: handle_output_popup()
        alt Save output
            Events->>UI: save_to_file()
        else Move to background
            Events->>Persistence: update_spell_background()
            Events->>Executor: kill_process()
            Events->>Executor: start_spell()
            Events->>UI: hide_output_popup()
        end
    end
    
    alt Alt+Shift+Enter
        Events->>Events: execute with force_background=true
        Events->>Executor: start_spell()
    end
```

## Persistence Flow

```mermaid
flowchart TD
    subgraph "Loading"
        A["codex.toml"] --> B["Archivist::load()"]
        B --> C["Parse TOML"]
        C --> D["Generate spell IDs"]
        D --> E["Resolve spellbook references"]
        E --> F["validate_codex()"]
        F --> G{"Valid?"}
        G -->|yes| H["Codex"]
        G -->|no| I["Error"]
    end
    
    subgraph "Saving"
        J["Spell update"] --> K["update_spell_background()"]
        K --> L["Read codex.toml"]
        L --> M["Parse lines"]
        M --> N["Find spell section"]
        N --> O{"background line exists?"}
        O -->|yes| P["Update existing"]
        O -->|no| Q["Insert new line"]
        P --> R["Write codex.toml"]
        Q --> R
    end
```

## File Organization

```mermaid
graph TD
    subgraph "Root"
        Cargo["Cargo.toml"]
        codex["codex.toml"]
        rustfmt["rustfmt.toml"]
        lib["src/lib.rs"]
        main["src/main.rs"]
    end
    
    subgraph "src/"
        models["models/"]
        ui["ui/"]
        persistence["persistence/"]
        executor["executor.rs"]
        clipboard["clipboard.rs"]
        state["state.rs"]
        validation["validation.rs"]
        logging["logging.rs"]
        cli["cli.rs"]
    end
    
    subgraph "tests/"
        integration["integration tests"]
    end
    
    subgraph "docs/"
        arch["architecture.md"]
        data["data-model.md"]
        diagrams["diagrams.md"]
    end
```

## Command Execution Options

```mermaid
flowchart LR
    subgraph "Execution Modes"
        A["Alt+Enter<br/>Sync"] 
        B["Alt+Shift+Enter<br/>Background"]
        C["'b' key<br/>Move to Background"]
    end
    
    A --> D["Quick commands"]
    A --> E["Show output immediately"]
    
    B --> F["Long-running commands"]
    B --> G["Don't block TUI"]
    
    C --> H["Save preference"]
    C --> I["Kill + restart detached"]
    
    D --> J["ps, ls, git status"]
    F --> K["nixos-rebuild, builds"]
```
