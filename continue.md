Thinking: The user wants:
1. No newline after the command
2. If no TTY available, fall back to clipboard copy (not fail silently)
This is a simple approach. Let me think through the implementation:
1. When Alt+Enter is pressed in simplified mode:
   - Try to open /dev/tty
   - If successful: write command to TTY
   - If fails: fall back to clipboard copy + show feedback
2. The TUI can exit after writing (or not - user said "can die or whatever it doesn't really matter")
Let me draft a plan:
Files to modify:
1. src/ui/events.rs - Modify execute_search_result() (Alt+Enter handler)
   - When in simplified mode: try to send to terminal first, fall back to clipboard
2. Maybe add a helper function in src/clipboard.rs or src/executor.rs:
   - send_to_terminal(command: &str) -> bool returns true if successful
Implementation:
// In clipboard.rs or executor.rs
pub fn send_to_terminal(command: &str) -> bool {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    match OpenOptions::new().write(true).open("/dev/tty") {
        Ok(mut tty) => {
            tty.write_all(command.as_bytes()).is_ok()
        }
        Err(_) => false,
    }
}
// In events.rs - modify the simplified mode handling
// In start_spell_execution or in execute_search_result
Wait, I need to think about where this logic goes. The simplified mode handling is in start_spell_execution(). Currently it just copies to clipboard. We need to modify it so that:
- If Alt+Enter (not just any execution): try terminal, fall back to clipboard
- If Enter (regular): just clipboard
But looking at the current code flow, start_spell_execution() receives a force_background parameter but that doesn't indicate whether it's Alt+Enter or regular Enter. I need to track which key was pressed so I can differentiate between trying the terminal on Alt+Enter versus copying to clipboard on regular Enter. Looking at how the handlers call into start_spell_execution(), I see that execute_search_result() gets invoked for both Alt+Enter (with force_background=true) and regular Enter (with force_background=false). The start_spell_execution() function receives this flag but uses it only to determine background execution behavior in experimental mode, not to distinguish between these two paths. I need to reconsider the logic here—the force_background parameter might not be the right signal for what I'm trying to achieve.
The cleanest approach is to add a dedicated parameter to start_spell_execution() that indicates whether to attempt terminal output. This way I can differentiate between the two execution paths without conflating background job handling with terminal display logic.
In the handler, I'd check state.user_settings.experimental_mode to determine the behavior. Simplified mode would try sending to the terminal first, falling back to clipboard if that fails. Experimental mode would bypass this and use the existing execution logic. I should also verify the spell list handler calls execute_spell_at_index rather than execute_search_result, which means I'd need to update that path too for consistency. The cleanest approach is to create a helper function try_send_to_terminal() that attempts to write to /dev/tty and falls back to clipboard copy if it fails, returning true only if terminal writing succeeded. Then in simplified mode, call this from start_spell_execution() to handle the "send to terminal" logic, while Alt+Enter and regular Enter in search can just invoke it directly without duplicating the terminal-sending logic. When in simplified mode and try_send_to_terminal succeeds, exit. Otherwise show the clipboard feedback. This way the function handles both the fallback behavior and controls whether the TUI terminates based on success or failure.
Alternatively, since the user mentioned the TUI can exit or not, I could keep the existing clipboard copy in simplified mode and introduce a separate Alt+Shift+Enter binding for terminal-send behavior. This avoids modifying the current simplified mode logic and lets the user choose between clipboard copying and direct terminal sending based on the key combination used. So I need to create a helper function that attempts to send to the terminal and falls back to clipboard if that fails. Then modify the simplified mode logic to check experimental_mode and either send to terminal or copy to clipboard, with the helper returning whether terminal worked. Finally, I should ensure this applies consistently across all the entry points where spells can be executed.
The main changes involve adding try_send_to_terminal() in clipboard.rs or executor.rs, updating the simplified mode handling in start_spell_execution(), and making sure Alt+Enter behaves the same way in both the search and spell list views.
Plan: Send to Terminal Feature
Files to Modify
1. src/clipboard.rs - Add helper function:
pub fn try_send_to_terminal(command: &str) -> bool {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    match OpenOptions::new().write(true).open("/dev/tty") {
        Ok(mut tty) => tty.write_all(command.as_bytes()).is_ok(),
        Err(_) => false,
    }
}
2. src/ui/events.rs - Modify start_spell_execution():
- Add send_to_terminal: bool parameter
- Simplified mode logic:
  - If send_to_terminal: try terminal, fall back to clipboard copy
  - If not send_to_terminal: clipboard copy only
- Experimental mode: existing behavior (unchanged)
3. Update call sites in start_spell_execution():
- Alt+Enter handlers: pass send_to_terminal=true
- Regular Enter: pass send_to_terminal=false
Behavior Summary
Key	Simplified Mode
Enter	Copy to clipboard
Alt+Enter	Try terminal, fall back to clipboard, exit
Questions
1. Should Alt+Enter in simplified mode exit the TUI after sending to terminal (or attempting)?
2. Should this apply to spell list (Alt+Enter on spell inside spellbook) too?
