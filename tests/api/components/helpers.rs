// tests/api/components/helpers.rs

// Intention: Contains helper functions for component API tests.

use std::env;

use chrono::Utc;
use dotenvy::dotenv;
use situation::{
    api_client,
    api_models,
}; // Use the library crate name 'situation'

// Helper function to get workspace_id (copied from change_sets.rs - consider consolidating later)
pub(super) async fn get_workspace_id() -> Result<String, String> {
    dotenv().ok(); // Load .env file
    match env::var("WORKSPACE_ID") {
        Ok(id) => Ok(id),
        Err(_) => match api_client::whoami().await {
            Ok((whoami_data, _logs)) => Ok(whoami_data.workspace_id),
            Err(e) => Err(format!(
                "WORKSPACE_ID not in .env and failed to get from whoami: {}",
                e
            )),
        },
    }
}

// Helper function to create a temporary change set for component tests
pub(super) async fn create_temp_change_set(
    workspace_id: &str,
) -> Result<String, String> {
    let change_set_name =
        format!("test-component-cs-{}", Utc::now().timestamp_millis());
    let request_body = api_models::CreateChangeSetV1Request { change_set_name };
    match api_client::create_change_set(workspace_id, request_body).await {
        Ok((response, _logs)) => Ok(response.change_set.id),
        Err(e) => Err(format!("Failed to create temp change set: {}", e)),
    }
}

// Helper function to abandon a change set (cleanup)
pub(super) async fn abandon_temp_change_set(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<(), String> {
    match api_client::abandon_change_set(workspace_id, change_set_id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to abandon temp change set: {}", e)),
    }
}
