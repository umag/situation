// src/api_client/create_component.rs

// Intention:
// Contains the implementation for the `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components` API endpoint call.

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
    CreateComponentV1Request,
    CreateComponentV1Response,
};

/// Creates a new component within a specific change set.
/// Corresponds to `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components`.
/// Operation ID: `create_component`
pub async fn create_component(
    workspace_id: &str,
    change_set_id: &str,
    request_body: CreateComponentV1Request,
) -> Result<
    (CreateComponentV1Response, Vec<String>),
    Box<dyn Error + Send + Sync>,
> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components",
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: POST {}", url));
    logs.push(format!("Request Body: {:?}", request_body));

    let response = config.client.post(&url).json(&request_body).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let create_response: CreateComponentV1Response = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Failed to deserialize create component response: {} - Body: {}",
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
