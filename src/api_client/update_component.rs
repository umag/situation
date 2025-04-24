// src/api_client/update_component.rs

// Intention:
// Contains the implementation for the `PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}` API endpoint call.

// Design Choices:
// - Uses the shared `get_api_config` function from the parent module.
// - Serializes the request body and sends a PUT request.
// - Handles response status and deserialization (expects empty `{}`).
// - Logs request and response details.

use std::error::Error;

// Use the shared config getter and ApiError type from the parent module
use super::{
    ApiError,
    get_api_config,
};
// Import the specific request and response models needed for this function
use crate::api_models::{
    UpdateComponentV1Request,
    UpdateComponentV1Response,
};

/// Updates a specific component within a change set.
/// Corresponds to `PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Operation ID: `update_component`
pub async fn update_component(
    workspace_id: &str,
    change_set_id: &str,
    component_id: &str,
    request_body: UpdateComponentV1Request,
) -> Result<
    (UpdateComponentV1Response, Vec<String>),
    Box<dyn Error + Send + Sync>,
> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components/{}",
        config.base_url, workspace_id, change_set_id, component_id
    );
    logs.push(format!("Calling API: PUT {}", url));
    logs.push(format!("Request Body: {:?}", request_body));

    let response = config.client.put(&url).json(&request_body).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        // Response body is empty `{}`, so deserialize into the empty struct
        let update_response: UpdateComponentV1Response = serde_json::from_str(
            &response_text,
        )
        .map_err(|e| {
            format!(
                "Failed to deserialize update component response: {} - Body: {}",
                e, response_text
            )
        })?;
        Ok((update_response, logs))
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
