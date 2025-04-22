// tests/api/change_sets.rs

// Intention:
// This module contains integration tests specifically for the change set related API endpoints
// called by the TUI application.

// Design Choices:
// - Uses standard Rust test conventions (`#[cfg(test)]`, `#[tokio::test]`).
// - Each test function focuses on a specific change set operation (list, create, etc.).
// - Placeholder tests are used initially.

use chrono::Utc;
use dotenvy::dotenv;
use situation::{api_client, api_models}; // Use the library crate name 'situation'
use std::env; // Import Utc from chrono

#[cfg(test)]
mod tests {
    use super::*; // Import items from parent module

    // Helper function to get workspace_id (could be more sophisticated later)
    // For now, assumes it's set directly in .env or fetched via whoami if needed
    async fn get_workspace_id() -> Result<String, String> {
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

    /// Test Case: Verify the `GET /v1/w/{workspace_id}/change-sets` endpoint call for listing.
    /// Intention: Ensure the application can correctly call the GET `/change_set` endpoint
    ///            and handle a successful response containing a list of change sets.
    /// Design: This test uses the API client to make a GET request
    ///         to list change sets and asserts that the response indicates success
    ///         and contains a list (potentially empty) of change set summary objects.
    ///         Requires a running SI instance and valid .env configuration.
    #[tokio::test]
    // #[ignore] // Removed: Requires API access, now enabled by user
    async fn test_list_change_sets_endpoint() {
        dotenv().ok(); // Load .env file
        let workspace_id = get_workspace_id()
            .await
            .expect("Failed to get workspace_id for test");

        let result = api_client::list_change_sets(&workspace_id).await;
        assert!(
            result.is_ok(),
            "API call should return Ok. Error: {:?}",
            result.err()
        );

        // Add explicit type annotation to the destructuring let binding
        let (list_response, _logs): (
            api_models::ListChangeSetV1Response,
            Vec<String>,
        ) = result.unwrap();
        // Check the structure based on ListChangeSetV1Response
        assert!(
            list_response // Access the field on the correct struct
                .change_sets
                .iter()
                .all(|cs| cs.id != "" && cs.name != ""),
            "Change sets should have id and name"
        );
    }

    /// Test Case: Verify the `POST /v1/w/{workspace_id}/change-sets` endpoint call for creation.
    /// Intention: Ensure the application can correctly call the POST endpoint to create
    ///            a new change set and handle a successful response.
    /// Design: This test uses the API client to make a POST request with a new change set name.
    ///         It asserts that the response indicates success and returns the created change set details.
    ///         Requires a running SI instance and valid .env configuration.
    #[tokio::test]
    // #[ignore] // Removed: Requires API access, now enabled by user
    async fn test_create_change_set_endpoint() {
        dotenv().ok(); // Load .env file
        let workspace_id = get_workspace_id()
            .await
            .expect("Failed to get workspace_id for test");
        let change_set_name =
            format!("test-changeset-{}", Utc::now().timestamp_millis()); // Unique name using imported Utc

        // Assume api_client::create_change_set exists and takes workspace_id and name
        // We need to define the request body structure based on CreateChangeSetV1Request
        let request_body = api_models::CreateChangeSetV1Request {
            change_set_name: change_set_name.clone(), // Use clone as we need the original later potentially
        };

        let result =
            api_client::create_change_set(&workspace_id, request_body).await; // Pass the request body struct

        assert!(
            result.is_ok(),
            "API call to create change set should return Ok. Error: {:?}",
            result.err()
        );

        // Add explicit type annotation to the destructuring let binding
        let (create_response, _logs): (
            api_models::CreateChangeSetV1Response,
            Vec<String>,
        ) = result.unwrap();

        // Check the structure based on CreateChangeSetV1Response
        // The response schema just has `{"changeSet":{}}`, which isn't very specific in the openapi doc.
        // We'll assume it returns *some* object under `changeSet`.
        // A better test would assert specific fields if the actual response is known.
        assert!(
            !create_response.change_set.is_null(), // Access the field on the correct struct
            "Response should contain a changeSet object"
        );
        // Ideally, we'd get the ID back and maybe verify the name, but the schema is vague.
        // We might need to list change sets again to confirm creation if the response isn't detailed.
    }

    // TODO: Add tests for other change set operations (get, abandon, apply, etc.)
}
