// tests/api/change_sets/test_get_merge_status_endpoint.rs

// Intention: Test the get merge status endpoint.

use chrono::Utc;
use dotenvy::dotenv;
use situation::{
    api_client,
    api_models,
};
use tokio::time::sleep;

// Import helper function from the same directory
use super::helpers::get_workspace_id;

/// Test Case: Verify the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status` endpoint call.
/// Intention: Ensure the application can correctly call the GET endpoint for a change set's merge status
///            and handle a successful response.
/// Design: This test first creates a new change set, then uses its ID to make a GET request
///         to retrieve the merge status. It asserts that the response indicates success
///         and contains the expected fields (`changeSet`, `actions`).
///         Requires a running SI instance and valid .env configuration.
#[tokio::test]
async fn test_get_merge_status_endpoint() {
    dotenv().ok(); // Load .env file
    let workspace_id = get_workspace_id()
        .await
        .expect("Failed to get workspace_id for test");
    let change_set_name =
        format!("test-merge-status-{}", Utc::now().timestamp_millis());

    // 1. Create a change set to get an ID
    let create_request_body = api_models::CreateChangeSetV1Request {
        change_set_name: change_set_name.clone(),
    };
    let create_result =
        api_client::create_change_set(&workspace_id, create_request_body).await;
    assert!(
        create_result.is_ok(),
        "Failed to create change set for merge status test: {:?}",
        create_result.err()
    );
    let (create_response, _logs) = create_result.unwrap();
    // Access the ID directly from the ChangeSet struct
    let change_set_id = create_response.change_set.id.clone();
    assert!(
        !change_set_id.is_empty(),
        "Created change set ID should not be empty"
    );

    // Add a small delay
    sleep(std::time::Duration::from_millis(200)).await; // Increased delay

    // 2. Get the merge status for the created change set
    // Assume api_client::get_merge_status exists
    let merge_status_result =
        api_client::get_merge_status(&workspace_id, &change_set_id).await;

    assert!(
        merge_status_result.is_ok(),
        "API call to get merge status should return Ok. Error: {:?}",
        merge_status_result.err()
    );

    // Add explicit type annotation
    let (merge_status_response, _logs): (
        api_models::MergeStatusV1Response, // Assuming this model exists
        Vec<String>,
    ) = merge_status_result.unwrap();

    // Check the structure based on MergeStatusV1Response using the ChangeSet struct
    // The type system ensures change_set exists if deserialization succeeded.
    // Verify the ID matches the created one.
    assert_eq!(
        merge_status_response.change_set.id, change_set_id,
        "Merge status change set ID should match the created one"
    );
    // Note: A newly created change set might have an empty actions array,
    // so we don't assert !is_empty(). Deserialization success implies the field exists.
    // We can assert that the actions field itself exists (is not None, which it can't be here).
    // The `actions` field is Vec<...>, so it exists, just might be empty.

    // Optionally verify other fields like name
    assert_eq!(
        merge_status_response.change_set.name, change_set_name,
        "Merge status change set name should match"
    );

    // Add delay before cleanup - Increased delay to see if it resolves runtime/client issue
    sleep(std::time::Duration::from_millis(500)).await;

    // Clean up: Abandon the change set
    let abandon_result = // Use abandon_change_set
        api_client::abandon_change_set(&workspace_id, &change_set_id).await;
    assert!(
        abandon_result.is_ok(),
        "Failed to abandon change set after merge status test: {:?}", // Updated message
        abandon_result.err()
    );
}
