// src/ui.rs

// Intention: Define the main function responsible for rendering the TUI layout and widgets.
// Design Choice: This file now acts as the entry point for the UI module.
// It declares submodules for specific rendering tasks and calls them from the main `ui` function.

// Declare submodules for rendering components
mod get_trigger_style; // Although not directly called by `ui`, it's part of the module
mod render_changeset_dropdown;
mod render_content_area;
mod render_input_line;
mod render_log_panel;
mod render_top_bar;

use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction,
        Layout,
    },
    prelude::*, // Import common traits and types
};
// Import helper functions from submodules
use render_changeset_dropdown::render_changeset_dropdown;
use render_content_area::render_content_area;
use render_input_line::render_input_line;
use render_log_panel::render_log_panel;
use render_top_bar::render_top_bar;

use crate::app::{
    App,
    InputMode,
}; // Use App, Enums from local app module

// --- Constants for UI Layout ---
const LOG_PANEL_HEIGHT: u16 = 10;
// Dropdown constants are now defined within render_changeset_dropdown.rs

// Intention: Main UI rendering function. Sets up the layout and calls helper functions for each section.
// Design Choice: Split rendering logic into focused helper functions for clarity and maintainability.
pub fn ui(f: &mut Frame, app: &App) {
    // Define main layout: Top Bar, Main Content, Logs, optional Input Line.
    // Use constant for log height
    let (log_constraint, input_constraint) =
        if app.input_mode == InputMode::ChangeSetName {
            (Constraint::Length(LOG_PANEL_HEIGHT), Constraint::Length(1))
        } else {
            (Constraint::Length(LOG_PANEL_HEIGHT), Constraint::Length(0))
        };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top bar for dropdown triggers & email
            Constraint::Min(0),    // Main content area (for details)
            log_constraint,        // Log window area
            input_constraint,      // Input line area (conditional)
        ])
        .split(f.size());

    let top_bar_area = main_chunks[0];
    let content_area = main_chunks[1]; // Area for details view
    let log_area = main_chunks[2];
    let input_area = if main_chunks.len() > 3 {
        Some(main_chunks[3])
    } else {
        None // Input area is optional
    };

    // Call helper functions to render each part of the UI
    // Note: render_top_bar returns the area needed for the dropdown
    let cs_trigger_area = render_top_bar(f, app, top_bar_area);
    render_content_area(f, app, content_area);
    render_log_panel(f, app, log_area);
    if let Some(input_area_rect) = input_area {
        render_input_line(f, app, input_area_rect);
    }
    // Pass the trigger area to the dropdown renderer
    render_changeset_dropdown(f, app, cs_trigger_area);
}

// Helper functions and tests previously here have been moved to their respective modules
// or should be moved to dedicated test files.
