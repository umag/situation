// src/api_client/get_merge_status.rs

// Intention:
// Contains the implementation for the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status` API endpoint call.

// Design Choices:
// - Uses the shared `get_api_config` function from the parent module.
// - Handles response status and deserialization.
// - Logs request and response details.

use std::error::Error;

// Use the shared config getter and ApiError type from the parent module
use super::{
    ApiError,
    get_api_config,
};
// Import the specific response model needed for this function
use crate::api_models::MergeStatusV1Response;

/// Fetches the merge status for a specific change set.
/// Corresponds to `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status`.
/// Operation ID: `merge_status`
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace containing the change set.
/// * `change_set_id` - The ID of the change set to get the status for.
///
/// # Returns
/// A `Result` containing the `MergeStatusV1Response` on success, or an error string on failure.
/// Also returns a `Vec<String>` containing logs generated during the call.
///
/// # Intention
/// Provides the functionality to retrieve the merge status (including actions) for a change set.
///
/// # Design
/// - Constructs the specific URL for the merge status endpoint.
/// - Uses the shared `reqwest` client and configuration (via `get_api_config`).
/// - Sends an HTTP GET request.
/// - Handles success and error responses similarly to other API client functions.
/// - Deserializes the success response into `MergeStatusV1Response`.
/// - Logs relevant information about the request and response.
pub async fn get_merge_status(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<(MergeStatusV1Response, Vec<String>), Box<dyn Error + Send + Sync>>
{
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/merge_status", // Added /merge_status
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let merge_status_response: MergeStatusV1Response = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Failed to deserialize merge status response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((merge_status_response, logs))
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
