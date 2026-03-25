Comprehensive Manual Testing Checklist
Phase 1: Core Navigation (30 min)
1.1 BrowseSpellbooks Mode
- [X] Arrow keys navigate grid (↑↓←→)
- [ ] Vim keys work (h j k l)
- [X] Enter opens selected spellbook
- [ ] / activates search mode
- [X] : opens command palette
- [ ] Empty state displays correctly
1.2 BrowseSpells Mode
- [X] Up/Down navigates spell list
- [ ] Vim keys work (j k)
- [ ] Back with ←, h, or Esc
- [ ] Search with /
1.3 Mode Switching
- [ ] BrowseSpells → BrowseSpellbooks works
- [ ] AddSpell opens from command palette
- [ ] AddSpellbook opens from command palette
---
Phase 2: Execution Modes (45 min)
2.1 Copy to Clipboard
- [ ] Enter copies incantation to clipboard
- [ ] Visual feedback shows "Copied!"
- [ ] Clipboard contains correct content
2.2 Simple Mode (s key)
- [ ] TUI exits completely
- [ ] Command executes in terminal
- [ ] User returns to shell
- [ ] Recents updated before exit
2.3 TUI Mode (Ctrl+r)
- [ ] Streaming modal appears
- [ ] Output streams in real-time
- [ ] Stderr highlighted differently
- [ ] Exit code shown on completion
2.4 Background Mode (Ctrl+b)
- [ ] Job appears in sidebar
- [ ] Job status shows running (⟳)
- [ ] Notification on completion
2.5 Process Control in Streaming Modal
- [ ] Ctrl+C kills process
- [ ] Ctrl+B promotes to background
- [ ] Auto-scroll toggles with s
- [ ] Manual scroll disables auto-scroll
---
Phase 3: CRUD Operations (45 min)
3.1 Add Spell (:n)
- [ ] Form opens with Name field focused
- [ ] Tab cycles through fields
- [ ] All text fields accept input
- [ ] RunMode dropdown works
- [ ] Confirm checkbox toggles
- [ ] Spellbook dropdown works
- [ ] Ctrl+S saves from any field
- [ ] Enter on last field saves
- [ ] Validation shows errors
3.2 Edit Spell (e key)
- [ ] Form pre-populates with spell data
- [ ] Changes save correctly
- [ ] Cancel returns to list
3.3 Delete Spell (d key)
- [ ] Confirmation dialog appears
- [ ] Yes deletes, No cancels
- [ ] Spell removed from codex
3.4 Toggle Favorite (f key)
- [ ] Star indicator toggles
- [ ] Favorites virtual spellbook updates
3.5 Add Spellbook (:nb)
- [ ] Name, Cover, Sigil fields work
- [ ] Save creates new spellbook
- [ ] New spellbook appears in list
---
Phase 4: Search & Filter (20 min)
4.1 Search Activation
- [ ] / activates in BrowseSpellbooks
- [ ] / activates in BrowseSpells
- [ ] Input field appears
4.2 Search Filtering
- [ ] Real-time filtering works
- [ ] Spellbooks filter by name
- [ ] Spells filter by name/lore/school/glyphs
4.3 Search Navigation
- [ ] Up/Down navigate filtered results
- [ ] Enter selects
- [ ] Esc clears and deactivates
---
Phase 5: Jobs Sidebar (30 min)
5.1 Toggle
- [ ] :jobs opens/closes sidebar
- [ ] Sidebar shows on right side
5.2 Focus Management
- [ ] Tab cycles focus (Main ↔ Sidebar)
- [ ] Visual indicator shows focused component
5.3 Job List
- [ ] Shows all jobs (running, completed, failed)
- [ ] Running shows ⟳ icon
- [ ] Completed shows ✓
- [ ] Failed shows ✗
5.4 Job Actions
- [ ] Up/Down navigates job list
- [ ] Enter views job output
- [ ] k kills running job
- [ ] c cancels queued job
---
Phase 6: Overlays (30 min)
6.1 Help Overlay (?)
- [ ] Opens with ? key
- [ ] Scroll with ↑/↓ or PgUp/PgDn
- [ ] Esc or ? closes
6.2 Command Palette (:)
- [ ] Opens with : key
- [ ] Type filters commands
- [ ] Up/Down navigates
- [ ] Enter executes
- [ ] Esc closes
6.3 Confirm Dialog
- [ ] Appears for delete operations
- [ ] ←/→ or Tab toggles Yes/No
- [ ] Enter confirms
- [ ] y/n quick selects
6.4 Output Modal
- [ ] Shows job output
- [ ] Scroll works
- [ ] Close only when process done
---
Phase 7: Themes & Views (15 min)
7.1 Theme Cycling
- [ ] t cycles themes
- [ ] Visual theme changes
- [ ] Theme persists after restart
7.2 View Modes
- [ ] v cycles (Cards → Spines → Auto)
- [ ] Cards view shows cards
- [ ] Spines view shows spines
- [ ] View preference persists
---
Phase 8: Edge Cases & Error Handling (30 min)
8.1 Empty States
- [ ] First run with no data
- [ ] Spellbook with no spells
8.2 Error Scenarios
- [ ] Invalid spell name (empty)
- [ ] Invalid command (empty)
- [ ] File permission errors
- [ ] Corrupt codex.toml
8.3 Long Output
- [ ] > 10,000 lines truncates
- [ ] Warning shows when truncated
8.4 Long Running
- [ ] Process runs > 1 minute
- [ ] Can kill from modal
- [ ] Can promote to background
---
Phase 9: Persistence (15 min)
9.1 Settings Persistence
- [ ] Theme survives restart
- [ ] View mode survives restart
9.2 Data Persistence
- [ ] New spells appear in codex.toml
- [ ] New spellbooks appear in codex.toml
- [ ] Favorites persist
- [ ] Recents persist (up to 100)
9.3 Job Persistence
- [ ] Jobs survive app restart
- [ ] Job output files exist
---
Phase 10: Quick Reference Verification
Walk through each key on this cheat sheet and verify it works:
Navigation       Actions           Modes
↑↓←→ / hjkl      Enter: open/copy  r: run default
Tab: focus       e: edit           s: simple run
Esc: back        d: delete         ^r: TUI run
                  f: favorite       ^b: background
Search & Help    Commands          System
/: search        :: palette        t: theme
?: help          :n: new spell     v: view mode
                  :jobs: sidebar    q: quit
