// Intention: Define application state and related enums/methods for the TUI.
// Design Choice: Central struct `App` holds all state, moved from main.rs.
// Enums `InputMode` and `DropdownFocus` define specific UI states.
// Methods previously in `impl App` are kept here.

use std::cmp::min;
use std::collections::HashMap; // Added for potential future use with schemas

use ratatui::widgets::ListState;
use situation::api_models::SchemaSummary;
use situation::api_models::{
    ChangeSet,
    ChangeSetSummary,
    ComponentViewV1, // Added import for component details
    MergeStatusV1Response,
    // SchemaSummary, // Removed from group
    WhoamiResponse,
}; // Ensure correct import name: MergeStatusV1Response // Import separately

// Intention: Define different input modes for the application.
// Design Choice: Enum to represent distinct input states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    ChangeSetName,
}

// Intention: Define the possible areas of the UI that can have focus.
// Design Choice: Enum provides a clear and type-safe way to manage focus state across different panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Added Copy
pub enum AppFocus {
    TopBar, // For switching between Workspace/ChangeSet triggers
    SchemaList,
    ContentArea, // Placeholder for future content interaction
    LogPanel,
    ChangeSetDropdown, // Focus specifically when the dropdown is active
    Input,             // Focus when in input mode
}

// Intention: Define which top-level element has focus *within the TopBar*.
// Design Choice: Enum to represent focus state for Workspace/ChangeSet triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Added Copy
pub enum DropdownFocus {
    Workspace,
    ChangeSet,
}

// Intention: Hold the application's state, including TUI interaction state,
// selected item details, merge status, UI flags, and dropdown state.
// Design Choice: Added fields for dropdown focus and activity. Removed show_details_pane for now,
// details will show in the main area based on selection.
#[derive(Debug, Clone)]
pub struct App {
    pub whoami_data: Option<WhoamiResponse>,
    pub change_sets: Option<Vec<ChangeSetSummary>>, // Use imported ChangeSetSummary
    pub change_set_list_state: ListState, // State for the change set list selection (now in dropdown)
    pub selected_change_set_details: Option<ChangeSet>, // Details of the selected change set
    pub selected_change_set_merge_status: Option<MergeStatusV1Response>, // Merge status of the selected change set
    pub selected_change_set_components: Option<Vec<ComponentViewV1>>, // Components in the selected change set, parsed from JSON string
    pub current_action: Option<String>, // Feedback for ongoing actions
    pub input_mode: InputMode,          // Current input mode
    pub input_buffer: String,           // Buffer for text input
    pub logs: Vec<String>,
    pub log_scroll: usize,
    pub dropdown_focus: DropdownFocus, // Which dropdown trigger is focused (within TopBar)
    pub changeset_dropdown_active: bool, // Is the changeset dropdown list visible?

    // Schema List State
    // Intention: Store detailed schema information for display and interaction.
    // Design Choice: Use the SchemaSummary struct from api_models to hold category, installed status, etc.
    pub schemas: Vec<SchemaSummary>, // Changed from Vec<String>
    pub schema_list_state: ListState, // State for the schema list selection

    // Overall Focus
    pub current_focus: AppFocus, // Tracks which major UI pane has focus
}

impl App {
    // Intention: Create a new App instance with default values.
    // Design Choice: Initialize new schema and focus fields.
    pub fn new() -> Self {
        Self {
            whoami_data: None,
            change_sets: None,
            change_set_list_state: ListState::default(),
            selected_change_set_details: None,
            selected_change_set_merge_status: None,
            selected_change_set_components: None, // Initialize the new field
            current_action: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            logs: Vec::new(),
            log_scroll: 0,
            dropdown_focus: DropdownFocus::Workspace, // Start focus on workspace trigger in top bar
            changeset_dropdown_active: false,         // Dropdown starts closed

            // Initialize schema list
            schemas: Vec::new(),
            schema_list_state: ListState::default(),

            // Initialize focus
            current_focus: AppFocus::TopBar, // Start focus on the top bar
        }
    }

    // Intention: Add a log message and automatically scroll to the bottom if needed.
    // Design Choice: Calculates the maximum scroll position based on log count and view height,
    // then sets the current scroll position to the maximum, ensuring the latest log is visible.
    // This method is intended to be used whenever a new log entry is generated by the application.
    // The `view_height` parameter should match the height constraint used for the log Paragraph in the UI.
    pub fn add_log_auto_scroll(&mut self, message: String, view_height: usize) {
        self.logs.push(message);
        // Calculate max scroll based on the *new* number of logs and window height
        let max_scroll = self.logs.len().saturating_sub(view_height);
        self.log_scroll = max_scroll; // Always scroll to the bottom
    }

    // Intention: Scroll the log view up by one line.
    pub fn scroll_logs_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }

    // Intention: Scroll the log view down by one line.
    // Design Choice: Prevent scrolling beyond the available log lines.
    pub fn scroll_logs_down(&mut self, view_height: usize) {
        // Calculate max scroll based on number of logs and window height
        let max_scroll = self.logs.len().saturating_sub(view_height);
        self.log_scroll = min(self.log_scroll.saturating_add(1), max_scroll);
    }

    // Intention: Move selection down in the change set list (dropdown).
    pub fn change_set_next(&mut self) {
        if let Some(change_sets) = &self.change_sets {
            if change_sets.is_empty() {
                return;
            } // Do nothing if empty
            let i = match self.change_set_list_state.selected() {
                Some(i) => {
                    if i >= change_sets.len() - 1 {
                        0 // Wrap around
                    } else {
                        i + 1
                    }
                }
                None => 0, // Select first if nothing selected
            };
            self.change_set_list_state.select(Some(i));
            // When selection changes, clear old details
            self.selected_change_set_details = None;
            self.selected_change_set_merge_status = None;
            self.selected_change_set_components = None; // Clear components too
        }
    }

    // Intention: Move selection up in the change set list (dropdown).
    pub fn change_set_previous(&mut self) {
        if let Some(change_sets) = &self.change_sets {
            if change_sets.is_empty() {
                return;
            } // Do nothing if empty
            let i = match self.change_set_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        change_sets.len() - 1 // Wrap around
                    } else {
                        i - 1
                    }
                }
                None => change_sets.len() - 1, // Select last if nothing selected
            };
            self.change_set_list_state.select(Some(i));
            // When selection changes, clear old details
            self.selected_change_set_details = None;
            self.selected_change_set_merge_status = None;
            self.selected_change_set_components = None; // Clear components too
        }
    }

    // Intention: Select a change set in the list state by its ID.
    // Design Choice: Iterates through the change_sets vector, finds the index matching the ID,
    // and updates the list state. If the ID is not found or the list is empty/None,
    // the selection remains unchanged. Also clears details/components.
    pub fn select_change_set_by_id(&mut self, change_set_id: &str) {
        if let Some(change_sets) = &self.change_sets {
            if let Some(index) =
                change_sets.iter().position(|cs| cs.id == change_set_id)
            {
                self.change_set_list_state.select(Some(index));
                // Clear details when selection changes programmatically too
                self.selected_change_set_details = None;
                self.selected_change_set_merge_status = None;
                self.selected_change_set_components = None; // Clear components too
            }
            // If ID not found, do nothing, keep current selection
        }
        // If change_sets is None, do nothing
    }

    // Intention: Get the summary of the currently selected change set.
    // Design Choice: Helper method to avoid repetitive code.
    pub fn get_selected_changeset_summary(&self) -> Option<&ChangeSetSummary> {
        self.change_set_list_state.selected().and_then(|idx| {
            self.change_sets.as_ref().and_then(|css| css.get(idx))
        })
    }

    // Intention: Move selection down in the schema list.
    // Design Choice: Handles wrapping and empty list case.
    // When a schema is selected, the content area will filter components to show only those
    // that match the selected schema's ID.
    pub fn schema_next(&mut self) {
        if self.schemas.is_empty() {
            return;
        }
        let i = match self.schema_list_state.selected() {
            Some(i) => {
                if i >= self.schemas.len() - 1 {
                    0 // Wrap around
                } else {
                    i + 1
                }
            }
            None => 0, // Select first if nothing selected
        };
        self.schema_list_state.select(Some(i));
        // Note: Component filtering based on selected schema is handled in render_content_area.rs

        // Debug: Log the selected schema
        if let Some(selected_idx) = self.schema_list_state.selected() {
            if !self.schemas.is_empty() {
                let selected_schema = &self.schemas[selected_idx];
                self.add_log_auto_scroll(
                    format!(
                        "DEBUG: Selected schema: {} (id: {})",
                        selected_schema.schema_name, selected_schema.schema_id
                    ),
                    10, // LOG_HEIGHT
                );
            }
        }
    }

    // Intention: Move selection up in the schema list.
    // Design Choice: Handles wrapping and empty list case.
    // When a schema is selected, the content area will filter components to show only those
    // that match the selected schema's ID.
    pub fn schema_previous(&mut self) {
        if self.schemas.is_empty() {
            return;
        }
        let i = match self.schema_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.schemas.len() - 1 // Wrap around
                } else {
                    i - 1
                }
            }
            None => self.schemas.len() - 1, // Select last if nothing selected
        };
        self.schema_list_state.select(Some(i));
        // Note: Component filtering based on selected schema is handled in render_content_area.rs

        // Debug: Log the selected schema
        if let Some(selected_idx) = self.schema_list_state.selected() {
            if !self.schemas.is_empty() {
                let selected_schema = &self.schemas[selected_idx];
                self.add_log_auto_scroll(
                    format!(
                        "DEBUG: Selected schema: {} (id: {})",
                        selected_schema.schema_name, selected_schema.schema_id
                    ),
                    10, // LOG_HEIGHT
                );
            }
        }
    }
}
