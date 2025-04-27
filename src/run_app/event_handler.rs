// src/run_app/event_handler.rs

// Intention: Handle key events based on the application's current mode and state.
// Design Choice: Extracted from the main loop in run_app.rs for modularity.
// Takes the key event, mutable app state, terminal, and log height.
// Returns Ok(true) if the app should quit, Ok(false) otherwise, or an io::Error.

use std::io;

use crossterm::event::{
    Event, // Keep Event import if needed for future expansion, though only KeyCode used now
    KeyCode,
    KeyEvent,
};
use ratatui::{
    Terminal,
    backend::Backend,
};
use situation::{
    // Use the library crate namespace
    api_client,
    api_models::CreateChangeSetV1Request,
};

use crate::{
    // Use local crate namespace for app modules
    app::{
        App,
        AppFocus, // Import AppFocus
        DropdownFocus,
        InputMode,
    },
    refresh_change_sets::refresh_change_sets,
    ui::ui, // Need ui to redraw during actions
};

// Define LOG_HEIGHT here or pass it as an argument if it might change
const LOG_HEIGHT: usize = 10;

// Intention: Process a single key event and update the app state accordingly.
// Design Choice: Contains the large match statement previously in the main loop. Async because it calls API functions.
// Refactored to handle AppFocus correctly.
pub async fn handle_key_event<B: Backend>(
    key: KeyEvent,
    app: &mut App,
    terminal: &mut Terminal<B>,
) -> io::Result<bool> {
    // Returns true if app should quit
    let workspace_id = app.whoami_data.as_ref().map(|d| d.workspace_id.clone());
    // Get selected CS ID *before* potential state changes within the match arms
    let selected_cs_id = app.get_selected_changeset_summary().map(|cs| cs.id.clone());

    // --- Global Quit ---
    // Allow 'q' to quit regardless of mode or focus, unless in input mode
    if app.input_mode != InputMode::ChangeSetName && key.code == KeyCode::Char('q')
    {
        return Ok(true); // Signal to quit
    }

    match app.input_mode {
        InputMode::Normal => {
            // --- Focus Handling (Tab Key) ---
            // Handle focus cycling first if Tab is pressed and dropdown is NOT active
            if !app.changeset_dropdown_active && key.code == KeyCode::Tab {
                app.current_focus = match app.current_focus {
                    AppFocus::TopBar => AppFocus::SchemaList,
                    AppFocus::SchemaList => AppFocus::LogPanel, // Skip ContentArea for now
                    AppFocus::ContentArea => AppFocus::LogPanel, // Should not happen currently
                    AppFocus::LogPanel => AppFocus::TopBar,
                    // These should not be reachable in Normal mode + Tab press, but handle defensively
                    AppFocus::ChangeSetDropdown => AppFocus::TopBar,
                    AppFocus::Input => AppFocus::TopBar,
                };
                return Ok(false); // Focus changed, no further action needed for Tab
            }

            // --- Handle based on Current Focus ---
            match app.current_focus {
                // --- Focus: Top Bar ---
                AppFocus::TopBar => {
                    // Handle dropdown state first if active
                    if app.changeset_dropdown_active {
                        match key.code {
                            KeyCode::Up => app.change_set_previous(),
                            KeyCode::Down => app.change_set_next(),
                            KeyCode::Enter => {
                                // Select item, close dropdown, keep focus TopBar
                                app.changeset_dropdown_active = false;
                                // app.current_focus = AppFocus::TopBar; // Focus already here
                                app.current_action = None;

                                // Fetch details and schemas for the newly selected item
                                if let (Some(ws_id), Some(cs_id)) = (
                                    workspace_id.clone(),
                                    app.get_selected_changeset_summary() // Get ID *after* selection might have changed
                                        .map(|cs| cs.id.clone()),
                                ) {
                                    app.current_action = Some(
                                        "Fetching details & schemas...".to_string(),
                                    );
                                    terminal.draw(|f| ui(f, app))?; // Redraw immediately
                                    fetch_details_and_status(app, &ws_id, &cs_id).await;
                                    fetch_schemas(app, &ws_id, &cs_id).await;
                                    app.current_action = None;
                                } else {
                                    // Clear details if no selection or error occurred during fetch
                                    app.selected_change_set_details = None;
                                    app.selected_change_set_merge_status = None;
                                    app.schemas.clear();
                                    app.schema_list_state.select(None);
                                }
                            }
                            KeyCode::Esc => {
                                // Close dropdown without changing selection, focus stays TopBar
                                app.changeset_dropdown_active = false;
                                // app.current_focus = AppFocus::TopBar; // Focus already here
                                app.current_action = None;
                            }
                            KeyCode::Tab => {
                                // Tab cycles focus even when dropdown is open
                                app.changeset_dropdown_active = false; // Close dropdown first
                                app.current_focus = AppFocus::SchemaList; // Move focus
                                app.current_action = None;
                            }
                            _ => {} // Other keys ignored when dropdown is active
                        }
                    } else {
                        // Normal mode, TopBar focus, dropdown closed
                        match key.code {
                            // KeyCode::Char('q') handled globally
                            // KeyCode::Tab handled globally above
                            KeyCode::Char('k') => app.scroll_logs_up(), // Keep global log scroll
                            KeyCode::Char('j') => app.scroll_logs_down(LOG_HEIGHT), // Keep global log scroll
                            KeyCode::Left | KeyCode::Right => {
                                // Switch focus between triggers within TopBar
                                app.dropdown_focus = match app.dropdown_focus {
                                    DropdownFocus::Workspace => DropdownFocus::ChangeSet,
                                    DropdownFocus::ChangeSet => DropdownFocus::Workspace,
                                };
                            }
                            KeyCode::Char(' ') | KeyCode::Enter => {
                                // Activate focused element within TopBar
                                match app.dropdown_focus {
                                    DropdownFocus::Workspace => {
                                        app.add_log_auto_scroll(
                                            "Workspace selection not implemented.".to_string(),
                                            LOG_HEIGHT,
                                        );
                                    }
                                    DropdownFocus::ChangeSet => {
                                        // Open the dropdown if change sets exist
                                        if app.change_sets.as_ref().map_or(false, |cs| !cs.is_empty()) {
                                            app.changeset_dropdown_active = true;
                                            // Keep focus AppFocus::TopBar while dropdown is open
                                            // Ensure selection is valid if opening
                                            if app.change_set_list_state.selected().is_none() {
                                                app.change_set_list_state.select(Some(0));
                                            }
                                        } else {
                                            app.add_log_auto_scroll(
                                                "No change sets to select.".to_string(),
                                                LOG_HEIGHT,
                                            );
                                        }
                                    }
                                }
                            }
                            // --- Change Set Actions (operate on selection from state) ---
                            KeyCode::Char('d') => { // Delete
                                if let (Some(ws_id), Some(cs_id)) = (workspace_id.clone(), selected_cs_id.clone()) {
                                    app.current_action = Some(format!("Deleting {}...", cs_id));
                                    terminal.draw(|f| ui(f, app))?;
                                    match api_client::abandon_change_set(&ws_id, &cs_id).await {
                                        Ok((resp, logs)) => {
                                            logs.into_iter().for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));
                                            app.add_log_auto_scroll(format!("Abandoned changeset {} (Success: {})", cs_id, resp.success), LOG_HEIGHT);
                                            // Clear state related to the deleted item
                                            app.selected_change_set_details = None;
                                            app.selected_change_set_merge_status = None;
                                            app.schemas.clear();
                                            app.schema_list_state.select(None);
                                        }
                                        Err(e) => app.add_log_auto_scroll(format!("Error abandoning changeset {}: {}", cs_id, e), LOG_HEIGHT),
                                    }
                                    app.current_action = None;
                                    refresh_change_sets(app).await; // Refresh list
                                    // Fetch schemas for potentially new selection after refresh
                                    if let Some(new_selected_cs) = app.get_selected_changeset_summary() {
                                        if let Some(ws_id_inner) = workspace_id.as_ref() { // Need ws_id again
                                            fetch_schemas(app, ws_id_inner, &new_selected_cs.id).await;
                                        }
                                    } else {
                                        // Ensure schemas are cleared if no CS selected after refresh
                                        app.schemas.clear();
                                        app.schema_list_state.select(None);
                                    }
                                    // Ensure details are cleared after refresh too
                                    app.selected_change_set_details = None;
                                    app.selected_change_set_merge_status = None;
                                } else {
                                    app.add_log_auto_scroll("Cannot delete: No change set selected.".to_string(), LOG_HEIGHT);
                                }
                            }
                            KeyCode::Char('c') => { // Create
                                if workspace_id.is_some() {
                                    app.input_mode = InputMode::ChangeSetName;
                                    app.current_focus = AppFocus::Input; // Set focus to input
                                    app.input_buffer.clear();
                                    app.current_action = None;
                                } else {
                                    app.add_log_auto_scroll("Cannot create: No workspace available.".to_string(), LOG_HEIGHT);
                                }
                            }
                            KeyCode::Char('f') => { // Force Apply
                                if let (Some(ws_id), Some(cs_id)) = (workspace_id.clone(), selected_cs_id.clone()) {
                                    app.current_action = Some(format!("Applying {}...", cs_id));
                                    terminal.draw(|f| ui(f, app))?;
                                    match api_client::force_apply(&ws_id, &cs_id).await {
                                        Ok((_, logs)) => {
                                            logs.into_iter().for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));
                                            app.add_log_auto_scroll(format!("Apply initiated for changeset {}", cs_id), LOG_HEIGHT);
                                            // Clear details as status might change
                                            app.selected_change_set_details = None;
                                            app.selected_change_set_merge_status = None;
                                        }
                                        Err(e) => app.add_log_auto_scroll(format!("Error applying changeset {}: {}", cs_id, e), LOG_HEIGHT),
                                    }
                                    app.current_action = None;
                                    refresh_change_sets(app).await; // Refresh list
                                    // Fetch schemas for potentially new selection after refresh
                                    if let Some(new_selected_cs) = app.get_selected_changeset_summary() {
                                        if let Some(ws_id_inner) = workspace_id.as_ref() { // Need ws_id again
                                            fetch_schemas(app, ws_id_inner, &new_selected_cs.id).await;
                                        }
                                    } else {
                                         // Ensure schemas are cleared if no CS selected after refresh
                                        app.schemas.clear();
                                        app.schema_list_state.select(None);
                                    }
                                     // Ensure details are cleared after refresh too
                                    app.selected_change_set_details = None;
                                    app.selected_change_set_merge_status = None;
                                } else {
                                    app.add_log_auto_scroll("Cannot apply: No change set selected.".to_string(), LOG_HEIGHT);
                                }
                            }
                            _ => {} // Ignore other keys
                        }
                    }
                } // End AppFocus::TopBar

                // --- Focus: Schema List ---
                AppFocus::SchemaList => {
                    match key.code {
                        KeyCode::Up => app.schema_previous(),
                        KeyCode::Down => app.schema_next(),
                        // KeyCode::Tab handled globally above
                        KeyCode::Char('k') => app.scroll_logs_up(), // Keep global log scroll
                        KeyCode::Char('j') => app.scroll_logs_down(LOG_HEIGHT), // Keep global log scroll
                        _ => {} // Ignore other keys when schema list is focused
                    }
                } // End AppFocus::SchemaList

                // --- Focus: Content Area (Placeholder) ---
                AppFocus::ContentArea => {
                     match key.code {
                        // KeyCode::Tab handled globally above
                        KeyCode::Char('k') => app.scroll_logs_up(), // Keep global log scroll
                        KeyCode::Char('j') => app.scroll_logs_down(LOG_HEIGHT), // Keep global log scroll
                        _ => {} // Ignore other keys for now
                    }
                } // End AppFocus::ContentArea

                // --- Focus: Log Panel ---
                AppFocus::LogPanel => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => app.scroll_logs_up(), // Allow Up arrow too
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_logs_down(LOG_HEIGHT), // Allow Down arrow too
                        // KeyCode::Tab handled globally above
                        _ => {} // Ignore other keys when log panel is focused
                    }
                } // End AppFocus::LogPanel

                // --- Focus: ChangeSet Dropdown / Input (Should not be reachable in Normal Mode) ---
                // These states should only be active when the respective UI element is active.
                // If focus somehow lands here incorrectly, redirect it.
                AppFocus::ChangeSetDropdown | AppFocus::Input => {
                    app.current_focus = AppFocus::TopBar; // Redirect focus
                }
            } // End match app.current_focus
        } // End InputMode::Normal

        InputMode::ChangeSetName => {
            // Ensure focus is set correctly when entering this mode
            app.current_focus = AppFocus::Input;
            match key.code {
                KeyCode::Enter => {
                    if let Some(ws_id) = workspace_id.clone() {
                        let new_cs_name = app.input_buffer.trim().to_string();
                        if !new_cs_name.is_empty() {
                            app.current_action = Some(format!("Creating '{}'...", new_cs_name));
                            terminal.draw(|f| ui(f, app))?; // Redraw to show action
                            let request = CreateChangeSetV1Request { change_set_name: new_cs_name.clone() };
                            match api_client::create_change_set(&ws_id, request).await {
                                Ok((created_cs_response, logs)) => {
                                    let new_change_set_id = created_cs_response.change_set.id.clone();
                                    logs.into_iter().for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));
                                    app.add_log_auto_scroll(format!("Created changeset '{}' ({})", created_cs_response.change_set.name, &new_change_set_id), LOG_HEIGHT);
                                    refresh_change_sets(app).await; // Refresh list
                                    app.select_change_set_by_id(&new_change_set_id); // Select the new one
                                    fetch_schemas(app, &ws_id, &new_change_set_id).await; // Fetch schemas for new CS
                                }
                                Err(e) => {
                                    app.add_log_auto_scroll(format!("Error creating changeset: {}", e), LOG_HEIGHT);
                                    refresh_change_sets(app).await; // Refresh even on error
                                    // Clear schemas if creation failed but list refreshed
                                    app.schemas.clear();
                                    app.schema_list_state.select(None);
                                }
                            }
                        } else {
                            app.add_log_auto_scroll("Change set name cannot be empty.".to_string(), LOG_HEIGHT);
                        }
                    } else {
                        app.add_log_auto_scroll("Cannot create: Workspace ID missing.".to_string(), LOG_HEIGHT);
                    }
                    // Reset state after submission or error
                    app.input_mode = InputMode::Normal;
                    app.current_focus = AppFocus::TopBar; // Return focus to TopBar
                    app.input_buffer.clear();
                    app.current_action = None;
                }
                KeyCode::Char(c) => app.input_buffer.push(c),
                KeyCode::Backspace => { app.input_buffer.pop(); }
                KeyCode::Esc => {
                    // Cancel input mode
                    app.input_mode = InputMode::Normal;
                    app.current_focus = AppFocus::TopBar; // Return focus to TopBar
                    app.input_buffer.clear();
                    app.current_action = None;
                    app.add_log_auto_scroll("Change set creation cancelled.".to_string(), LOG_HEIGHT);
                }
                _ => {} // Ignore other keys in input mode
            }
        } // End InputMode::ChangeSetName
    } // End match app.input_mode

    Ok(false) // Signal to continue the loop
}

// --- Helper Async Functions --- (Defined outside handle_key_event)

// Intention: Fetch change set details and merge status.
// Design Choice: Encapsulate the dual fetch logic. Updates app state.
async fn fetch_details_and_status(app: &mut App, ws_id: &str, cs_id: &str) {
    // Fetch details
    match api_client::get_change_set(ws_id, cs_id).await {
        Ok((get_response, logs)) => {
            app.selected_change_set_details = Some(get_response.change_set);
            logs.into_iter().for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));
            app.add_log_auto_scroll(format!("Details fetched for {}", cs_id), LOG_HEIGHT);
        }
        Err(e) => {
            app.selected_change_set_details = None; // Clear on error
            app.add_log_auto_scroll(format!("Error fetching details for {}: {}", cs_id, e), LOG_HEIGHT);
        }
    }
    // Fetch merge status
    match api_client::get_merge_status(ws_id, cs_id).await {
        Ok((status_response, logs)) => {
            app.selected_change_set_merge_status = Some(status_response);
            logs.into_iter().for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));
            app.add_log_auto_scroll(format!("Merge status fetched for {}", cs_id), LOG_HEIGHT);
        }
        Err(e) => {
            app.selected_change_set_merge_status = None; // Clear on error
            app.add_log_auto_scroll(format!("Error fetching merge status for {}: {}", cs_id, e), LOG_HEIGHT);
        }
    }
}

// Intention: Fetch the list of schemas for the given workspace and change set.
// Design Choice: Encapsulate schema fetching logic. Updates app state.
async fn fetch_schemas(app: &mut App, ws_id: &str, cs_id: &str) {
    app.add_log_auto_scroll(format!("Fetching schemas for change set {}...", cs_id), LOG_HEIGHT);
    match api_client::list_schemas(ws_id, cs_id).await {
        Ok(schema_response) => {
            app.schemas = schema_response.schemas.into_iter().map(|s| s.schema_name).collect();
            app.schemas.sort_unstable(); // Sort alphabetically
            // Select first item if list is not empty, otherwise clear selection
            if !app.schemas.is_empty() {
                app.schema_list_state.select(Some(0));
            } else {
                app.schema_list_state.select(None);
            }
            app.add_log_auto_scroll("Successfully fetched schemas.".to_string(), LOG_HEIGHT);
        }
        Err(e) => {
            app.schemas.clear(); // Clear schemas on error
            app.schema_list_state.select(None); // Clear selection on error
            app.add_log_auto_scroll(format!("Error fetching schemas: {}", e), LOG_HEIGHT);
        }
    }
}
