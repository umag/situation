// tests/api/change_sets/test_force_apply_endpoint.rs

// Intention: Test the force apply endpoint.

use chrono::Utc;
use dotenvy::dotenv;
use situation::{
    api_client,
    api_models,
};
use tokio::time::sleep;

// Import helper function from the same directory
use super::helpers::get_workspace_id;

/// Test Case: Verify the `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply` endpoint call.
/// Intention: Ensure the application can correctly call the POST endpoint to force apply a change set.
/// Design: This test first creates a new change set, then uses its ID to make a POST request
///         to force apply it. It asserts that the response indicates success (returns Ok).
///         Requires a running SI instance and valid .env configuration.
///         Note: The API returns 200 OK with no body on success.
#[tokio::test]
async fn test_force_apply_endpoint() {
    // Renamed test function
    dotenv().ok(); // Load .env file
    let workspace_id = get_workspace_id()
        .await
        .expect("Failed to get workspace_id for test");
    let change_set_name =
        format!("test-force-apply-{}", Utc::now().timestamp_millis());

    // 1. Create a change set to get an ID
    let create_request_body = api_models::CreateChangeSetV1Request {
        change_set_name: change_set_name.clone(),
    };
    let create_result =
        api_client::create_change_set(&workspace_id, create_request_body).await;
    assert!(
        create_result.is_ok(),
        "Failed to create change set for force apply test: {:?}",
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
    sleep(std::time::Duration::from_millis(200)).await;

    // 2. Force apply the created change set
    // Use the renamed api_client::force_apply function
    let apply_result = api_client::force_apply(&workspace_id, &change_set_id) // Use renamed function
        .await;

    assert!(
        apply_result.is_ok(),
        "API call to force apply change set should return Ok. Error: {:?}",
        apply_result.err()
    );

    // The success response has no body, so checking for Ok is the main assertion.
    // We get back logs, but no specific response data model.
    let (_response_body_ignored, _logs): ((), Vec<String>) =
        apply_result.unwrap();

    // Add delay before cleanup
    sleep(std::time::Duration::from_millis(100)).await;

    // Clean up: Delete the change set (optional, but good practice if apply doesn't auto-delete)
    // Note: Force applying might merge/abandon the change set automatically.
    // If abandonment fails, it might be expected. We'll log the result but not fail the test.
    let abandon_result = // Use abandon_change_set
        api_client::abandon_change_set(&workspace_id, &change_set_id).await;
    if abandon_result.is_err() {
        println!(
            "Note: Failed to abandon change set after force apply (might be expected): {:?}", // Updated message
            abandon_result.err()
        );
    }
}
