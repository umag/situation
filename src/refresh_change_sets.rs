// Intention: Define the function responsible for refreshing the change set list.
// Design Choice: Moved from main.rs to its own module. Takes a mutable App reference.
// Uses the api_client to fetch data and updates the App state.

use crate::app::App; // Use App from the local app module
use situation::api_client; // Use api_client from the library crate

// Intention: Helper function to refresh the list of change sets.
// Design Choice: Encapsulates the API call and state update logic.
pub async fn refresh_change_sets(app: &mut App) {
    // Define log height consistent with UI definition
    const LOG_HEIGHT: usize = 10;

    if let Some(whoami_data) = &app.whoami_data {
        let workspace_id = whoami_data.workspace_id.clone();
        app.add_log_auto_scroll(
            format!("Refreshing change sets for workspace {}...", workspace_id),
            LOG_HEIGHT,
        );
        match api_client::list_change_sets(&workspace_id).await {
            Ok((list_response, cs_logs)) => {
                // Preserve selection if possible, otherwise select first or none
                let current_selection = app.change_set_list_state.selected();
                let new_len = list_response.change_sets.len();

                if new_len == 0 {
                    app.change_set_list_state.select(None);
                } else if let Some(selected_idx) = current_selection {
                    // If previous selection index is still valid, keep it
                    if selected_idx >= new_len {
                        app.change_set_list_state.select(Some(new_len - 1)); // Select last if out of bounds
                    } else {
                        // Keep selection - no need to call select
                    }
                } else {
                    // No previous selection or list was empty, select first
                    app.change_set_list_state.select(Some(0));
                }

                app.change_sets = Some(list_response.change_sets);
                // Add logs individually to ensure auto-scroll for each
                for log in cs_logs {
                    app.add_log_auto_scroll(log, LOG_HEIGHT);
                }
                app.add_log_auto_scroll(
                    "Change set list refreshed.".to_string(),
                    LOG_HEIGHT,
                );
            }
            Err(e) => {
                app.change_set_list_state.select(None); // Ensure nothing selected on error
                let error_msg = format!("Error refreshing change sets: {}", e);
                app.add_log_auto_scroll(error_msg, LOG_HEIGHT);
            }
        }
    } else {
        app.add_log_auto_scroll(
            "Cannot refresh change sets: Whoami data not available."
                .to_string(),
            LOG_HEIGHT,
        );
    }
}
