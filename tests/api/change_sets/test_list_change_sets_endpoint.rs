// tests/api/change_sets/test_list_change_sets_endpoint.rs

// Intention: Test the list change sets endpoint.

use dotenvy::dotenv;
use situation::{
    api_client,
    api_models,
};

// Import helper function from the same directory
use super::helpers::get_workspace_id;

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
            .all(|cs| !cs.id.is_empty() && !cs.name.is_empty()), // Corrected assertion logic
        "Change sets should have non-empty id and name"
    );
}
