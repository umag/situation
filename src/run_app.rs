// Intention: Define the main application loop, handling events and state updates.
// Design Choice: Contains the core TUI logic, including the main loop and initial data fetching.
// Event handling is delegated to the `event_handler` submodule.

mod event_handler; // Declare the submodule file

use std::{
    io,
    time::Duration,
};

use crossterm::event::{
    self,
    Event,
    KeyCode, // Keep KeyCode if needed for non-event_handler logic, otherwise remove
    KeyEvent, // Needed for the event::read pattern
};
use event_handler::handle_key_event; // Import from the declared submodule
use ratatui::{
    Terminal,
    backend::Backend,
};
use situation::api_client; // Use api_client from the library crate
use situation::api_models::CreateChangeSetV1Request; // Use specific model

use crate::app::App; // Use App from local app module
use crate::refresh_change_sets::refresh_change_sets; // Use refresh function from local module
use crate::ui::ui; // Use ui function from local module // Import the new handler function

// Intention: Main application loop for initializing, fetching data, rendering UI, and dispatching events.
// Design Choice: A loop that initializes state, fetches data, draws UI, and handles input asynchronously.
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // Intention: Initialize application state using the new constructor.
    let mut app = App::new();
    // Define log height consistent with UI definition here as well
    const LOG_HEIGHT: usize = 10;

    // Intention: Perform initial data fetch (whoami and change sets) and log the process.
    // Design Choice: Call whoami first, then list_change_sets if whoami succeeds.
    app.add_log_auto_scroll(
        "Fetching initial /whoami data...".to_string(),
        LOG_HEIGHT,
    );
    match api_client::whoami().await {
        Ok((whoami_data, whoami_logs)) => {
            let _workspace_id = whoami_data.workspace_id.clone(); // Prefix with _ as it's not directly used here
            app.whoami_data = Some(whoami_data);
            // Add logs individually to ensure auto-scroll
            for log in whoami_logs {
                app.add_log_auto_scroll(log, LOG_HEIGHT);
            }
            app.add_log_auto_scroll(
                "/whoami call successful.".to_string(),
                LOG_HEIGHT,
            );
            // Initial fetch of change sets
            refresh_change_sets(&mut app).await; // This populates app.change_sets and might select one

            // After fetching change sets, try to fetch schemas for the selected one
            if let Some(selected_cs) = app.get_selected_changeset_summary() {
                let cs_id = selected_cs.id.clone();
                let workspace_id =
                    app.whoami_data.as_ref().unwrap().workspace_id.clone(); // Safe unwrap due to check above
                app.add_log_auto_scroll(
                    format!("Fetching schemas for change set {}...", cs_id),
                    LOG_HEIGHT,
                );
                match api_client::list_schemas(&workspace_id, &cs_id).await {
                    Ok(schema_response) => {
                        // Removed 'mut'
                        // Store the full SchemaSummary vector directly
                        app.schemas = schema_response.schemas;
                        // Sort by category, then by schema name
                        app.schemas.sort_unstable_by(|a, b| {
                            a.category
                                .cmp(&b.category)
                                .then_with(|| a.schema_name.cmp(&b.schema_name))
                        });
                        // Select the first schema by default if list is not empty
                        if !app.schemas.is_empty() {
                            app.schema_list_state.select(Some(0));
                        }
                        app.add_log_auto_scroll(
                            "Successfully fetched schemas.".to_string(),
                            LOG_HEIGHT,
                        );

                        // Fetch components for the selected change set
                        app.add_log_auto_scroll(
                            format!(
                                "Fetching components for change set {}...",
                                cs_id
                            ),
                            LOG_HEIGHT,
                        );
                        match api_client::list_components(&workspace_id, &cs_id)
                            .await
                        {
                            Ok((components_response, mut api_logs)) => {
                                // Add API client logs first
                                api_logs.drain(..).for_each(|log| {
                                    app.add_log_auto_scroll(log, LOG_HEIGHT)
                                });

                                // Log the component IDs
                                let num_components =
                                    components_response.components.len();
                                app.add_log_auto_scroll(
                                    format!(
                                        "DEBUG: Received {} component IDs from API.",
                                        num_components
                                    ),
                                    LOG_HEIGHT,
                                );

                                // Log the component IDs for debugging
                                for (i, component_id) in components_response
                                    .components
                                    .iter()
                                    .enumerate()
                                {
                                    app.add_log_auto_scroll(
                                        format!(
                                            "DEBUG: Component ID {}: {}",
                                            i, component_id
                                        ),
                                        LOG_HEIGHT,
                                    );
                                }

                                // For now, create dummy ComponentViewV1 objects with the IDs
                                // In a real implementation, you would fetch the component details for each ID
                                let components = components_response
                                    .components
                                    .iter()
                                    .map(|id| {
                                        situation::api_models::ComponentViewV1 {
                                            id: id.clone(),
                                            schema_id: "unknown".to_string(), // We don't need to filter by schema ID
                                            schema_variant_id: "unknown"
                                                .to_string(),
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

                                app.selected_change_set_components =
                                    Some(components);
                                app.add_log_auto_scroll(
                                    format!(
                                        "Successfully processed {} component IDs.",
                                        num_components
                                    ),
                                    LOG_HEIGHT,
                                );
                            }
                            Err(e) => {
                                app.add_log_auto_scroll(
                                    format!(
                                        "Error fetching components: {:?}",
                                        e
                                    ),
                                    LOG_HEIGHT,
                                );
                                app.selected_change_set_components = None;
                            }
                        }
                    }
                    Err(e) => {
                        app.add_log_auto_scroll(
                            format!("Error fetching schemas: {}", e),
                            LOG_HEIGHT,
                        );
                    }
                }
            } else {
                app.add_log_auto_scroll(
                    "No change set selected initially, skipping schema fetch."
                        .to_string(),
                    LOG_HEIGHT,
                );
            }
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
        terminal.draw(|f| ui(f, &mut app))?; // Pass mutable app state to ui

        // Intention: Handle user input events asynchronously by polling and dispatching to the handler.
        // Design Choice: Poll for events, then call the dedicated handler function if it's a key event.
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Call the extracted handler function
                // Pass the mutable app state and terminal reference
                let should_quit =
                    handle_key_event(key, &mut app, terminal).await?;
                if should_quit {
                    return Ok(()); // Exit the loop if the handler signals quit
                }
            }
            // Handle other event types (e.g., Mouse, Resize) here if needed in the future
        }
        // Placeholder for other async tasks or periodic refresh if needed later
        // tokio::time::sleep(Duration::from_millis(50)).await; // Small sleep to prevent busy-looping if no events
    }
    // Note: Loop is infinite, exit happens via `return Ok(())` on 'q' press.
}
