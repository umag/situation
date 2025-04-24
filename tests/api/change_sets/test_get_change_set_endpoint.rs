// tests/api/change_sets/test_get_change_set_endpoint.rs

// Intention: Test the get change set endpoint.

use chrono::Utc;
use dotenvy::dotenv;
use situation::{
    api_client,
    api_models,
};
use tokio::time::sleep;

// Import helper function from the same directory
use super::helpers::get_workspace_id;

/// Test Case: Verify the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}` endpoint call.
/// Intention: Ensure the application can correctly call the GET endpoint for a specific
///            change set and handle a successful response.
/// Design: This test first creates a new change set, then uses its ID to make a GET request
///         to retrieve the specific change set details. It asserts that the response indicates
///         success and contains the change set object.
///         Requires a running SI instance and valid .env configuration.
#[tokio::test]
async fn test_get_change_set_endpoint() {
    dotenv().ok(); // Load .env file
    let workspace_id = get_workspace_id()
        .await
        .expect("Failed to get workspace_id for test");
    let change_set_name =
        format!("test-get-changeset-{}", Utc::now().timestamp_millis());

    // 1. Create a change set to get an ID
    let create_request_body = api_models::CreateChangeSetV1Request {
        change_set_name: change_set_name.clone(),
    };
    let create_result =
        api_client::create_change_set(&workspace_id, create_request_body).await;
    assert!(
        create_result.is_ok(),
        "Failed to create change set for get test: {:?}",
        create_result.err()
    );
    let (create_response, _logs) = create_result.unwrap();
    // Access the ID directly from the ChangeSet struct
    let change_set_id = create_response.change_set.id.clone();
    assert!(
        !change_set_id.is_empty(),
        "Created change set ID should not be empty"
    );

    // Add a small delay to see if it helps
    sleep(std::time::Duration::from_millis(300)).await; // Increased delay further

    // 2. Get the created change set
    // Assume api_client::get_change_set exists
    let get_result =
        api_client::get_change_set(&workspace_id, &change_set_id).await;

    assert!(
        get_result.is_ok(),
        "API call to get change set should return Ok. Error: {:?}",
        get_result.err()
    );

    // Add explicit type annotation
    let (get_response, _logs): (
        api_models::GetChangeSetV1Response,
        Vec<String>,
    ) = get_result.unwrap();

    // Check the structure based on GetChangeSetV1Response using the ChangeSet struct
    // The type system ensures change_set exists if deserialization succeeded.
    // Verify the ID matches the created one.
    assert_eq!(
        get_response.change_set.id, change_set_id,
        "Fetched change set ID should match the created one"
    );
    // Optionally verify other fields like name
    assert_eq!(
        get_response.change_set.name, change_set_name,
        "Fetched change set name should match"
    );

    // Clean up: Abandon the created change set
    sleep(std::time::Duration::from_millis(100)).await; // Small delay before abandon
    let abandon_result = // Use abandon_change_set
        api_client::abandon_change_set(&workspace_id, &change_set_id).await;
    assert!(
        abandon_result.is_ok(),
        "Failed to abandon change set after get test: {:?}", // Updated message
        abandon_result.err()
    );
}
