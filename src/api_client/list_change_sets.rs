// src/api_client/list_change_sets.rs

// Intention:
// Contains the implementation for the `GET /v1/w/{workspace_id}/change-sets` API endpoint call.

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
use crate::api_models::ListChangeSetV1Response;

/// Fetches a list of change sets for a given workspace.
/// Intention: Calls the `GET /v1/w/{workspace_id}/change-sets` endpoint.
/// Design: Uses the initialized `reqwest::Client`, constructs the URL with the workspace ID,
///         sends a GET request, and deserializes the JSON response into `ListChangeSetV1Response`.
///         Includes logging similar to the `whoami` function.
/// Returns: A tuple containing the `ListChangeSetV1Response` on success and a `Vec<String>` of log messages.
pub async fn list_change_sets(
    workspace_id: &str,
) -> Result<(ListChangeSetV1Response, Vec<String>), Box<dyn Error + Send + Sync>>
{
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!("{}/v1/w/{}/change-sets", config.base_url, workspace_id);
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let list_response: ListChangeSetV1Response =
            serde_json::from_str(&response_text).map_err(|e| {
                format!(
                    "Failed to deserialize list change sets response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((list_response, logs))
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
