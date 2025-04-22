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
    use tokio::time::sleep; // Add sleep import

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
        // Let's assume the response structure is {"changeSet": {"id": "..."}} for now.
        let created_id = create_response
            .change_set
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        assert!(
            created_id.is_some(),
            "Response should contain a changeSet object with an id"
        );
    }

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
            api_client::create_change_set(&workspace_id, create_request_body)
                .await;
        assert!(
            create_result.is_ok(),
            "Failed to create change set for get test: {:?}",
            create_result.err()
        );
        let (create_response, _logs) = create_result.unwrap();
        let change_set_id = create_response
            .change_set
            .get("id")
            .and_then(|v| v.as_str())
            .expect("Created change set response did not contain an ID")
            .to_string();

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

        // Check the structure based on GetChangeSetV1Response
        assert!(
            !get_response.change_set.is_null(),
            "Response should contain a changeSet object"
        );
        // Optionally, verify the ID matches if the response structure allows
        let fetched_id = get_response
            .change_set
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        assert_eq!(
            fetched_id,
            Some(change_set_id),
            "Fetched change set ID should match the created one"
        );
    }

    /// Test Case: Verify the `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}` endpoint call.
    /// Intention: Ensure the application can correctly call the DELETE endpoint for a specific
    ///            change set and handle a successful response indicating deletion.
    /// Design: This test first creates a new change set, then uses its ID to make a DELETE request.
    ///         It asserts that the response indicates success (`success: true`).
    ///         Requires a running SI instance and valid .env configuration.
    #[tokio::test]
    async fn test_delete_change_set_endpoint() {
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
            api_client::create_change_set(&workspace_id, create_request_body)
                .await;
        assert!(
            create_result.is_ok(),
            "Failed to create change set for delete test: {:?}",
            create_result.err()
        );
        let (create_response, _logs) = create_result.unwrap();
        let change_set_id = create_response
            .change_set
            .get("id")
            .and_then(|v| v.as_str())
            .expect("Created change set response did not contain an ID")
            .to_string();

        // Add a small delay to allow the system to process the creation if needed
        sleep(std::time::Duration::from_millis(100)).await;

        // 2. Delete the created change set
        // Assume api_client::delete_change_set exists
        let delete_result =
            api_client::delete_change_set(&workspace_id, &change_set_id).await;

        assert!(
            delete_result.is_ok(),
            "API call to delete change set should return Ok. Error: {:?}",
            delete_result.err()
        );

        // Add explicit type annotation
        let (delete_response, _logs): (
            api_models::DeleteChangeSetV1Response,
            Vec<String>,
        ) = delete_result.unwrap();

        // Check the structure based on DeleteChangeSetV1Response
        assert!(
            delete_response.success,
            "Response should indicate success (success: true)"
        );

        // Optional: Verify deletion by trying to GET the change set again (expecting an error)
        // sleep(std::time::Duration::from_millis(100)).await; // Delay before checking
        // let get_result_after_delete = api_client::get_change_set(&workspace_id, &change_set_id).await;
        // assert!(get_result_after_delete.is_err(), "Getting the change set after deletion should fail.");
        // Note: The exact error type/status code for getting a deleted change set isn't specified,
        // so checking for `is_err()` is a basic verification.
    }

    // TODO: Add tests for other change set operations (apply, etc.)

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
            api_client::create_change_set(&workspace_id, create_request_body)
                .await;
        assert!(
            create_result.is_ok(),
            "Failed to create change set for merge status test: {:?}",
            create_result.err()
        );
        let (create_response, _logs) = create_result.unwrap();
        let change_set_id = create_response
            .change_set
            .get("id")
            .and_then(|v| v.as_str())
            .expect("Created change set response did not contain an ID")
            .to_string();

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

        // Check the structure based on MergeStatusV1Response
        assert!(
            !merge_status_response.change_set.is_null(),
            "Response should contain a changeSet object"
        );
        // Note: A newly created change set might have an empty actions array,
        // so we don't assert !is_empty(). Deserialization success implies the field exists.

        // Optionally, verify the change set ID matches if the response structure allows
        let fetched_cs_id = merge_status_response
            .change_set
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let expected_id_str = change_set_id.as_str(); // Borrow explicitly before assertion
        assert_eq!(
            fetched_cs_id.as_deref(),
            Some(expected_id_str), // Use the borrowed slice here
            "Merge status change set ID should match the created one"
        );

        // Add delay before cleanup
        sleep(std::time::Duration::from_millis(100)).await;

        // Clean up: Delete the change set
        let delete_result =
            api_client::delete_change_set(&workspace_id, &change_set_id).await;
        assert!(
            delete_result.is_ok(),
            "Failed to delete change set after merge status test: {:?}",
            delete_result.err()
        );
    }
}
