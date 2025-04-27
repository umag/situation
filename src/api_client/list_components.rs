// src/api_client/list_components.rs

// Intention:
// Defines the API client function to fetch the list of components associated with a specific change set.

// Design Choices:
// - Follows the pattern established by other API client functions in this project.
// - Takes workspace_id and change_set_id as arguments.
// - Uses the shared `get_api_config` helper from the parent module to get the reqwest client and base URL.
// - Deserializes the response into the `ListComponentsV1Response` struct defined in `api_models.rs`.
// - Returns a `Result` containing the response data and any logs generated during the call.
// - Refactored (2025-04-27): Changed to follow the pattern in list_schemas.rs, using get_api_config directly. Added basic logging.

use std::error::Error;

use reqwest::Method; // Keep Method import for clarity, even if not used directly in this version

// Use the shared config getter from the parent module
use super::get_api_config;
// Use models from the crate root
use crate::api_models::{
    ApiError,
    ListComponentsV1Response,
};

/// Fetches the list of components for a given workspace and change set.
///
/// # Arguments
///
/// * `workspace_id` - The ID of the workspace.
/// * `change_set_id` - The ID of the change set.
///
/// # Returns
///
/// A `Result` containing:
/// - Ok: A tuple with `ListComponentsV1Response` and a `Vec<String>` of logs.
/// - Err: A `Box<dyn Error + Send + Sync>` indicating an error occurred.
pub async fn list_components(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<(ListComponentsV1Response, Vec<String>), Box<dyn Error + Send + Sync>>
{
    let mut logs = Vec::new();

    // Get the static ApiConfig reference containing the client and base URL
    let config = get_api_config()?; // Propagate config error

    // Construct the URL
    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components",
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("API Call: GET {}", url));

    // Make the GET request using the configured client
    let response = config.client.get(&url).send().await?; // Propagate request error

    let status = response.status();
    logs.push(format!("Response Status: {}", status));

    if status.is_success() {
        // Deserialize the successful response
        let response_body = response.json::<ListComponentsV1Response>().await?; // Propagate JSON parsing error
        logs.push(
            "Successfully deserialized ListComponentsV1Response.".to_string(),
        );
        Ok((response_body, logs))
    } else {
        // Attempt to deserialize the error response as ApiError
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("Error response body: {}", error_text));

        let error_message = match serde_json::from_str::<ApiError>(&error_text)
        {
            Ok(api_error) => format!(
                "API Error listing components ({}): {}",
                api_error.status_code, api_error.message
            ),
            Err(_) => format!(
                "API request failed listing components with status {}: {}",
                status, error_text
            ),
        };
        logs.push(error_message.clone());
        Err(error_message.into()) // Return the formatted error message
    }
}
