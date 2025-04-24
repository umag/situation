// src/api_client/create_change_set.rs

// Intention:
// Contains the implementation for the `POST /v1/w/{workspace_id}/change-sets` API endpoint call.

// Design Choices:
// - Uses the shared `get_api_config` function from the parent module.
// - Serializes the request body and sends a POST request.
// - Handles response status and deserialization.
// - Logs request and response details.

use std::error::Error;

// Use the shared config getter and ApiError type from the parent module
use super::{
    ApiError,
    get_api_config,
};
// Import the specific request and response models needed for this function
use crate::api_models::{
    CreateChangeSetV1Request,
    CreateChangeSetV1Response,
};

/// Creates a new change set in the specified workspace.
/// Intention: Calls the `POST /v1/w/{workspace_id}/change-sets` endpoint.
/// Design: Uses the initialized `reqwest::Client`, constructs the URL,
///         serializes the request body (`CreateChangeSetV1Request`), sends a POST request,
///         and deserializes the JSON response into `CreateChangeSetV1Response`.
///         Includes logging similar to other API functions.
/// Returns: A tuple containing the `CreateChangeSetV1Response` on success and a `Vec<String>` of log messages.
pub async fn create_change_set(
    workspace_id: &str,
    request_body: CreateChangeSetV1Request, // Use imported type directly
) -> Result<
    (CreateChangeSetV1Response, Vec<String>), // Use imported type directly
    Box<dyn Error + Send + Sync>,
> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!("{}/v1/w/{}/change-sets", config.base_url, workspace_id);
    logs.push(format!("Calling API: POST {}", url));
    logs.push(format!("Request Body: {:?}", request_body)); // Log the request body

    let response = config
        .client
        .post(&url)
        .json(&request_body) // Serialize the request body struct to JSON
        .send()
        .await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let create_response: CreateChangeSetV1Response = serde_json::from_str(&response_text) // Use imported type directly
            .map_err(|e| {
                format!(
                    "Failed to deserialize create change set response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((create_response, logs))
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
