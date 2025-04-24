// tests/api/change_sets/helpers.rs

// Intention: Contains helper functions for change set API tests.

use std::env;

use dotenvy::dotenv;
use situation::api_client; // Use the library crate name 'situation'

// Helper function to get workspace_id (could be moved to a shared test utils module later)
// For now, assumes it's set directly in .env or fetched via whoami if needed
pub(super) async fn get_workspace_id() -> Result<String, String> {
    dotenv().ok(); // Load .env file

    // Try getting from env var first
    match env::var("WORKSPACE_ID") {
        Ok(id) => Ok(id), // Return Ok if found
        Err(_) => {
            // If not in env, try fetching from whoami
            match api_client::whoami().await {
                // Remove incorrect type annotation from pattern
                Ok((whoami_data, _logs)) => Ok(whoami_data.workspace_id),
                Err(e) => Err(format!(
                    "WORKSPACE_ID not in .env and failed to get from whoami: {}",
                    e
                )),
            }
        }
    }
}
