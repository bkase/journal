# Implement View Layer

**Status:** Done
**Agent PID:** 15170

## Original Todo

## 2. Implement View Layer

**Priority:** Critical - Fixes Architecture  
**Goal:** Separate presentation from business logic following Elm architecture.

### Steps

1. Create `view.rs` module with `fn view(state: &State)`
2. Move all UI rendering logic to view functions:
   - Welcome messages
   - Prompts
   - Session status
   - Error displays
3. Remove ALL `println!`/`eprintln!` from `effects.rs`
4. Remove `Effect::Show...` variants
5. Update main loop:

   ```rust
   let action = effects::execute(effect, &mut runner)?;
   let (new_state, new_effect) = state::update(state, action);
   view::view(&new_state);  // NEW
   state = new_state;
   effect = new_effect;
   ```

6. Convert show effects to proper state transitions

### Testing Strategy

- Ensure all UI elements still display correctly
- Verify no presentation logic remains in effects.rs
- Test all state transitions produce appropriate views
- Verify error states display properly

## Description

We'll create a separate `view.rs` module that handles all UI rendering logic, following the Elm architecture pattern. This will move all `println!`, `eprintln!`, and display-related effects out of `effects.rs` into a dedicated view layer. The view will render based on the current `State`, making the presentation layer completely separate from business logic.

## Implementation Plan

- [x] Create src/journal/src/view.rs module with view(state: &State) function
- [x] Move all println!/eprintln! from effects.rs lines 118-151, 380-383, 387-390 to view functions
- [x] Create view functions for each state: render_initializing, render_prompting_for_new, render_in_session, render_analyzing, render_done, render_error
- [x] Remove Effect::ShowModePrompt, ShowQuestion, ShowCoachResponse, ShowMessage, ShowError, ShowAnalysis variants from effects.rs
- [x] Update effects.rs run_effect() to remove show_* method calls
- [x] Add view module to main.rs imports
- [x] Update main.rs JournalApp::run() loop to call view::view(&self.state) after state updates
- [x] Remove println! statements from main.rs lines 41, 45, 119, 124, 189-192
- [x] Update update.rs to return proper states instead of Show effects
- [x] Test all UI elements display correctly in each state
- [x] Verify no presentation logic remains in effects.rs

## Notes

[Implementation notes]