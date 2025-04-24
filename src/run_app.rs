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
