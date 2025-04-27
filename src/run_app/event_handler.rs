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
    KeyModifiers, // Import KeyModifiers for Alt key check
};
use ratatui::{
    Terminal,
    backend::Backend,
};
use situation::{
    // Use the library crate namespace
    api_client,
    api_models::{
        ComponentViewV1,
        CreateChangeSetV1Request,
    },
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
    let selected_cs_id =
        app.get_selected_changeset_summary().map(|cs| cs.id.clone());

    // --- Global Quit ---
    // Allow 'q' to quit regardless of mode or focus, unless in input mode
    if app.input_mode != InputMode::ChangeSetName
        && key.code == KeyCode::Char('q')
    {
        return Ok(true); // Signal to quit
    }

    match app.input_mode {
        InputMode::Normal => {
            // --- Focus Hotkeys (Alt + Key) ---
            if key.modifiers == KeyModifiers::ALT {
                // Check for ALT modifier
                match key.code {
                    KeyCode::Char('w') => {
                        // Alt+W for Workspace/Changeset focus
                        app.current_focus = AppFocus::TopBar;
                        // Optionally set default dropdown focus if needed
                        app.dropdown_focus = DropdownFocus::Workspace; // Set focus within top bar
                        return Ok(false); // Consumed event
                    }
                    KeyCode::Char('c') => {
                        // Alt+C for Change Set focus
                        app.current_focus = AppFocus::TopBar;
                        app.dropdown_focus = DropdownFocus::ChangeSet; // Set focus within top bar
                        return Ok(false); // Consumed event
                    }
                    KeyCode::Char('s') => {
                        // Alt+S for Schema List focus
                        app.current_focus = AppFocus::SchemaList;
                        return Ok(false); // Consumed event
                    }
                    KeyCode::Char('l') => {
                        // Alt+L for Log Panel focus
                        app.current_focus = AppFocus::LogPanel;
                        return Ok(false); // Consumed event
                    }
                    _ => {} // Ignore other Alt combinations
                }
            } // End Alt key check

            // --- Focus Handling (Tab Key) ---
            // Handle focus cycling first if Tab is pressed and dropdown is NOT active
            if !app.changeset_dropdown_active && key.code == KeyCode::Tab {
                app.current_focus = match app.current_focus {
                    AppFocus::TopBar => AppFocus::SchemaList,
                    AppFocus::SchemaList => AppFocus::ContentArea, // Cycle to Content Area
                    AppFocus::ContentArea => AppFocus::LogPanel, // Cycle to Log Panel
                    AppFocus::LogPanel => AppFocus::TopBar, // Cycle back to Top Bar
                    // These should not be reachable in Normal mode + Tab press, but handle defensively
                    AppFocus::ChangeSetDropdown => AppFocus::TopBar, // If somehow here, go to TopBar
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
                        // This block should ideally not be reached if focus is managed correctly,
                        // but as a fallback, maybe close the dropdown and reset focus?
                        // app.changeset_dropdown_active = false;
                        // app.current_focus = AppFocus::TopBar; // Ensure focus returns to TopBar
                        // app.add_log_auto_scroll("DEBUG: Unexpected state: Dropdown active but focus is TopBar. Resetting.".to_string(), LOG_HEIGHT);
                        // For now, just let the ChangeSetDropdown focus handle it.
                        match key.code {
                            // Enter logic is now MOVED to AppFocus::ChangeSetDropdown
                            _ => {} // Other keys ignored when dropdown is active (in this temporary state)
                        }
                    } else {
                        // Normal mode, TopBar focus, dropdown closed
                        match key.code {
                            // KeyCode::Char('q') handled globally
                            // KeyCode::Tab handled globally above
                            KeyCode::Char('k') => app.scroll_logs_up(), // Keep global log scroll
                            KeyCode::Char('j') => {
                                app.scroll_logs_down(LOG_HEIGHT)
                            } // Keep global log scroll
                            KeyCode::Left | KeyCode::Right => {
                                // Switch focus between triggers within TopBar
                                app.dropdown_focus = match app.dropdown_focus {
                                    DropdownFocus::Workspace => {
                                        DropdownFocus::ChangeSet
                                    }
                                    DropdownFocus::ChangeSet => {
                                        DropdownFocus::Workspace
                                    }
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
                                        if app
                                            .change_sets
                                            .as_ref()
                                            .map_or(false, |cs| !cs.is_empty())
                                        {
                                            app.changeset_dropdown_active =
                                                true;
                                            // *** Set focus specifically to the dropdown ***
                                            app.current_focus =
                                                AppFocus::ChangeSetDropdown;
                                            // Ensure selection is valid if opening
                                            if app
                                                .change_set_list_state
                                                .selected()
                                                .is_none()
                                            {
                                                app.change_set_list_state
                                                    .select(Some(0));
                                            }
                                        } else {
                                            app.add_log_auto_scroll(
                                                "No change sets to select."
                                                    .to_string(),
                                                LOG_HEIGHT,
                                            );
                                        }
                                    }
                                }
                            }
                            // --- Change Set Actions (operate on selection from state) ---
                            KeyCode::Char('d') => {
                                // Delete
                                if let (Some(ws_id), Some(cs_id)) = (
                                    workspace_id.clone(),
                                    selected_cs_id.clone(),
                                ) {
                                    app.current_action =
                                        Some(format!("Deleting {}...", cs_id));
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
                                    let new_selected_cs_id = app
                                        .get_selected_changeset_summary()
                                        .map(|cs| cs.id.clone()); // Get ID and drop borrow
                                    if let Some(cs_id) = new_selected_cs_id {
                                        if let Some(ws_id_inner) =
                                            workspace_id.as_ref()
                                        {
                                            // Need ws_id again
                                            fetch_schemas(
                                                app,
                                                ws_id_inner,
                                                &cs_id,
                                            )
                                            .await; // Now only mutable borrow needed
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
                            KeyCode::Char('c') => {
                                // Create
                                if workspace_id.is_some() {
                                    app.input_mode = InputMode::ChangeSetName;
                                    app.current_focus = AppFocus::Input; // Set focus to input
                                    app.input_buffer.clear();
                                    app.current_action = None;
                                } else {
                                    app.add_log_auto_scroll("Cannot create: No workspace available.".to_string(), LOG_HEIGHT);
                                }
                            }
                            KeyCode::Char('f') => {
                                // Force Apply
                                if let (Some(ws_id), Some(cs_id)) = (
                                    workspace_id.clone(),
                                    selected_cs_id.clone(),
                                ) {
                                    app.current_action =
                                        Some(format!("Applying {}...", cs_id));
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
                                    let new_selected_cs_id = app
                                        .get_selected_changeset_summary()
                                        .map(|cs| cs.id.clone()); // Get ID and drop borrow
                                    if let Some(cs_id) = new_selected_cs_id {
                                        if let Some(ws_id_inner) =
                                            workspace_id.as_ref()
                                        {
                                            // Need ws_id again
                                            fetch_schemas(
                                                app,
                                                ws_id_inner,
                                                &cs_id,
                                            )
                                            .await; // Now only mutable borrow needed
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
                                    app.add_log_auto_scroll(
                                        "Cannot apply: No change set selected."
                                            .to_string(),
                                        LOG_HEIGHT,
                                    );
                                }
                            }
                            _ => {} // Ignore other keys
                        }
                    }
                } // End AppFocus::TopBar

                // --- Focus: Schema List ---
                AppFocus::SchemaList => {
                    match key.code {
                        KeyCode::Up => {
                            app.schema_previous();
                            // Fetch components for the selected change set
                            if let (Some(ws_id), Some(cs_id)) =
                                (workspace_id.clone(), selected_cs_id.clone())
                            {
                                app.add_log_auto_scroll(
                                    "DEBUG: Fetching components after schema selection (Up)".to_string(),
                                    LOG_HEIGHT,
                                );
                                fetch_components(app, &ws_id, &cs_id).await;
                            }
                        }
                        KeyCode::Down => {
                            app.schema_next();
                            // Fetch components for the selected change set
                            if let (Some(ws_id), Some(cs_id)) =
                                (workspace_id.clone(), selected_cs_id.clone())
                            {
                                app.add_log_auto_scroll(
                                    "DEBUG: Fetching components after schema selection (Down)".to_string(),
                                    LOG_HEIGHT,
                                );
                                fetch_components(app, &ws_id, &cs_id).await;
                            }
                        }
                        KeyCode::Enter => {
                            // When Enter is pressed on a schema, fetch components for the selected change set
                            if let (Some(ws_id), Some(cs_id)) =
                                (workspace_id.clone(), selected_cs_id.clone())
                            {
                                app.current_action =
                                    Some("Fetching components...".to_string());
                                terminal.draw(|f| ui(f, app))?; // Redraw immediately
                                fetch_components(app, &ws_id, &cs_id).await;
                                app.current_action = None;
                            } else {
                                app.add_log_auto_scroll(
                                    "Cannot fetch components: No change set selected.".to_string(),
                                    LOG_HEIGHT,
                                );
                            }
                        }
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
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.scroll_logs_up()
                        } // Allow Up arrow too
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.scroll_logs_down(LOG_HEIGHT)
                        } // Allow Down arrow too
                        // KeyCode::Tab handled globally above
                        _ => {} // Ignore other keys when log panel is focused
                    }
                } // End AppFocus::LogPanel

                // --- Focus: ChangeSet Dropdown (When dropdown list is active) ---
                AppFocus::ChangeSetDropdown => {
                    // Keys relevant only when the dropdown is active
                    match key.code {
                        KeyCode::Up => app.change_set_previous(),
                        KeyCode::Down => app.change_set_next(),
                        KeyCode::Enter => {
                            // Select item, close dropdown, keep focus TopBar
                            app.changeset_dropdown_active = false;
                            app.current_focus = AppFocus::TopBar; // Return focus to TopBar after selection
                            app.current_action = None;

                            // Fetch details and schemas for the newly selected item
                            // Explicitly get index first, then ID, then call fetches
                            let selected_index =
                                app.change_set_list_state.selected(); // Get index *before* potentially changing state further

                            if let Some(index) = selected_index {
                                // Now try to get the summary based on the index
                                if let Some(selected_cs) = app
                                    .change_sets
                                    .as_ref()
                                    .and_then(|css| css.get(index))
                                {
                                    let cs_id = selected_cs.id.clone(); // Clone the ID
                                    if let Some(ws_id) = workspace_id.clone() {
                                        // Clone ws_id too
                                        app.current_action = Some(
                                            "Fetching details, schemas & components..." // Updated action message
                                                .to_string(),
                                        );
                                        terminal.draw(|f| ui(f, app))?; // Redraw immediately
                                        // Now call with the cloned IDs
                                        fetch_details_and_status(
                                            app, &ws_id, &cs_id,
                                        )
                                        .await;
                                        fetch_schemas(app, &ws_id, &cs_id)
                                            .await;
                                        fetch_components(app, &ws_id, &cs_id) // Added call to fetch components
                                            .await;
                                        app.current_action = None;
                                    } else {
                                        // Handle missing ws_id case if necessary, though unlikely here
                                        app.add_log_auto_scroll("Workspace ID missing unexpectedly.".to_string(), LOG_HEIGHT);
                                        // Clear details...
                                        app.selected_change_set_details = None;
                                        app.selected_change_set_merge_status =
                                            None;
                                        app.selected_change_set_components =
                                            None; // Clear components too
                                        app.schemas.clear();
                                        app.schema_list_state.select(None);
                                    }
                                } else {
                                    // Handle case where index is valid but CS not found (shouldn't happen)
                                    app.add_log_auto_scroll(
                                        "Selected changeset not found."
                                            .to_string(),
                                        LOG_HEIGHT,
                                    );
                                    // Clear details...
                                    app.selected_change_set_details = None;
                                    app.selected_change_set_merge_status = None;
                                    app.selected_change_set_components = None; // Clear components too
                                    app.schemas.clear();
                                    app.schema_list_state.select(None);
                                }
                            } else {
                                // Clear details if no selection or error occurred during fetch
                                app.selected_change_set_details = None;
                                app.selected_change_set_merge_status = None;
                                app.selected_change_set_components = None; // Clear components too
                                app.schemas.clear();
                                app.schema_list_state.select(None);
                            }
                        }
                        KeyCode::Esc => {
                            // Close dropdown without changing selection, return focus to TopBar
                            app.changeset_dropdown_active = false;
                            app.current_focus = AppFocus::TopBar; // Return focus
                            app.current_action = None;
                        }
                        KeyCode::Tab => {
                            // Tab cycles focus even when dropdown is open, close dropdown first
                            app.changeset_dropdown_active = false; // Close dropdown
                            app.current_focus = AppFocus::SchemaList; // Move focus according to Tab cycle
                            app.current_action = None;
                        }
                        _ => {} // Ignore other keys for now
                    }
                } // End AppFocus::ChangeSetDropdown

                // --- Focus: Input (Should not be reachable in Normal Mode) ---
                // --- Focus: Input (Should not be reachable in Normal Mode) ---
                // This state should only be active when the respective UI element is active.
                // If focus somehow lands here incorrectly, redirect it.
                AppFocus::Input => {
                    // Removed ChangeSetDropdown from this arm as it's handled above
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
                            app.current_action =
                                Some(format!("Creating '{}'...", new_cs_name));
                            terminal.draw(|f| ui(f, app))?; // Redraw to show action
                            let request = CreateChangeSetV1Request {
                                change_set_name: new_cs_name.clone(),
                            };
                            match api_client::create_change_set(&ws_id, request)
                                .await
                            {
                                Ok((created_cs_response, logs)) => {
                                    let new_change_set_id = created_cs_response
                                        .change_set
                                        .id
                                        .clone();
                                    logs.into_iter().for_each(|log| {
                                        app.add_log_auto_scroll(log, LOG_HEIGHT)
                                    });
                                    app.add_log_auto_scroll(
                                        format!(
                                            "Created changeset '{}' ({})",
                                            created_cs_response.change_set.name,
                                            &new_change_set_id
                                        ),
                                        LOG_HEIGHT,
                                    );
                                    refresh_change_sets(app).await; // Refresh list
                                    app.select_change_set_by_id(
                                        &new_change_set_id,
                                    ); // Select the new one
                                    fetch_schemas(
                                        app,
                                        &ws_id,
                                        &new_change_set_id,
                                    )
                                    .await; // Fetch schemas for new CS
                                }
                                Err(e) => {
                                    app.add_log_auto_scroll(
                                        format!(
                                            "Error creating changeset: {}",
                                            e
                                        ),
                                        LOG_HEIGHT,
                                    );
                                    refresh_change_sets(app).await; // Refresh even on error
                                    // Clear schemas if creation failed but list refreshed
                                    app.schemas.clear();
                                    app.schema_list_state.select(None);
                                }
                            }
                        } else {
                            app.add_log_auto_scroll(
                                "Change set name cannot be empty.".to_string(),
                                LOG_HEIGHT,
                            );
                        }
                    } else {
                        app.add_log_auto_scroll(
                            "Cannot create: Workspace ID missing.".to_string(),
                            LOG_HEIGHT,
                        );
                    }
                    // Reset state after submission or error
                    app.input_mode = InputMode::Normal;
                    app.current_focus = AppFocus::TopBar; // Return focus to TopBar
                    app.input_buffer.clear();
                    app.current_action = None;
                }
                KeyCode::Char(c) => app.input_buffer.push(c),
                KeyCode::Backspace => {
                    app.input_buffer.pop();
                }
                KeyCode::Esc => {
                    // Cancel input mode
                    app.input_mode = InputMode::Normal;
                    app.current_focus = AppFocus::TopBar; // Return focus to TopBar
                    app.input_buffer.clear();
                    app.current_action = None;
                    app.add_log_auto_scroll(
                        "Change set creation cancelled.".to_string(),
                        LOG_HEIGHT,
                    );
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
            logs.into_iter()
                .for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));
            app.add_log_auto_scroll(
                format!("Details fetched for {}", cs_id),
                LOG_HEIGHT,
            );
        }
        Err(e) => {
            app.selected_change_set_details = None; // Clear on error
            app.add_log_auto_scroll(
                format!("Error fetching details for {}: {}", cs_id, e),
                LOG_HEIGHT,
            );
        }
    }
    // Fetch merge status
    match api_client::get_merge_status(ws_id, cs_id).await {
        Ok((status_response, logs)) => {
            app.selected_change_set_merge_status = Some(status_response);
            logs.into_iter()
                .for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));
            app.add_log_auto_scroll(
                format!("Merge status fetched for {}", cs_id),
                LOG_HEIGHT,
            );
        }
        Err(e) => {
            app.selected_change_set_merge_status = None; // Clear on error
            app.add_log_auto_scroll(
                format!("Error fetching merge status for {}: {}", cs_id, e),
                LOG_HEIGHT,
            );
        }
    }
}

// Intention: Fetch the list of components for the given workspace and change set.
// Design Choice: Encapsulate component fetching logic. Updates app state.
async fn fetch_components(app: &mut App, ws_id: &str, cs_id: &str) {
    app.add_log_auto_scroll(
        format!("Fetching components for change set {}...", cs_id),
        LOG_HEIGHT,
    );
    match api_client::list_components(ws_id, cs_id).await {
        Ok((components_response, mut api_logs)) => {
            // Make logs mutable
            // Add API client logs first
            api_logs
                .drain(..)
                .for_each(|log| app.add_log_auto_scroll(log, LOG_HEIGHT));

            // Log the component IDs
            let num_components = components_response.components.len();
            app.add_log_auto_scroll(
                format!(
                    "DEBUG: Received {} component IDs from API.",
                    num_components
                ),
                LOG_HEIGHT,
            );

            // Log the component IDs for debugging
            for (i, component_id) in
                components_response.components.iter().enumerate()
            {
                app.add_log_auto_scroll(
                    format!("DEBUG: Component ID {}: {}", i, component_id),
                    LOG_HEIGHT,
                );
            }

            // For now, create dummy ComponentViewV1 objects with the IDs
            // In a real implementation, you would fetch the component details for each ID
            let components = components_response
                .components
                .iter()
                .map(|id| {
                    ComponentViewV1 {
                        id: id.clone(),
                        schema_id: "unknown".to_string(), // We don't need to filter by schema ID
                        schema_variant_id: "unknown".to_string(),
                        sockets: Vec::new(),
                        domain_props: Vec::new(),
                        resource_props: Vec::new(),
                        name: id.clone(), // Use the ID as the name for now
                        resource_id: "unknown".to_string(),
                        to_delete: false,
                        can_be_upgraded: false,
                        connections: Vec::new(),
                        views: Vec::new(),
                    }
                })
                .collect::<Vec<_>>();

            app.selected_change_set_components = Some(components);
            app.add_log_auto_scroll(
                format!(
                    "Successfully processed {} component IDs.",
                    num_components
                ),
                LOG_HEIGHT,
            );
        }
        Err(e) => {
            // Log the detailed error
            app.add_log_auto_scroll(
                format!("ERROR fetching components: {:?}", e), // Use debug format for full error
                LOG_HEIGHT,
            );
            // Ensure state is cleared on error
            app.selected_change_set_components = None;
            app.add_log_auto_scroll(
                "Cleared component state due to fetch error.".to_string(),
                LOG_HEIGHT,
            );
        }
    }
}

// Intention: Fetch the list of schemas for the given workspace and change set.
// Design Choice: Encapsulate schema fetching logic. Updates app state.
async fn fetch_schemas(app: &mut App, ws_id: &str, cs_id: &str) {
    app.add_log_auto_scroll(
        format!("Fetching schemas for change set {}...", cs_id),
        LOG_HEIGHT,
    );
    match api_client::list_schemas(ws_id, cs_id).await {
        Ok(schema_response) => {
            // Removed 'mut'
            // Store the full SchemaSummary vector
            app.schemas = schema_response.schemas;
            // Sort by category, then by schema name
            app.schemas.sort_unstable_by(|a, b| {
                a.category
                    .cmp(&b.category)
                    .then_with(|| a.schema_name.cmp(&b.schema_name))
            });
            // Remove the incorrect sort_unstable() call that caused the Ord error
            // app.schemas.sort_unstable(); // Remove this line
            // Select first item if list is not empty, otherwise clear selection
            if !app.schemas.is_empty() {
                app.schema_list_state.select(Some(0));
            } else {
                app.schema_list_state.select(None);
            }
            app.add_log_auto_scroll(
                "Successfully fetched schemas.".to_string(),
                LOG_HEIGHT,
            );
        }
        Err(e) => {
            app.schemas.clear(); // Clear schemas on error
            app.schema_list_state.select(None); // Clear selection on error
            app.add_log_auto_scroll(
                format!("Error fetching schemas: {}", e),
                LOG_HEIGHT,
            );
        }
    }
}
