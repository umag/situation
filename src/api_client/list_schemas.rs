// src/api_client/list_schemas.rs

// Intention: Defines the API client function to list available schemas for a given workspace and change set.
// Design Choice: Follows the pattern of other API client functions in this module.
// Uses the shared `get_api_client_config` and `make_api_request` helpers (assuming they exist in mod.rs or similar).
// Returns a Result containing the ListSchemaV1Response or an error.

use std::error::Error;

use reqwest::Method; // Method is not used directly anymore, but keep reqwest imports if needed

// Use the shared config getter and ApiError type from the parent module
use super::{
    ApiError,
    get_api_config,
};
use crate::api_models::{
    ApiError,
    ListSchemaV1Response,
}; // Use crate:: for models within the library

/// Fetches the list of schemas for a specific workspace and change set.
///
/// # Arguments
///
/// * `workspace_id` - The ID of the workspace.
/// * `change_set_id` - The ID of the change set.
///
/// # Returns
///
/// A `Result` containing either:
/// - `Ok(ListSchemaV1Response)`: The successfully fetched schema list.
/// - `Err(Box<dyn Error + Send + Sync>)`: An error if the request failed.
/// Design Choice: Follows pattern of list_change_sets.rs, handles response directly.
pub async fn list_schemas(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<ListSchemaV1Response, Box<dyn Error + Send + Sync>> {
    // Get the static ApiConfig reference containing the client and base URL
    let config = get_api_config()?;

    // Construct the URL
    let url = format!(
        "{}/v1/w/{}/change-sets/{}/schema",
        config.base_url, workspace_id, change_set_id
    );

    // Make the GET request using the configured client
    let response = config.client.get(&url).send().await?;

    let status = response.status();

    if status.is_success() {
        // Deserialize the successful response
        let response_body = response.json::<ListSchemaV1Response>().await?;
        Ok(response_body)
    } else {
        // Attempt to deserialize the error response as ApiError
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        let error_message = match serde_json::from_str::<ApiError>(&error_text)
        {
            Ok(api_error) => format!(
                "API Error listing schemas ({}): {}",
                api_error.status_code, api_error.message
            ),
            Err(_) => format!(
                "API request failed listing schemas with status {}: {}",
                status, error_text
            ),
        };
        Err(error_message.into()) // Return the formatted error message
    }
}
