// src/api_client/force_apply.rs

// Intention:
// Contains the implementation for the `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply` API endpoint call.

// Design Choices:
// - Uses the shared `get_api_config` function from the parent module.
// - Sends a POST request with no body.
// - Handles response status (expects 200 OK with empty body).
// - Logs request and response details.

use std::error::Error;

// Use the shared config getter and ApiError type from the parent module
use super::{
    ApiError,
    get_api_config,
};

/// Force applies a specific change set.
/// Corresponds to `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply`.
/// Operation ID: `force_apply` (Matches OpenAPI spec)
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace containing the change set.
/// * `change_set_id` - The ID of the change set to force apply.
///
/// # Returns
/// A `Result` containing `()` on success (as the API returns no body), or an error string on failure.
/// Also returns a `Vec<String>` containing logs generated during the call.
///
/// # Intention
/// Provides the functionality to force apply a change set via the API.
///
/// # Design
/// - Constructs the specific URL for the force apply endpoint.
/// - Uses the shared `reqwest` client and configuration (via `get_api_config`).
/// - Sends an HTTP POST request (with no body).
/// - Handles success (200 OK, empty body according to OpenAPI spec) and error responses similarly to other API client functions.
/// - Logs relevant information about the request and response.
pub async fn force_apply(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<((), Vec<String>), Box<dyn Error + Send + Sync>> {
    // Return type is correct (unit tuple)
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/force_apply", // Added /force_apply
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: POST {}", url));

    // Send POST request with no body
    let response = config.client.post(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        // Success response has no body according to OpenAPI spec
        let response_text = response.text().await?; // Read body anyway for logging
        logs.push(format!(
            "API Success Body (expected empty): {}",
            response_text
        ));
        Ok(((), logs)) // Return unit tuple for success
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
