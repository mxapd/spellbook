# Architecture Refactor Plan [ARCHIVED - COMPLETE]

> **STATUS: REFACTOR COMPLETED** - All changes outlined in this plan have been implemented. See refactor-completion.md for final status.

## Architecture Style: Elm Architecture

This refactor moves the codebase toward the [Elm Architecture](https://guide.elm-lang.org/architecture/) pattern:

```
     ┌─────────────────────────────────────────┐
     │                  Model                   │
     │  (Mode enum with nested state)          │
     └─────────────────┬───────────────────────┘
                       │
     ┌─────────────┐   │   ┌─────────────┐
     │    View     │◄──┴──►│   Update    │
     │ (render_*)  │        │ (handlers)  │
     └─────────────┘        └─────────────┘
```

**Elm Core Concepts → Our Implementation:**

| Elm | Our TUI |
|-----|---------|
| Model | `Mode` enum with nested `BrowseState`, `FormState` |
| View | Existing `render_*` functions |
| Update | Event handlers split by mode |
| Messages | `KeyCode` events |

## Current Problems

### The Core Issue: Parallel State Machines

The codebase has evolved to have **multiple overlapping state flags** that don't know about each other:

```rust
// UiState has these separate, interacting fields:
pub struct UiState {
    mode: Mode,                      // what "screen" am I on
    search_mode: SearchMode,          // what "search state" am I in (DIFFERENT enum!)
    is_typing: bool,                  // am I in text input?
    search: SearchState {              // separate struct
        search_active: bool,
        query: String,
        // ...
    }
    showing_spellbooks: bool,         // which list is visible?
    // ... 20+ more fields
}
```

Every event handler must manually check ALL these flags and hope it gets the combination right. This is why:
- `handle_search` is **2,482 lines** - it's not a function, it's a state machine
- The implicit/explicit search toggle bug took **many hours** to diagnose
- Adding new features is risky because interactions are unpredictable

### Event Handler Bloat

```
main.rs
  → handle_event()
    → handle_overlay()           # if overlay active
    → handle_jobs_key()          # if sidebar focused  
    → handle_mode()              # dispatch to mode handler
      → handle_search()           # 2,482 lines! handles BOTH modes
    → handle_global_keys()
```

## Target Architecture (Elm-Style)

### Single Source of Truth: Mode Contains Everything

```rust
// Mode now CONTAINS its state - the Elm "Model"
// This is our single source of truth

enum Mode {
    BrowseSpellbooks(BrowseState),
    BrowseSpells(BrowseState),
    AddSpell(FormState),
    EditSpell(FormState),
    AddSpellbook(FormState),
}

// Nested state for each mode (Elm's Model composition)
enum BrowseState {
    Idle,
    Searching(String),   // query lives here, not in a separate struct
}

enum FormState {
    Idle,
    Editing(FieldId),    // which field has focus
}
```

### Derived State (No More Duplication) - Elm's `view` function computes derived state

Instead of storing `is_typing`, derive it from Mode:

```rust
fn is_typing(mode: &Mode) -> bool {
    matches!(mode,
        Mode::BrowseSpells(BrowseState::Searching(_)) |
        Mode::BrowseSpellbooks(BrowseState::Searching(_)) |
        Mode::AddSpell(FormState::Editing(_)) |
        Mode::EditSpell(FormState::Editing(_))
    )
}
```

### Event Router Pattern (Elm's "Update")

events.rs becomes an Elm-style "Update" function (~400 lines total):

```
handle_event()
  → overlays first              (50 lines)
  → match mode                 (20 lines dispatch)
    → browse_spellbooks.rs     (200 lines)
    → browse_spells.rs         (200 lines)
    → form.rs                  (150 lines, shared by add/edit)
  → global_keys()              (50 lines)
```

## Migration Steps

### Step 1: Freeze events.rs
**Before starting any refactor:**
- No new features in events.rs
- Any new behavior goes in a new file, called from events.rs
- This prevents the codebase from getting worse while we fix it

### Step 2: Collapse Search State (HIGHEST PRIORITY)
**Target: Fix the implicit/explicit search bug cleanly**

1. Add `BrowseState` enum to Mode variants
2. Move `search.query` into `BrowseState::Searching(String)`
3. Remove: `search_mode`, `search.search_active`, `is_typing` (derived)
4. Let compiler find all references - each error is a migration item
5. Delete old flags as they become unreferenced

**After this step:** The search bug becomes a 10-line change

### Step 3: Split events.rs by Mode
**Once state is clean, the split is mechanical:**

1. Create `src/ui/browse_spellbooks.rs` - move handler code there
2. Create `src/ui/browse_spells.rs` - move handler code there
3. Create `src/ui/form.rs` - shared by AddSpell/EditSpell/AddSpellbook
4. events.rs becomes a simple router calling these modules

### Step 4: Audit Remaining UiState Fields
Go through every remaining field in UiState and ask:
- Does this belong inside a Mode variant?
- Is it genuinely global (like overlays)?
- Can it be derived from Mode?

Most fields will want to move inward.

## Expected Benefits

1. **Predictable behavior** - State transitions are explicit, not emergent
2. **Testability** - Each mode handler can be unit tested in isolation
3. **Feature velocity** - Adding new features won't cause mysterious bugs
4. **Code size** - events.rs: 2,482 → ~400 lines
5. **Onboarding** - New developers can understand one small file, not a monolith

## The One Thing To Do Today

Define the new enum and make it compile, without changing any logic:

```rust
enum BrowseState {
    Idle,
    Searching(String),
}
```

Add it to Mode::BrowseSpells and Mode::BrowseSpellbooks. Let the compiler tell you everywhere the old state is being read or written. That error list is your migration checklist.

## Files That Need Changes

### To Create
- `src/ui/browse_spellbooks.rs` - spellbook browsing handler
- `src/ui/browse_spells.rs` - spell list handler  
- `src/ui/form.rs` - shared form handling

### To Modify
- `src/ui/mod.rs` - Mode enum, UiState
- `src/ui/events.rs` - collapse into router
- `src/models/theme.rs` - may need BrowseState reference

### To Delete (after migration)
- `src/ui/search_state.rs` - state moves into Mode
- Many boolean fields from UiState
