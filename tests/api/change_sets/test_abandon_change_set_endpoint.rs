// tests/api/change_sets/test_abandon_change_set_endpoint.rs

// Intention: Test the abandon change set endpoint.

use chrono::Utc;
use dotenvy::dotenv;
use situation::{
    api_client,
    api_models,
};
use tokio::time::sleep;

// Import helper function from the same directory
use super::helpers::get_workspace_id;

/// Test Case: Verify the `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}` endpoint call (abandon).
/// Intention: Ensure the application can correctly call the DELETE endpoint (operationId: abandon_change_set)
///            for a specific change set and handle a successful response indicating abandonment.
/// Design: This test first creates a new change set, then uses its ID to make a DELETE request.
///         It asserts that the response indicates success (`success: true`).
///         Requires a running SI instance and valid .env configuration.
#[tokio::test]
async fn test_abandon_change_set_endpoint() {
    // Renamed test function
    dotenv().ok(); // Load .env file
    let workspace_id = get_workspace_id()
        .await
        .expect("Failed to get workspace_id for test");
    let change_set_name =
        format!("test-delete-changeset-{}", Utc::now().timestamp_millis());

    // 1. Create a change set to get an ID
    let create_request_body = api_models::CreateChangeSetV1Request {
        change_set_name: change_set_name.clone(),
    };
    let create_result =
        api_client::create_change_set(&workspace_id, create_request_body).await;
    assert!(
        create_result.is_ok(),
        "Failed to create change set for delete test: {:?}",
        create_result.err()
    );
    let (create_response, _logs) = create_result.unwrap();
    // Access the ID directly from the ChangeSet struct
    let change_set_id = create_response.change_set.id.clone();
    assert!(
        !change_set_id.is_empty(),
        "Created change set ID should not be empty"
    );

    // Add a small delay to allow the system to process the creation if needed
    sleep(std::time::Duration::from_millis(100)).await;

    // 2. Abandon the created change set
    // Use the renamed api_client::abandon_change_set function
    let abandon_result =
        api_client::abandon_change_set(&workspace_id, &change_set_id).await;

    assert!(
        abandon_result.is_ok(),
        "API call to abandon change set should return Ok. Error: {:?}", // Updated message
        abandon_result.err()
    );

    // Add explicit type annotation
    let (abandon_response, _logs): (
        // Renamed variable
        api_models::DeleteChangeSetV1Response, // Model name is correct
        Vec<String>,
    ) = abandon_result.unwrap();

    // Check the structure based on DeleteChangeSetV1Response
    assert!(
        abandon_response.success, // Use renamed variable
        "Response should indicate success (success: true)"
    );

    // Optional: Verify deletion by trying to GET the change set again (expecting an error)
    // sleep(std::time::Duration::from_millis(100)).await; // Delay before checking
    // let get_result_after_delete = api_client::get_change_set(&workspace_id, &change_set_id).await;
    // assert!(get_result_after_delete.is_err(), "Getting the change set after deletion should fail.");
    // Note: The exact error type/status code for getting a deleted change set isn't specified,
    // so checking for `is_err()` is a basic verification.
}
