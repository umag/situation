// src/api_client/get_component.rs

// Intention:
// Contains the implementation for the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}` API endpoint call.

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
use crate::api_models::GetComponentV1Response;

/// Fetches details for a specific component within a change set.
/// Corresponds to `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Operation ID: `get_component`
pub async fn get_component(
    workspace_id: &str,
    change_set_id: &str,
    component_id: &str,
) -> Result<(GetComponentV1Response, Vec<String>), Box<dyn Error + Send + Sync>>
{
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components/{}",
        config.base_url, workspace_id, change_set_id, component_id
    );
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let get_response: GetComponentV1Response = serde_json::from_str(
            &response_text,
        )
        .map_err(|e| {
            format!(
                "Failed to deserialize get component response: {} - Body: {}",
                e, response_text
            )
        })?;
        Ok((get_response, logs))
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
