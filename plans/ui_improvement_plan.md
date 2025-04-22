# UI Improvement Plan (Generated 2025-04-22)

Based on analysis of `specs/system_specification.txt` and `src/ui.rs`, the
following potential improvements are suggested:

1. **Refactor `ui` Function into Smaller Helpers:**
   - **Problem:** The `ui` function in `src/ui.rs` is long (~200 lines) and
     handles all rendering logic.
   - **Suggestion:** Break it down into smaller, dedicated functions (e.g.,
     `render_top_bar`, `render_content_area`, `render_log_panel`,
     `render_input_line`, `render_changeset_dropdown`).
   - **Benefit:** Improved readability, maintainability, and testability.

2. **Reintroduce Change Set Details View:**
   - **Problem:** The main content area currently shows static keybindings, not
     the selected change set's details or merge status as described in older
     spec versions.
   - **Suggestion:** Display `app.selected_change_set_details` and
     `app.selected_change_set_merge_status` when a change set is selected.
     Options include replacing keybindings, adding a toggle ('?'), or showing
     keybindings only when no details are available.
   - **Benefit:** Restores core application functionality and usefulness.

3. **Simplify Log Panel Rendering:**
   - **Problem:** The log block and paragraph are rendered multiple times,
     partly to update the title with `current_action`.
   - **Suggestion:** Determine the final log title string first, render the
     `Block` once, get the inner area, and render the `Paragraph` once into that
     area.
   - **Benefit:** Minor code simplification and potentially slight performance
     improvement.

4. **Reduce Minor Code Duplication (Highlight Style):**
   - **Problem:** The focus highlight style
     (`Style::default().bg(Color::Blue).fg(Color::White)`) is defined separately
     for both Workspace and Change Set triggers.
   - **Suggestion:** Create a small private helper function
     (`get_trigger_style(is_focused: bool) -> Style`) within `ui.rs` to return
     the appropriate style.
   - **Benefit:** Reduces redundancy, improves consistency.

5. **Consider Configuration for UI Elements:**
   - **Problem:** UI layout values (e.g., dropdown width `50`, max height `10`)
     are hardcoded in `src/ui.rs`.
   - **Suggestion:** Move these values to constants at the top of the file or
     into a dedicated configuration struct if more complex configuration is
     anticipated.
   - **Benefit:** Easier modification and tuning of UI layout.

**Recommendation:** Implementing suggestion #1 (Refactoring) first would likely
make implementing #2 (Details View) easier.
