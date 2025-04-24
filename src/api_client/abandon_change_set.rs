// src/api_client/abandon_change_set.rs

// Intention:
// Contains the implementation for the `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}` API endpoint call.

// Design Choices:
// - Uses the shared `get_api_config` function from the parent module.
// - Sends a DELETE request.
// - Handles response status and deserialization (expects `{"success": true}`).
// - Logs request and response details.

use std::error::Error;

// Use the shared config getter and ApiError type from the parent module
use super::{
    ApiError,
    get_api_config,
};
// Import the specific response model needed for this function
use crate::api_models::DeleteChangeSetV1Response;

/// Abandons a specific change set by its ID.
/// Corresponds to `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}`.
/// Operation ID: `abandon_change_set` (Matches OpenAPI spec)
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace containing the change set.
/// * `change_set_id` - The ID of the change set to delete.
///
/// # Returns
/// A `Result` containing the `DeleteChangeSetV1Response` (which includes a `success` boolean) on success, or an error string on failure.
/// Also returns a `Vec<String>` containing logs generated during the call.
///
/// # Intention
/// Provides the functionality to abandon a change set via the API.
///
/// # Design
/// - Constructs the specific URL for the change set deletion endpoint.
/// - Uses the shared `reqwest` client and configuration (via `get_api_config`).
/// - Sends an HTTP DELETE request.
/// - Handles success and error responses similarly to other API client functions.
/// - Deserializes the success response into `DeleteChangeSetV1Response` (which contains `{ "success": true }`).
/// - Logs relevant information about the request and response.
pub async fn abandon_change_set(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<
    (DeleteChangeSetV1Response, Vec<String>), // Return type already matches plan
    Box<dyn Error + Send + Sync>,
> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}",
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: DELETE {}", url));

    let response = config.client.delete(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        // Deserialize the response which should contain `{"success": true}`
        let abandon_response: DeleteChangeSetV1Response =
            serde_json::from_str(&response_text).map_err(|e| {
                format!(
                    "Failed to deserialize abandon change set response: {} - Body: {}", // Updated error message
                    e, response_text
                )
            })?;
        // TODO: Consider checking abandon_response.success here? Or let caller handle it.
        Ok((abandon_response, logs)) // Return the deserialized response
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text)
        {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!(
                "API request failed with status {}: {}",
                status, error_text
            ),
        };
        Err(error_message.into())
    }
}
