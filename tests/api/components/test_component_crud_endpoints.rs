// tests/api/components/test_component_crud_endpoints.rs

// Intention: Test the component CRUD operations.

use chrono::Utc;
use dotenvy::dotenv;
use serde_json::json;
use situation::{
    api_client,
    api_models,
};
use tokio::time::sleep;

// Import helper functions from the same directory
use super::helpers::{
    abandon_temp_change_set,
    create_temp_change_set,
    get_workspace_id,
};

/// Test Case: Verify the component CRUD operations.
/// Intention: Ensure the application can create, get, update, and delete a component.
/// Design: Creates a change set, then creates a component within it, gets it,
///         updates it, deletes it, and finally abandons the change set.
///         Uses a simple schema like "AWS::Region" for testing.
#[tokio::test]
async fn test_component_crud_endpoints() {
    dotenv().ok();
    let workspace_id = get_workspace_id()
        .await
        .expect("Failed to get workspace_id for test");

    // 1. Create a temporary change set
    let change_set_id = create_temp_change_set(&workspace_id)
        .await
        .expect("Failed to create temp change set for component test");
    sleep(std::time::Duration::from_millis(200)).await; // Delay

    // 2. Create Component
    let component_name =
        format!("test-component-{}", Utc::now().timestamp_millis());
    let create_request = api_models::CreateComponentV1Request {
        // Use a domain appropriate for EC2 Instance, or an empty one if allowed
        domain: json!({}), // Using empty domain for simplicity
        name: component_name.clone(),
        schema_name: "AWS::EC2::Instance".to_string(), // Use a likely valid schema
        connections: vec![], // No connections for this simple test
        view_name: None,
    };

    let create_result = api_client::create_component(
        &workspace_id,
        &change_set_id,
        create_request,
    )
    .await;
    assert!(
        create_result.is_ok(),
        "API call to create component should return Ok. Error: {:?}",
        create_result.err()
    );
    let (create_response, _logs) = create_result.unwrap();
    let component_id = create_response.component_id;
    assert!(
        !component_id.is_empty(),
        "Created component ID should not be empty"
    );
    sleep(std::time::Duration::from_millis(200)).await; // Delay

    // 3. Get Component
    let get_result =
        api_client::get_component(&workspace_id, &change_set_id, &component_id)
            .await;
    assert!(
        get_result.is_ok(),
        "API call to get component should return Ok. Error: {:?}",
        get_result.err()
    );
    let (get_response, _logs) = get_result.unwrap();
    // Basic check: component field should exist (is serde_json::Value)
    assert!(get_response.component.is_object());
    // Domain check removed as we used an empty domain for creation.
    // Add checks based on the actual structure of AWS::EC2::Instance if needed.
    sleep(std::time::Duration::from_millis(200)).await; // Delay

    // 4. Update Component
    let updated_component_name = format!("{}-updated", component_name);
    let update_request = api_models::UpdateComponentV1Request {
        // Update domain with a plausible EC2 property, or keep it simple
        domain: json!({ "ami": "ami-12345678" }), // Example update
        name: Some(updated_component_name.clone()), // Update name
    };
    let update_result = api_client::update_component(
        &workspace_id,
        &change_set_id,
        &component_id,
        update_request,
    )
    .await;
    assert!(
        update_result.is_ok(),
        "API call to update component should return Ok. Error: {:?}",
        update_result.err()
    );
    // Update response is empty `{}`, so Ok is the main check.
    sleep(std::time::Duration::from_millis(200)).await; // Delay

    // 5. Get Component Again (Verify Update)
    let get_after_update_result =
        api_client::get_component(&workspace_id, &change_set_id, &component_id)
            .await;
    assert!(
        get_after_update_result.is_ok(),
        "API call to get component after update should return Ok. Error: {:?}",
        get_after_update_result.err()
    );
    let (get_after_update_response, _logs) = get_after_update_result.unwrap();
    // Check updated domain
    assert_eq!(
        get_after_update_response
            .domain
            .get("ami") // Check the updated property
            .and_then(|v| v.as_str()),
        Some("ami-12345678")
    );
    // Note: Verifying the name update depends on whether the 'get' response includes the name
    // within the `component` object or elsewhere. Adjust assertion as needed.
    sleep(std::time::Duration::from_millis(200)).await; // Delay

    // 6. Delete Component
    let delete_result = api_client::delete_component(
        &workspace_id,
        &change_set_id,
        &component_id,
    )
    .await;
    assert!(
        delete_result.is_ok(),
        "API call to delete component should return Ok. Error: {:?}",
        delete_result.err()
    );
    let (delete_response, _logs) = delete_result.unwrap();
    assert_eq!(
        delete_response.status, "MarkedForDeletion",
        "Delete response status should be MarkedForDeletion"
    );
    sleep(std::time::Duration::from_millis(200)).await; // Delay

    // 7. Clean up: Abandon the temporary change set
    abandon_temp_change_set(&workspace_id, &change_set_id)
        .await
        .expect("Failed to abandon temp change set during cleanup");
}
