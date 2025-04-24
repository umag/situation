// src/api_client/whoami.rs

// Intention:
// Contains the implementation for the `/whoami` API endpoint call.

// Design Choices:
// - Uses the shared `get_api_config` function from the parent module.
// - Handles response status and deserialization.
// - Logs request and response details.

use std::error::Error;

// Use the shared config getter and ApiError type from the parent module
use super::{
    get_api_config,
    ApiError,
};
// Import the specific response model needed for this function
use crate::api_models::WhoamiResponse;

/// Fetches user information from the `/whoami` endpoint.
/// Intention: Calls the actual `/whoami` API endpoint using configuration from `.env`.
/// Design: Uses the initialized `reqwest::Client` and constructs the URL.
///         Sends a GET request and deserializes the JSON response into `WhoamiResponse`.
/// Verification (2025-04-21):
///   - Confirmed endpoint (`GET /whoami`) matches OpenAPI spec and luminork service impl.
///   - Confirmed Bearer token authentication matches expectations.
///   - Confirmed `WhoamiResponse` struct in `api_models.rs` matches the actual runtime response structure
///     (Note: `token` field is an object, differing from OpenAPI spec/service code which suggested string).
/// Returns: A tuple containing the `WhoamiResponse` on success and a `Vec<String>` of log messages.
pub async fn whoami() -> Result<(WhoamiResponse, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference using the unified function
    let config = get_api_config()?;

    let url = format!("{}/whoami", config.base_url);
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        // Attempt to deserialize the successful response
        let response_text = response.text().await?; // Read body first for logging
        logs.push(format!("API Success Body: {}", response_text));
        let whoami_data: WhoamiResponse = serde_json::from_str(&response_text).map_err(|e| {
            format!(
                "Failed to deserialize success response: {} - Body: {}",
                e, response_text
            )
        })?;
        // logs.push(format!("API Response Data Parsed: {:?}", whoami_data)); // Maybe too verbose?
        Ok((whoami_data, logs))
    } else {
        // Verification (2025-04-21):
        // - OpenAPI spec lists 401/403 for /whoami but doesn't explicitly link ApiError schema.
        // - Luminork service code doesn't show explicit ApiError construction for these statuses.
        // - Current generic error handling (status + text body) is acceptable.
        // TODO: Consider attempting to parse the error body as `ApiError` in the future
        //       if the API guarantees that structure for 4xx/5xx errors.
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        // Attempt to parse as ApiError for more structured logging, but fall back
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into()) // Return the error message, logs are not returned on error path
    }
}
