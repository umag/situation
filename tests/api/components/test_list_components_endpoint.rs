// tests/api/components/test_list_components_endpoint.rs

// Intention:
// Contains integration tests for the `list_components` API client function,
// verifying its interaction with the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components` endpoint.

// Design Choices:
// - Uses `tokio::test` for asynchronous testing.
// - Leverages helper functions from `tests/api/helpers.rs` (if available) or defines necessary setup locally.
// - Assumes a running mock server or requires environment setup (`.env`) for real API interaction.
// - Creates a temporary change set and potentially a component to ensure the list endpoint has data to return.
// - Cleans up created resources (change set, component) after the test.
// - Asserts that the function returns successfully (`is_ok()`).
// - Asserts that the returned `ListComponentsV1Response` contains a `Vec<ComponentViewV1>`.
// - Potentially asserts specific details about the components if known/mocked.

use std::error::Error;

use situation::api_client::{
    // Changed crate name
    abandon_change_set, // Add other necessary imports like create_component if needed
    create_change_set,
    list_components,
};
use situation::api_models::{
    // Changed crate name
    ComponentViewV1,
    CreateChangeSetV1Request, // Added import
    ListComponentsV1Response,
}; // Import necessary models

// Helper function to create a change set for testing (consider moving to a shared helper)
async fn setup_test_change_set(
    name: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Changed return type
    let workspace_id = std::env::var("TEST_WORKSPACE_ID")
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?; // Map error explicitly
    let request = CreateChangeSetV1Request {
        // Create the request struct
        change_set_name: name.to_string(),
    };
    let (response, _) = create_change_set(&workspace_id, request).await?; // Pass struct
    Ok(response.change_set.id)
}

// Helper function to clean up a change set (consider moving to a shared helper)
async fn cleanup_test_change_set(
    change_set_id: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Changed return type
    let workspace_id = std::env::var("TEST_WORKSPACE_ID")
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?; // Map error explicitly
    abandon_change_set(&workspace_id, change_set_id).await?; // Result type now matches
    Ok(())
}

#[tokio::test]
async fn test_list_components_success()
-> Result<(), Box<dyn Error + Send + Sync>> {
    // Changed return type
    // Setup: Create a temporary change set
    let change_set_name = "test-list-components-cs";
    let change_set_id = setup_test_change_set(change_set_name).await?;
    let workspace_id = std::env::var("TEST_WORKSPACE_ID")
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?; // Map error explicitly

    // TODO: Optionally create a component within this change set first
    //       to ensure the list is not empty. This requires importing
    //       and using `create_component`.

    // Action: Call the list_components function
    let result = list_components(&workspace_id, &change_set_id).await;

    // Assertions
    assert!(result.is_ok(), "list_components failed: {:?}", result.err());
    let (response, logs) = result.unwrap();

    // Check logs (optional)
    assert!(
        logs.iter().any(|log| log.contains(&format!(
            "GET /v1/w/{}/change-sets/{}/components",
            workspace_id, change_set_id
        ))),
        "API call log not found"
    );
    assert!(
        logs.iter().any(|log| log.contains("Status: 200 OK")),
        "Success status log not found"
    );

    // Check that the components array is not empty
    assert!(
        !response.components.is_empty(),
        "Components array should not be empty"
    );

    // Check that all component IDs are non-empty strings
    for component_id in &response.components {
        assert!(!component_id.is_empty(), "Component ID should not be empty");
    }
    // Add more specific assertions if component details are known/mocked
    // e.g., if a component was created, check if its ID/name is in the list.

    // Cleanup: Abandon the temporary change set
    cleanup_test_change_set(&change_set_id)
        .await
        .map_err(|e| e as Box<dyn Error + Send + Sync>)?; // Map error explicitly

    Ok(())
}

// TODO: Add tests for error cases (e.g., invalid change_set_id, unauthorized) if possible/needed.
