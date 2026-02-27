┌─ SPELLBOOK ─────────────────────────┐
│  ‹ Networking ›                     │
│ ─────────────────────────────────── │
│   ▸ Kill Port                       │
│     Port Scan                       │
│     DNS Lookup                      │
│ ─────────────────────────────────── │
│   lsof -ti:{{port}} | xargs kill -9 │
│                                     │
│   Terminates the daemon bound to    │
│   the specified port.               │
│                                     │
│   network  kill  daemon             │
│ [y] copy  [esc] back  [h/l] cat    │
└─────────────────────────────────────┘

[OPEN]
   │
   ▼
┌─────────────────────┐
│  spellbook          │
│                     │
│  > Networking       │
│    Docker           │     ← Menu 1: Category select
│    Git              │       or type to search
│    SSH              │
│                     │
│  [/] search  [q] quit│
└─────────────────────┘
         │
    select or search
         │
         ▼
┌─────────────────────┐
│  spellbook/ Network │
│                     │
│  > Kill Port        │     ← Menu 2: Canticle browse + detail
│    Port Scan        │       [esc] to go back
│    DNS Lookup       │
│  ─────────────────  │
│  lsof -ti:{{port}}  │
│    | xargs kill -9  │
│                     │
│  Terminates daemon. │
│  network · kill     │
└─────────────────────┘

as a starting point: 
- app.rs
- main.rs
- model.rs
- ui
    - browse.rs
    - home.rs
    - mod.rs
    - theme.rs
