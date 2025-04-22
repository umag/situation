// src/api_client.rs

// Intention:
// Provides functions to interact with the Luminork/Systeminit API.
// Currently contains placeholder implementations until the actual API client
// Handles API interactions.

// Design Choices:
// - Functions are async using `reqwest` for network calls.
// - Returns `Result` to handle potential errors (env var loading, network, deserialization, API errors).
// - Uses the data structures defined in `api_models.rs`.
// - Loads API URL and JWT token from environment variables using `dotenvy`.
// - Creates a reusable `reqwest::Client` initialized lazily via `OnceLock`.
// - Returns results containing both the data and logs for TUI display.

use crate::api_models::{
    ApiError,
    CreateChangeSetV1Request,
    CreateChangeSetV1Response,
    GetChangeSetV1Response, // Added GetChangeSetV1Response
    ListChangeSetV1Response,
    WhoamiResponse,
};
use dotenvy::dotenv;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use std::env;
use std::error::Error;
use std::sync::OnceLock; // Use OnceLock for lazy static initialization

// Intention: Lazily initialize the reqwest client and load env vars once.
// Design Choice: Use OnceLock for thread-safe, one-time initialization.
// Stores the API base URL, JWT token, and the reqwest client.
struct ApiConfig {
    client: reqwest::Client,
    base_url: String,
    jwt_token: String,
}

static API_CONFIG: OnceLock<Result<ApiConfig, Box<dyn Error + Send + Sync>>> =
    OnceLock::new();

// Helper function to create a config instance. Used by get_api_config.
fn create_new_api_config() -> Result<ApiConfig, Box<dyn Error + Send + Sync>> {
    dotenv().ok(); // Load .env file, ignore errors if it doesn't exist

    let base_url = env::var("SI_API")
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    let jwt_token = env::var("JWT_TOKEN")
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

    let mut headers = HeaderMap::new();
    let mut auth_value =
        HeaderValue::from_str(&format!("Bearer {}", jwt_token))
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    auth_value.set_sensitive(true);
    headers.insert(AUTHORIZATION, auth_value);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

    Ok(ApiConfig {
        client,
        base_url,
        jwt_token,
    }) // jwt_token stored for potential future use/refresh
}

// In test environments, create a new client each time to avoid runtime conflicts.
#[cfg(test)]
fn get_api_config() -> Result<ApiConfig, Box<dyn Error + Send + Sync>> {
    create_new_api_config()
}

// In non-test environments, use OnceLock for efficiency.
#[cfg(not(test))]
fn get_api_config()
-> Result<&'static ApiConfig, &'static (dyn Error + Send + Sync)> {
    API_CONFIG
        .get_or_init(create_new_api_config) // Use the helper function here
        .as_ref()
        .map_err(|e| &**e) // Convert Box<dyn Error> to &dyn Error
}

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
pub async fn whoami()
-> Result<(WhoamiResponse, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // In tests, this returns an owned ApiConfig. In non-tests, a static ref.
    // We need to handle both cases. Let's borrow it if it's owned.
    #[cfg(test)]
    let config_holder = get_api_config()?;
    #[cfg(test)]
    let config = &config_holder; // Borrow the owned config

    #[cfg(not(test))]
    let config = get_api_config()?; // Get static ref

    let url = format!("{}/whoami", config.base_url);
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        // Attempt to deserialize the successful response
        let response_text = response.text().await?; // Read body first for logging
        logs.push(format!("API Success Body: {}", response_text));
        let whoami_data: WhoamiResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
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
        Err(error_message.into()) // Return the error message, logs are not returned on error path
    }
}

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
    #[cfg(test)]
    let config_holder = get_api_config()?;
    #[cfg(test)]
    let config = &config_holder;

    #[cfg(not(test))]
    let config = get_api_config()?;

    let url = format!("{}/v1/w/{}/change-sets", config.base_url, workspace_id);
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let list_response: ListChangeSetV1Response = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to deserialize list change sets response: {} - Body: {}", e, response_text))?;
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
    #[cfg(test)]
    let config_holder = get_api_config()?;
    #[cfg(test)]
    let config = &config_holder;

    #[cfg(not(test))]
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
            .map_err(|e| format!("Failed to deserialize create change set response: {} - Body: {}", e, response_text))?;
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

/// Fetches details for a specific change set.
/// Corresponds to `GET /v1/w/{workspace_id}/change-sets/{change_set_id}`.
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace.
/// * `change_set_id` - The ID of the change set to fetch.
///
/// # Returns
/// A `Result` containing the `GetChangeSetV1Response` on success, or an error string.
/// Also returns a `Vec<String>` containing logs generated during the call.
pub async fn get_change_set(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<(GetChangeSetV1Response, Vec<String>), Box<dyn Error + Send + Sync>>
{
    let mut logs = Vec::new();
    #[cfg(test)]
    let config_holder = get_api_config()?;
    #[cfg(test)]
    let config = &config_holder;

    #[cfg(not(test))]
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}",
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let get_response: GetChangeSetV1Response = serde_json::from_str(
            &response_text,
        )
        .map_err(|e| {
            format!(
                "Failed to deserialize get change set response: {} - Body: {}",
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

// TODO: Add functions for other API endpoints as needed.
// Examples:
// pub async fn get_component(workspace_id: &str, change_set_id: &str, component_id: &str) -> Result<(GetComponentV1Response, Vec<String>), Box<dyn Error + Send + Sync>> { ... }

// Note: The error type `Box<dyn Error + Send + Sync>` is used for flexibility,
// allowing different error types (IO, HTTP, API errors) to be returned.
// Consider defining a custom error enum for more specific error handling later.
