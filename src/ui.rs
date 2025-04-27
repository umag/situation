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
mod render_schema_list; // Declare the new module
mod render_top_bar;

use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction,
        Layout,
    },
    prelude::*,     // Import common traits and types
    widgets::Block, // Import Block for layout structure
};
// Import helper functions from submodules
use render_changeset_dropdown::render_changeset_dropdown;
use render_content_area::render_content_area;
use render_input_line::render_input_line;
use render_log_panel::render_log_panel;
use render_schema_list::render_schema_list; // Import the new function
use render_top_bar::render_top_bar;

use crate::app::{
    App,
    InputMode,
}; // Use App, Enums from local app module

// --- Constants for UI Layout ---
const LOG_PANEL_HEIGHT: u16 = 10;
const SCHEMA_LIST_WIDTH: u16 = 30; // Width for the new schema list pane

// Intention: Main UI rendering function. Sets up the layout and calls helper functions for each section.
// Design Choice: Split rendering logic into focused helper functions. Added horizontal split for schema list.
// Changed `app` parameter to `&mut App` to allow state modification by stateful widgets.
pub fn ui(f: &mut Frame, app: &mut App) {
    // Changed to &mut App
    // Define main vertical layout: Top Bar, Middle Area, Logs, optional Input Line.
    let (log_constraint, input_constraint) =
        if app.input_mode == InputMode::ChangeSetName {
            (Constraint::Length(LOG_PANEL_HEIGHT), Constraint::Length(1)) // Log height, Input line height
        } else {
            (Constraint::Length(LOG_PANEL_HEIGHT), Constraint::Length(0)) // Log height, No input line
        };

    // Vertical layout for the whole screen
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top bar
            Constraint::Min(0),    // Middle area (will be split horizontally)
            log_constraint,        // Log panel
            input_constraint,      // Input line (conditional)
        ])
        .split(f.size());

    let top_bar_area = vertical_chunks[0];
    let middle_area = vertical_chunks[1]; // This area will contain schema list + content
    let log_area = vertical_chunks[2];
    let input_area = if vertical_chunks.len() > 3 {
        Some(vertical_chunks[3])
    } else {
        None
    };

    // Horizontal layout for the middle area
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(SCHEMA_LIST_WIDTH), // Left pane for schema list
            Constraint::Min(0), // Right pane for main content
        ])
        .split(middle_area); // Split the middle_area defined above

    let schema_list_area = horizontal_chunks[0];
    let content_area = horizontal_chunks[1]; // This is the new content area

    // --- Render UI Components ---

    // Render Top Bar (returns area for dropdown)
    let cs_trigger_area = render_top_bar(f, app, top_bar_area);

    // Render Schema List
    render_schema_list(f, app, schema_list_area); // Call the new function

    // Render Main Content Area (now on the right)
    render_content_area(f, &*app, content_area);

    // Render Log Panel
    render_log_panel(f, app, log_area);

    // Render Input Line (conditional)
    if let Some(input_area_rect) = input_area {
        render_input_line(f, app, input_area_rect);
    }

    // Render Change Set Dropdown (overlay)
    render_changeset_dropdown(f, app, cs_trigger_area); // Pass mutable app
}

// Helper functions and tests previously here have been moved to their respective modules
// or should be moved to dedicated test files.
