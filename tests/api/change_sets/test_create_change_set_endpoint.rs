// tests/api/change_sets/test_create_change_set_endpoint.rs

// Intention: Test the create change set endpoint.

use chrono::Utc;
use dotenvy::dotenv;
use situation::{
    api_client,
    api_models,
};
use tokio::time::sleep;

// Import helper function from the same directory
use super::helpers::get_workspace_id;

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

    // Check the structure based on CreateChangeSetV1Response using the ChangeSet struct
    // Assert that the ID field is not empty (basic validation)
    assert!(
        !create_response.change_set.id.is_empty(),
        "Created change set ID should not be empty"
    );
    // Assert that the name matches (if needed, though we provided it)
    assert_eq!(
        create_response.change_set.name, change_set_name,
        "Created change set name should match the request"
    );

    // Clean up: Abandon the created change set
    let change_set_id = create_response.change_set.id.clone();
    // Increased delay before abandon to potentially avoid DispatchGone error
    sleep(std::time::Duration::from_millis(500)).await;
    let abandon_result = // Use abandon_change_set
        api_client::abandon_change_set(&workspace_id, &change_set_id).await;
    assert!(
        abandon_result.is_ok(),
        "Failed to abandon change set after create test: {:?}", // Updated message
        abandon_result.err()
    );
}
