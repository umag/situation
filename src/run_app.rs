// Intention: Define the main application loop, handling events and state updates.
// Design Choice: Moved from main.rs to its own module. Contains the core TUI logic.
// Fetches initial data, handles user input, triggers API calls, and calls the UI rendering function.

use std::{
    io,
    time::Duration,
};

use crossterm::event::{
    self,
    Event,
    KeyCode,
};
use ratatui::{
    Terminal,
    backend::Backend,
};
use situation::api_client; // Use api_client from the library crate
use situation::api_models::CreateChangeSetV1Request; // Use specific model

use crate::app::{
    App,
    DropdownFocus,
    InputMode,
}; // Use App, Enums from local app module
use crate::refresh_change_sets::refresh_change_sets; // Use refresh function from local module
use crate::ui::ui; // Use ui function from local module

// Intention: Main application loop for handling events and rendering the UI.
// Design Choice: A loop that initializes state, fetches data, draws UI, and handles input asynchronously.
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // Intention: Initialize application state using the new constructor.
    let mut app = App::new();
    // Define log height consistent with UI definition here as well
    const LOG_HEIGHT: usize = 10;

    // Intention: Perform initial data fetch (whoami and change sets) and log the process.
    // Design Choice: Call whoami first, then list_change_sets if whoami succeeds.
    app.add_log_auto_scroll("Fetching initial /whoami data...".to_string(), LOG_HEIGHT);
    match api_client::whoami().await {
        Ok((whoami_data, whoami_logs)) => {
            let _workspace_id = whoami_data.workspace_id.clone(); // Prefix with _ as it's not directly used here
            app.whoami_data = Some(whoami_data);
            // Add logs individually to ensure auto-scroll
            for log in whoami_logs {
                app.add_log_auto_scroll(log, LOG_HEIGHT);
            }
            app.add_log_auto_scroll("/whoami call successful.".to_string(), LOG_HEIGHT);
            // Initial fetch of change sets
            refresh_change_sets(&mut app).await;
        }
        Err(e) => {
            // Log the error message for whoami failure into the app's log buffer.
            let error_msg = format!("Error fetching initial data: {}", e);
            app.add_log_auto_scroll(error_msg, LOG_HEIGHT);
            // Optionally, still print to stderr during development if helpful
            // eprintln!("Error fetching initial data: {}", e);
        }
    }

    loop {
        // Intention: Draw the current state of the UI using app state.
        terminal.draw(|f| ui(f, &app))?; // Pass app state to ui

        // Intention: Handle user input events asynchronously.
        // Design Choice: Poll for events, handle keys based on mode and dropdown state.
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Clone necessary data *before* the mode match
                let workspace_id = app.whoami_data.as_ref().map(|d| d.workspace_id.clone());
                // Use the helper method to get selected summary/ID
                let selected_cs_summary = app.get_selected_changeset_summary();
                let selected_cs_id = selected_cs_summary.map(|cs| cs.id.clone());

                match app.input_mode {
                    InputMode::Normal => {
                        let log_height = 10; // Must match the Constraint::Length in ui()

                        // Handle dropdown state first if active
                        if app.changeset_dropdown_active {
                            match key.code {
                                KeyCode::Up => app.change_set_previous(),
                                KeyCode::Down => app.change_set_next(),
                                KeyCode::Enter => {
                                    // Select item, close dropdown
                                    app.changeset_dropdown_active = false;
                                    // Clear details, they need explicit fetch now
                                    app.selected_change_set_details = None;
                                    app.selected_change_set_merge_status = None;
                                    app.current_action = None; // Clear any lingering action message
                                }
                                KeyCode::Esc => {
                                    // Close dropdown without changing selection
                                    app.changeset_dropdown_active = false;
                                    app.current_action = None;
                                }
                                // Allow Tab to close and switch focus even when dropdown is open
                                KeyCode::Tab => {
                                    app.changeset_dropdown_active = false; // Close dropdown first
                                    app.dropdown_focus = match app.dropdown_focus {
                                        DropdownFocus::Workspace => DropdownFocus::ChangeSet,
                                        DropdownFocus::ChangeSet => DropdownFocus::Workspace,
                                    };
                                    app.current_action = None;
                                }
                                _ => {} // Other keys ignored when dropdown is active
                            }
                        } else {
                            // Normal mode, dropdown closed
                            match key.code {
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('k') => app.scroll_logs_up(),
                                KeyCode::Char('j') => app.scroll_logs_down(log_height),
                                KeyCode::Tab => {
                                    // Switch focus between triggers
                                    app.dropdown_focus = match app.dropdown_focus {
                                        DropdownFocus::Workspace => DropdownFocus::ChangeSet,
                                        DropdownFocus::ChangeSet => DropdownFocus::Workspace,
                                    };
                                }
                                KeyCode::Char(' ') | KeyCode::Enter => {
                                    // Activate focused element
                                    match app.dropdown_focus {
                                        DropdownFocus::Workspace => {
                                            // Do nothing for now, workspace selection not implemented
                                            app.add_log_auto_scroll(
                                                "Workspace selection not implemented.".to_string(),
                                                LOG_HEIGHT,
                                            );
                                        }
                                        DropdownFocus::ChangeSet => {
                                            // If Enter is pressed on CS trigger:
                                            // 1. If dropdown is closed, open it.
                                            // 2. If dropdown was already open (shouldn't happen here, but safety), close it.
                                            // 3. If dropdown is closed AND an item is selected, fetch details.
                                            if !app.changeset_dropdown_active {
                                                if app
                                                    .change_sets
                                                    .as_ref()
                                                    .map_or(false, |cs| !cs.is_empty())
                                                {
                                                    app.changeset_dropdown_active = true; // Open dropdown
                                                    // Ensure selection is valid if opening
                                                    if app
                                                        .change_set_list_state
                                                        .selected()
                                                        .is_none()
                                                    {
                                                        app.change_set_list_state.select(Some(0));
                                                    }
                                                } else {
                                                    app.add_log_auto_scroll(
                                                        "No change sets to select.".to_string(),
                                                        LOG_HEIGHT,
                                                    );
                                                }
                                            } else {
                                                // This case should be handled by the block above, but for safety:
                                                app.changeset_dropdown_active = false;
                                            }

                                            // If Enter is pressed and dropdown is closed, fetch details for selected
                                            if !app.changeset_dropdown_active {
                                                if let (Some(ws_id), Some(cs_id)) =
                                                    (workspace_id.clone(), selected_cs_id.clone())
                                                {
                                                    app.current_action = Some(
                                                        "Fetching details & status...".to_string(),
                                                    );
                                                    terminal.draw(|f| ui(f, &app))?; // Redraw

                                                    // Fetch details
                                                    match api_client::get_change_set(&ws_id, &cs_id)
                                                        .await
                                                    {
                                                        Ok((get_response, logs)) => {
                                                            app.selected_change_set_details =
                                                                Some(get_response.change_set);
                                                            // Add logs individually
                                                            for log in logs {
                                                                app.add_log_auto_scroll(
                                                                    log, LOG_HEIGHT,
                                                                );
                                                            }
                                                            app.add_log_auto_scroll(
                                                                format!(
                                                                    "Details fetched for {}",
                                                                    cs_id
                                                                ),
                                                                LOG_HEIGHT,
                                                            );
                                                        }
                                                        Err(e) => {
                                                            app.selected_change_set_details = None;
                                                            app.add_log_auto_scroll(format!("Error fetching details for {}: {}", cs_id, e), LOG_HEIGHT);
                                                        }
                                                    }
                                                    // Fetch merge status
                                                    match api_client::get_merge_status(
                                                        &ws_id, &cs_id,
                                                    )
                                                    .await
                                                    {
                                                        Ok((status_response, logs)) => {
                                                            app.selected_change_set_merge_status =
                                                                Some(status_response);
                                                            // Add logs individually
                                                            for log in logs {
                                                                app.add_log_auto_scroll(
                                                                    log, LOG_HEIGHT,
                                                                );
                                                            }
                                                            app.add_log_auto_scroll(
                                                                format!(
                                                                    "Merge status fetched for {}",
                                                                    cs_id
                                                                ),
                                                                LOG_HEIGHT,
                                                            );
                                                        }
                                                        Err(e) => {
                                                            app.selected_change_set_merge_status =
                                                                None;
                                                            app.add_log_auto_scroll(format!("Error fetching merge status for {}: {}", cs_id, e), LOG_HEIGHT);
                                                        }
                                                    }
                                                    app.current_action = None;
                                                } else {
                                                    app.add_log_auto_scroll("Cannot fetch details: No workspace or change set selected.".to_string(), LOG_HEIGHT);
                                                }
                                            }
                                        }
                                    }
                                }

                                // --- Change Set Actions (operate on selection from state) ---
                                KeyCode::Char('d') => {
                                    if let (Some(ws_id), Some(cs_id)) =
                                        (workspace_id.clone(), selected_cs_id.clone())
                                    {
                                        app.current_action = Some(format!("Deleting {}...", cs_id));
                                        terminal.draw(|f| ui(f, &app))?;

                                        // Use the renamed function abandon_change_set
                                        match api_client::abandon_change_set(&ws_id, &cs_id).await {
                                            // The response now contains `success: bool`, we can log it or check it.
                                            // For now, just log the success message as before if Ok.
                                            Ok((abandon_response, logs)) => {
                                                // Add logs individually
                                                for log in logs {
                                                    app.add_log_auto_scroll(log, LOG_HEIGHT);
                                                }
                                                app.add_log_auto_scroll(
                                                    format!(
                                                        "Abandoned changeset {} (Success: {})", // Updated log message
                                                        cs_id, abandon_response.success
                                                    ),
                                                    LOG_HEIGHT,
                                                );
                                                // Clear details if they were for the abandoned item
                                                app.selected_change_set_details = None;
                                                app.selected_change_set_merge_status = None;
                                            }
                                            Err(e) => {
                                                app.add_log_auto_scroll(
                                                    format!(
                                                        "Error abandoning changeset {}: {}",
                                                        cs_id, e
                                                    ),
                                                    LOG_HEIGHT,
                                                ); // Updated error message
                                            }
                                        }
                                        app.current_action = None;
                                        refresh_change_sets(&mut app).await; // Refresh list
                                        // After refresh, clear details again as selection might change
                                        app.selected_change_set_details = None;
                                        app.selected_change_set_merge_status = None;
                                    } else {
                                        app.add_log_auto_scroll(
                                            "Cannot delete: No change set selected.".to_string(),
                                            LOG_HEIGHT,
                                        );
                                    }
                                }
                                KeyCode::Char('c') => {
                                    if workspace_id.is_some() {
                                        app.input_mode = InputMode::ChangeSetName;
                                        app.input_buffer.clear();
                                        // Prompt is now handled by the input line rendering in ui()
                                        app.current_action = None; // Clear any other action
                                    } else {
                                        app.add_log_auto_scroll(
                                            "Cannot create: No workspace available.".to_string(),
                                            LOG_HEIGHT,
                                        );
                                    }
                                }
                                KeyCode::Char('f') => {
                                    if let (Some(ws_id), Some(cs_id)) =
                                        (workspace_id.clone(), selected_cs_id.clone())
                                    {
                                        app.current_action = Some(format!("Applying {}...", cs_id));
                                        terminal.draw(|f| ui(f, &app))?;

                                        // Use the renamed function force_apply
                                        match api_client::force_apply(&ws_id, &cs_id).await {
                                            Ok(((), logs)) => {
                                                // Response is unit tuple ()
                                                // Add logs individually
                                                for log in logs {
                                                    app.add_log_auto_scroll(log, LOG_HEIGHT);
                                                }
                                                app.add_log_auto_scroll(
                                                    format!(
                                                        "Apply initiated for changeset {}",
                                                        cs_id
                                                    ),
                                                    LOG_HEIGHT,
                                                );
                                                // Clear details as status might change
                                                app.selected_change_set_details = None;
                                                app.selected_change_set_merge_status = None;
                                            }
                                            Err(e) => {
                                                app.add_log_auto_scroll(
                                                    format!(
                                                        "Error applying changeset {}: {}",
                                                        cs_id, e
                                                    ),
                                                    LOG_HEIGHT,
                                                );
                                            }
                                        }
                                        app.current_action = None;
                                        refresh_change_sets(&mut app).await; // Refresh list
                                        // Clear details after refresh
                                        app.selected_change_set_details = None;
                                        app.selected_change_set_merge_status = None;
                                    } else {
                                        app.add_log_auto_scroll(
                                            "Cannot apply: No change set selected.".to_string(),
                                            LOG_HEIGHT,
                                        );
                                    }
                                }
                                _ => {} // Ignore other keys
                            }
                        } // End Normal Mode, dropdown closed
                    } // End Normal Mode Match KeyCode

                    InputMode::ChangeSetName => {
                        // ChangeSetName input mode key handling (mostly unchanged)
                        let current_workspace_id = workspace_id.clone();
                        match key.code {
                            KeyCode::Enter => {
                                if let Some(ws_id) = current_workspace_id {
                                    let new_cs_name = app.input_buffer.trim().to_string();
                                    if !new_cs_name.is_empty() {
                                        app.current_action =
                                            Some(format!("Creating '{}'...", new_cs_name));
                                        terminal.draw(|f| ui(f, &app))?;

                                        let request = CreateChangeSetV1Request {
                                            change_set_name: new_cs_name.clone(),
                                        };

                                        match api_client::create_change_set(&ws_id, request).await {
                                            Ok((created_cs_response, logs)) => {
                                                // Store the ID of the newly created change set
                                                let new_change_set_id =
                                                    created_cs_response.change_set.id.clone();

                                                // Add logs individually
                                                for log in logs {
                                                    app.add_log_auto_scroll(log, LOG_HEIGHT);
                                                }
                                                app.add_log_auto_scroll(
                                                    format!(
                                                        "Created changeset '{}' ({})",
                                                        created_cs_response.change_set.name,
                                                        &new_change_set_id // Use stored ID for log
                                                    ),
                                                    LOG_HEIGHT,
                                                );

                                                // Refresh the list *before* trying to select
                                                refresh_change_sets(&mut app).await;

                                                // Intention: Automatically select the newly created change set.
                                                // Design Choice: Call select_change_set_by_id after refreshing the list.
                                                app.select_change_set_by_id(&new_change_set_id);
                                            }
                                            Err(e) => {
                                                app.add_log_auto_scroll(
                                                    format!("Error creating changeset: {}", e),
                                                    LOG_HEIGHT,
                                                );
                                                // Refresh even on error, in case the list changed partially or needs cleanup
                                                refresh_change_sets(&mut app).await;
                                            }
                                        }
                                        // Details are cleared within select_change_set_by_id or if refresh fails implicitly
                                        // app.selected_change_set_details = None; // No longer needed here
                                        // app.selected_change_set_merge_status = // No longer needed here
                                        // None; // This line caused the compile error, removing it.
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
                                app.input_mode = InputMode::Normal;
                                app.input_buffer.clear();
                                app.current_action = None;
                            }
                            KeyCode::Char(c) => app.input_buffer.push(c),
                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.input_buffer.clear();
                                app.current_action = None;
                                app.add_log_auto_scroll(
                                    "Change set creation cancelled.".to_string(),
                                    LOG_HEIGHT,
                                );
                            }
                            _ => {}
                        }
                    } // End ChangeSetName Mode Match KeyCode
                } // End Match app.input_mode
            } // End Key Event Handling
        } // End Event Poll
        // Placeholder for other async tasks or periodic refresh if needed later
        // tokio::time::sleep(Duration::from_millis(50)).await; // Small sleep to prevent busy-looping if no events
    }
    // Note: Loop is infinite, exit happens via `return Ok(())` on 'q' press.
}
